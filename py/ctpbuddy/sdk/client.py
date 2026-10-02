"""Synchronous client for the CTPBuddy core wire protocol.

One `Client` = one TCP connection = one CTP front session. Request frames are
correlated by `req_id`; unsolicited pushes (`req_id == 0`: RTN_ORDER,
RTN_TRADE, RTN_DEPTH_MD ...) land in a queue that `events()` drains, mirroring
the shim's SPI callback thread.
"""
from __future__ import annotations

import json
import queue
import socket
import threading
import time
from typing import Any, Dict, Iterable, List, Optional

from .. import generated
from ..wire import (
    AUTH,
    AUTH_RSP,
    ERR_RTN_ORDER_ACTION,
    ERR_RTN_ORDER_INSERT,
    LOGOUT,
    LOGOUT_RSP,
    QRY_LAST,
    REQ_ORDER_ACTION,
    REQ_ORDER_INSERT,
    REQ_QRY_INSTRUMENT,
    REQ_QRY_INVESTOR_POSITION,
    REQ_QRY_ORDER,
    REQ_QRY_TRADE,
    REQ_QRY_TRADING_ACCOUNT,
    REQ_SETTLE_CONFIRM,
    REQ_USER_LOGIN,
    REQ_USER_LOGOUT,
    RSP_ERROR,
    RSP_ORDER_ACTION,
    RSP_ORDER_INSERT,
    RSP_QRY_INSTRUMENT,
    RSP_QRY_INVESTOR_POSITION,
    RSP_QRY_ORDER,
    RSP_QRY_TRADE,
    RSP_QRY_TRADING_ACCOUNT,
    RSP_SETTLE_CONFIRM,
    RSP_SUB_MD,
    RSP_UNSUB_MD,
    RSP_USER_LOGIN,
    RSP_USER_LOGOUT,
    RTN_DEPTH_MD,
    RTN_ORDER,
    RTN_TRADE,
    SUB_MD,
    UNSUB_MD,
    Frame,
)

DEFAULT_TIMEOUT = 10.0
#: Sentinel pushed into every wait queue when the connection dies.
CLOSED = object()


class CTPError(Exception):
    """A failed request: RSP_ERROR / ERR_RTN_* with ErrorID + ErrorMsg."""

    def __init__(self, error_id: int, msg: str, kind: str = "RSP_ERROR") -> None:
        super().__init__("[%s %d] %s" % (kind, error_id, msg))
        self.error_id = error_id
        self.msg = msg
        self.kind = kind


class Client:
    def __init__(self, addr: str = "127.0.0.1:5560", timeout: float = DEFAULT_TIMEOUT) -> None:
        host, _, port = addr.rpartition(":")
        self.sock = socket.create_connection((host, int(port)), timeout=timeout)
        self.sock.setsockopt(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)
        self._req_seq = 0
        self._pending: Dict[int, "queue.Queue[Frame]"] = {}
        self._pushes: "queue.Queue[Frame]" = queue.Queue()
        self._lock = threading.Lock()
        self._closed = False
        self._reader = threading.Thread(target=self._read_loop, daemon=True)
        self._reader.start()

    # ---- connection -------------------------------------------------------

    def close(self) -> None:
        if not self._closed:
            self._closed = True
            try:
                self.sock.shutdown(socket.SHUT_RDWR)
            except OSError:
                pass
            self.sock.close()

    def __enter__(self) -> "Client":
        return self

    def __exit__(self, *exc: Any) -> None:
        self.close()

    # ---- low level --------------------------------------------------------

    def _next_req_id(self) -> int:
        with self._lock:
            self._req_seq += 1
            return self._req_seq

    def _read_loop(self) -> None:
        try:
            while True:
                f = Frame.decode_from(self.sock)
                if f is None:
                    break
                if f.req_id == 0:
                    self._pushes.put(f)
                else:
                    q = self._pending.get(f.req_id)
                    if q is not None:
                        q.put(f)
        except (OSError, ValueError):
            pass
        finally:
            # wake up every waiter so they fail loudly instead of hanging
            for q in list(self._pending.values()):
                q.put(CLOSED)
            self._pushes.put(CLOSED)

    def _wake_all(self) -> None:
        for q in list(self._pending.values()):
            q.put(CLOSED)
        self._pushes.put(CLOSED)

    def _request(self, msg_type: int, payload: bytes = b"", expect: Optional[int] = None,
                 timeout: Optional[float] = None) -> Frame:
        req_id = self._next_req_id()
        q: "queue.Queue" = queue.Queue()
        self._pending[req_id] = q
        try:
            self.sock.sendall(Frame(msg_type, req_id, payload).encode())
            f = q.get(timeout=timeout or DEFAULT_TIMEOUT)
        finally:
            self._pending.pop(req_id, None)
        if f is CLOSED or f.msg_type == RSP_ERROR or f.msg_type == ERR_RTN_ORDER_INSERT:
            info = generated.unpack("CThostFtdcRspInfoField", f.payload) if f is not CLOSED else {}
            raise CTPError(info.get("ErrorID", -1), info.get("ErrorMsg", "connection closed"))
        if expect is not None and f.msg_type != expect:
            raise CTPError(-2, "expected msg 0x%04x, got 0x%04x" % (expect, f.msg_type))
        return f

    # Front per-second query budget retries (CTP docs: 查询流控 / front_se QryFreq).
    _QRY_MAX_ATTEMPTS = 8

    def _query_stream(self, req_msg: int, rsp_msg: int, payload: bytes) -> List[bytes]:
        """Send a query request and collect RSP payloads until QRY_LAST.

        The front's per-second query budget (CTP docs: 查询流控, front_se
        QryFreq) answers an over-budget query with RSP_ERROR[90]
        "CTP：查询未就绪，请稍后重试" -- the query does not run. Real CTP
        clients wait past the one-second window and re-issue; the SDK
        absorbs that loop so callers never see the throttle.
        """
        for _ in range(self._QRY_MAX_ATTEMPTS):
            out, throttled = self._query_once(req_msg, rsp_msg, payload)
            if not throttled:
                return out
            time.sleep(1.1)  # past the one-second QryFreq window
        raise CTPError(90, "CTP：查询未就绪，请稍后重试 (front query budget exhausted)")

    def _query_once(self, req_msg: int, rsp_msg: int, payload: bytes) -> "tuple[List[bytes], bool]":
        """One query attempt: `(rows, throttled)`; throttled -> re-issue."""
        req_id = self._next_req_id()
        q: "queue.Queue" = queue.Queue()
        self._pending[req_id] = q
        out: List[bytes] = []
        try:
            self.sock.sendall(Frame(req_msg, req_id, payload).encode())
            while True:
                f = q.get(timeout=DEFAULT_TIMEOUT)
                if f is CLOSED:
                    raise CTPError(-1, "connection closed")
                if f.msg_type == QRY_LAST:
                    return out, False
                if f.msg_type == RSP_ERROR:
                    info = generated.unpack("CThostFtdcRspInfoField", f.payload)
                    if info.get("ErrorID") == 90:
                        return out, True  # NEED_RETRY: budget spent, nothing ran
                    raise CTPError(info["ErrorID"], info["ErrorMsg"])
                if f.msg_type == rsp_msg:
                    out.append(f.payload)
        finally:
            self._pending.pop(req_id, None)

    def ping(self) -> None:
        """Protocol-level liveness (no session needed)."""
        from ..wire import PING, PONG

        self._request(PING, b"", expect=PONG)

    # ---- session ----------------------------------------------------------

    def auth(self, broker_id: str, user_id: str, app_id: str = "ctpbuddy-python") -> Dict[str, Any]:
        payload = json.dumps({"broker_id": broker_id, "user_id": user_id, "app_id": app_id}).encode()
        f = self._request(AUTH, payload, expect=AUTH_RSP)
        v = json.loads(f.payload)
        if not v.get("ok"):
            raise CTPError(-1, v.get("error", "auth failed"), kind="AUTH")
        return v

    def login(self, broker_id: str, user_id: str, password: str = "") -> Dict[str, Any]:
        payload = generated.pack(
            "CThostFtdcReqUserLoginField",
            BrokerID=broker_id,
            UserID=user_id,
            Password=password,
            UserProductInfo="ctpbuddy",
        )
        f = self._request(REQ_USER_LOGIN, payload, expect=RSP_USER_LOGIN)
        field = generated.unpack("CThostFtdcRspUserLoginField", f.payload)
        self.front_id = field["FrontID"]
        self.session_id = field["SessionID"]
        self.broker_id = broker_id
        self.investor_id = user_id
        return field

    def logout(self) -> None:
        payload = generated.pack("CThostFtdcUserLogoutField", BrokerID=self.broker_id, UserID=self.investor_id)
        self._request(REQ_USER_LOGOUT, payload, expect=RSP_USER_LOGOUT)

    def settle_confirm(self) -> Dict[str, Any]:
        payload = generated.pack(
            "CThostFtdcSettlementInfoConfirmField",
            BrokerID=self.broker_id,
            InvestorID=self.investor_id,
        )
        f = self._request(REQ_SETTLE_CONFIRM, payload, expect=RSP_SETTLE_CONFIRM)
        return generated.unpack("CThostFtdcSettlementInfoConfirmField", f.payload)

    # ---- orders -----------------------------------------------------------

    def order_insert(
        self,
        instrument: str,
        direction: str,
        offset: str,
        volume: int,
        limit_price: float,
        exchange: str = "",
        order_ref: str = "",
        price_type: str = "2",  # '2' = limit; '1' = any price (market)
        time_condition: str = "1",  # GFD
        volume_condition: str = "1",  # any volume
        **extra: Any,
    ) -> Frame:
        """Send an order; returns the RSP_ORDER_INSERT frame (accepted at front).

        Order status transitions and trades arrive asynchronously via events().
        Raises CTPError on immediate rejection (RSP_ERROR / ERR_RTN_ORDER_INSERT).
        """
        payload = generated.pack(
            "CThostFtdcInputOrderField",
            BrokerID=self.broker_id,
            InvestorID=self.investor_id,
            InstrumentID=instrument,
            ExchangeID=exchange,
            OrderRef=order_ref,
            UserID=self.investor_id,
            OrderPriceType=ord(price_type),
            Direction=ord(direction),
            CombOffsetFlag=offset,
            CombHedgeFlag="1",
            LimitPrice=limit_price,
            VolumeTotalOriginal=volume,
            TimeCondition=ord(time_condition),
            VolumeCondition=ord(volume_condition),
            MinVolume=1,
            ContingentCondition=ord("1"),
            StopPrice=0.0,
            ForceCloseReason=ord("0"),
            IsAutoSuspend=0,
            BusinessUnit="",
            **extra,
        )
        f = self._request(REQ_ORDER_INSERT, payload, expect=RSP_ORDER_INSERT)
        return f

    def order_action(
        self,
        instrument: str,
        order_ref: str,
        front_id: Optional[int] = None,
        session_id: Optional[int] = None,
        action_flag: str = "0",
    ) -> Frame:
        payload = generated.pack(
            "CThostFtdcInputOrderActionField",
            BrokerID=self.broker_id,
            InvestorID=self.investor_id,
            OrderRef=order_ref,
            FrontID=front_id if front_id is not None else getattr(self, "front_id", 0),
            SessionID=session_id if session_id is not None else getattr(self, "session_id", 0),
            InstrumentID=instrument,
            ActionFlag=ord(action_flag),
            UserID=self.investor_id,
        )
        return self._request(REQ_ORDER_ACTION, payload, expect=RSP_ORDER_ACTION)

    # ---- market data ------------------------------------------------------

    def subscribe(self, instruments: Iterable[str]) -> Frame:
        # CTP wire format: back-to-back CThostFtdcSpecificInstrumentField structs.
        # The core answers one RSP_SUB_MD per instrument (same req_id).
        payload = b"".join(
            generated.pack("CThostFtdcSpecificInstrumentField", InstrumentID=i) for i in instruments
        )
        return self._request(SUB_MD, payload, expect=RSP_SUB_MD)

    def unsubscribe(self, instruments: Iterable[str]) -> Frame:
        payload = b"".join(
            generated.pack("CThostFtdcSpecificInstrumentField", InstrumentID=i) for i in instruments
        )
        return self._request(UNSUB_MD, payload, expect=RSP_UNSUB_MD)

    # ---- queries ----------------------------------------------------------

    def qry_instrument(self, instrument: str = "") -> List[Dict[str, Any]]:
        rows = self._query_stream(
            REQ_QRY_INSTRUMENT,
            RSP_QRY_INSTRUMENT,
            generated.pack("CThostFtdcQryInstrumentField", InstrumentID=instrument),
        )
        return [generated.unpack("CThostFtdcInstrumentField", r) for r in rows]

    def qry_trading_account(self) -> Dict[str, Any]:
        rows = self._query_stream(
            REQ_QRY_TRADING_ACCOUNT, RSP_QRY_TRADING_ACCOUNT, b""
        )
        if not rows:
            raise CTPError(-1, "empty trading account query")
        return generated.unpack("CThostFtdcTradingAccountField", rows[0])

    def qry_investor_position(self, instrument: str = "") -> List[Dict[str, Any]]:
        rows = self._query_stream(
            REQ_QRY_INVESTOR_POSITION,
            RSP_QRY_INVESTOR_POSITION,
            generated.pack("CThostFtdcQryInvestorPositionField", BrokerID=self.broker_id, InvestorID=self.investor_id, InstrumentID=instrument),
        )
        return [generated.unpack("CThostFtdcInvestorPositionField", r) for r in rows]

    def qry_order(self, instrument: str = "") -> List[Dict[str, Any]]:
        rows = self._query_stream(
            REQ_QRY_ORDER,
            RSP_QRY_ORDER,
            generated.pack("CThostFtdcQryOrderField", BrokerID=self.broker_id, InvestorID=self.investor_id, InstrumentID=instrument),
        )
        return [generated.unpack("CThostFtdcOrderField", r) for r in rows]

    def qry_trade(self, instrument: str = "") -> List[Dict[str, Any]]:
        rows = self._query_stream(
            REQ_QRY_TRADE,
            RSP_QRY_TRADE,
            generated.pack("CThostFtdcQryTradeField", BrokerID=self.broker_id, InvestorID=self.investor_id, InstrumentID=instrument),
        )
        return [generated.unpack("CThostFtdcTradeField", r) for r in rows]

    # ---- pushes -----------------------------------------------------------

    def events(self, timeout: Optional[float] = None):
        """Blocking iterator over push frames (RTN_ORDER / RTN_TRADE / RTN_DEPTH_MD)."""
        while True:
            item = self._pushes.get(timeout=timeout)
            if item is CLOSED:
                return
            f = item
            if f.msg_type in (RTN_ORDER, RTN_TRADE):
                yield f.msg_type, generated.unpack(
                    "CThostFtdcOrderField" if f.msg_type == RTN_ORDER else "CThostFtdcTradeField",
                    f.payload,
                )
            elif f.msg_type == RTN_DEPTH_MD:
                yield f.msg_type, generated.unpack("CThostFtdcDepthMarketDataField", f.payload)
