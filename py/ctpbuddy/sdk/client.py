"""Synchronous client for the CTPBuddy core wire protocol.

One `Client` = one TCP connection = one CTP front session. Request frames are
correlated by `req_id`; unsolicized pushes (`req_id == 0`: RTN_ORDER,
RTN_TRADE, RTN_DEPTH_MD ...) land in a queue that `events()` drains, mirroring
the shim's SPI callback thread.

Rejection surfaces (DESIGN §8.12, #42) mirror real CTP's two callbacks:
- the front-office half answers the pending request itself via `RSP_ERROR`
  (`OnRspOrderInsert`/`OnRspOrderAction`, input NULL) -- `order_insert()` /
  `order_action()` raise `CTPError` from it;
- the exchange half arrives *after* a successful response via
  `ERR_RTN_ORDER_INSERT` / `ERR_RTN_ORDER_ACTION` (`OnErrRtn*`) -- the request
  already returned, so those frames land in `late_frames` (a real CTP app gets
  them on its OnErrRtn* hook, never as a request failure).
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
    REQ_QRY_BROKER_TRADING_PARAMS,
    REQ_QRY_INSTRUMENT,
    REQ_QRY_INSTRUMENT_COMMISSION_RATE,
    REQ_QRY_INSTRUMENT_MARGIN_RATE,
    REQ_QRY_INSTRUMENT_ORDER_COMM_RATE,
    REQ_QRY_INVESTOR_POSITION,
    REQ_QRY_INVESTOR_POSITION_DETAIL,
    REQ_QRY_ORDER,
    REQ_QRY_TRADE,
    REQ_QRY_TRADING_ACCOUNT,
    REQ_SETTLE_CONFIRM,
    REQ_USER_LOGIN,
    REQ_USER_LOGOUT,
    RSP_ERROR,
    RSP_ORDER_ACTION,
    RSP_ORDER_INSERT,
    RSP_QRY_BROKER_TRADING_PARAMS,
    RSP_QRY_INSTRUMENT,
    RSP_QRY_INSTRUMENT_COMMISSION_RATE,
    RSP_QRY_INSTRUMENT_MARGIN_RATE,
    RSP_QRY_INSTRUMENT_ORDER_COMM_RATE,
    RSP_QRY_INVESTOR_POSITION,
    RSP_QRY_INVESTOR_POSITION_DETAIL,
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

#: `THOST_FTDC_HF_Speculation` — rate queries are conventionally sent with
#: 投机套保标志; the core answers with the row it stores for this flag.
HEDGE_FLAG_SPECULATION = ord("1")

DEFAULT_TIMEOUT = 10.0
#: Sentinel pushed into every wait queue when the connection dies.
CLOSED = object()

#: Rejection surfaces (DESIGN §8.12): `RSP_ERROR` answers the pending request
#: itself (the front-office half); `ERR_RTN_*` arrive after the request has
#: already completed (the exchange half) and are captured by `late_frames`.
REJECTION_FRAMES = (RSP_ERROR, ERR_RTN_ORDER_INSERT, ERR_RTN_ORDER_ACTION)

#: How many post-completion (late) frames to keep per client.
LATE_LIMIT = 256


def rsp_info_of(payload: bytes) -> Dict[str, Any]:
    """RspInfoField of a rejection frame: the bare struct on `RSP_ERROR`, the
    trailing struct on `ERR_RTN_*` (payload = input struct ++ RspInfoField)."""
    n = generated.SIZES["CThostFtdcRspInfoField"]
    if len(payload) < n:
        return {}

    return generated.unpack("CThostFtdcRspInfoField", payload[-n:])


class CTPError(Exception):
    """A failed request: RSP_ERROR / ERR_RTN_* with ErrorID + ErrorMsg."""

    def __init__(self, error_id: int, msg: str, kind: str = "RSP_ERROR") -> None:
        super().__init__("[%s %d] %s" % (kind, error_id, msg))
        self.error_id = error_id
        self.msg = msg
        self.kind = kind

    @classmethod
    def from_frame(cls, f: Frame) -> "CTPError":
        """Decode a rejection frame, preserving its真实推送面 as `kind`."""
        if f is CLOSED:
            return cls(-1, "connection closed", kind="CLOSED")
        if f.msg_type == ERR_RTN_ORDER_INSERT:
            kind = "ERR_RTN_ORDER_INSERT"
        elif f.msg_type == ERR_RTN_ORDER_ACTION:
            kind = "ERR_RTN_ORDER_ACTION"
        else:
            kind = "RSP_ERROR"
        info = rsp_info_of(f.payload) if f.payload else {}
        return cls(info.get("ErrorID", -1), info.get("ErrorMsg", ""), kind=kind)


class _Pending:
    """One in-flight request: its frame queue plus how many frames it takes.

    `multi` requests (ReqQry*, answered by a row stream terminated by
    QRY_LAST) keep consuming frames; unary requests are completed by their
    first response, and any further rejection frame on the same req_id is the
    错单回报 half (DESIGN §8.12) which the reader diverts to the late queue.
    """

    __slots__ = ("q", "multi", "answered")

    def __init__(self, multi: bool = False) -> None:
        self.q: "queue.Queue[Frame]" = queue.Queue()
        self.multi = multi
        self.answered = False


class Client:
    def __init__(self, addr: str = "127.0.0.1:5560", timeout: float = DEFAULT_TIMEOUT) -> None:
        host, _, port = addr.rpartition(":")
        self.sock = socket.create_connection((host, int(port)), timeout=timeout)
        self.sock.setsockopt(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)
        self._req_seq = 0
        self._pending: Dict[int, _Pending] = {}
        self._pushes: "queue.Queue[Frame]" = queue.Queue()
        self._lock = threading.Lock()
        self._late: List[Frame] = []
        self._late_cv = threading.Condition(self._lock)
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

    # ---- late frames (push surfaces that land after their request) -------

    @property
    def late_frames(self) -> List[Frame]:
        """Frames that arrived for a request that had already completed.

        The exchange half of a rejection (`ERR_RTN_ORDER_INSERT` /
        `ERR_RTN_ORDER_ACTION`, DESIGN §8.12) is sent after the front office
        already answered the request, so `_request` has returned by then -- a
        real CTP app receives these on its OnErrRtn* hook, not as a request
        failure. Tests use this list to assert the surface.
        """
        with self._lock:
            return list(self._late)

    def wait_late(self, msg_type: int, timeout: float = 5.0) -> Optional[Frame]:
        """Block until a late frame of `msg_type` shows up (None on timeout)."""
        deadline = time.monotonic() + timeout
        with self._late_cv:
            while True:
                for f in self._late:
                    if f.msg_type == msg_type:
                        return f
                remaining = deadline - time.monotonic()
                if remaining <= 0:
                    return None
                self._late_cv.wait(remaining)

    def clear_late(self) -> None:
        with self._late_cv:
            self._late.clear()

    def _note_late(self, f: Frame) -> None:
        with self._late_cv:
            self._late.append(f)
            if len(self._late) > LATE_LIMIT:
                del self._late[: len(self._late) - LATE_LIMIT]
            self._late_cv.notify_all()

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
                    continue
                p = self._pending.get(f.req_id)
                if p is not None and (p.multi or not p.answered):
                    # first frame of a unary request completes it; a streaming
                    # query keeps collecting until QRY_LAST
                    p.answered = True
                    p.q.put(f)
                elif self._is_rejection(f):
                    # a *second* frame on a completed unary request: this is the
                    # post-response 错单回报 half (#42). Handing it to the
                    # pending queue instead would drop it on the floor -- that
                    # queue has already been read once and never again.
                    self._note_late(f)
        except (OSError, ValueError):
            pass
        finally:
            # wake up every waiter so they fail loudly instead of hanging
            for p in list(self._pending.values()):
                p.q.put(CLOSED)
            self._pushes.put(CLOSED)

    @staticmethod
    def _is_rejection(f: Frame) -> bool:
        return f.msg_type in REJECTION_FRAMES

    def _wake_all(self) -> None:
        for p in list(self._pending.values()):
            p.q.put(CLOSED)
        self._pushes.put(CLOSED)

    def _request(self, msg_type: int, payload: bytes = b"", expect: Optional[int] = None,
                 timeout: Optional[float] = None) -> Frame:
        req_id = self._next_req_id()
        p = _Pending()
        self._pending[req_id] = p
        try:
            self.sock.sendall(Frame(msg_type, req_id, payload).encode())
            f = p.q.get(timeout=timeout or DEFAULT_TIMEOUT)
        finally:
            self._pending.pop(req_id, None)
        if f is CLOSED or self._is_rejection(f):
            # keep the真实推送面 (DESIGN §8.12): the front-office half arrives
            # as RSP_ERROR (OnRspOrderInsert / OnRspOrderAction, input NULL);
            # the exchange half arrives as ERR_RTN_* only after the success
            # Rsp -- downstreams must hook both callbacks. The reader routes
            # that second frame to the late queue, never here.
            raise CTPError.from_frame(f)
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
        p = _Pending(multi=True)
        self._pending[req_id] = p
        out: List[bytes] = []
        try:
            self.sock.sendall(Frame(req_msg, req_id, payload).encode())
            while True:
                f = p.q.get(timeout=DEFAULT_TIMEOUT)
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
        time_condition: str = "3",  # '3' = GFD (official TC_GFD; '1' = IOC)
        volume_condition: str = "1",  # any volume ('2' = min volume, '3' = all)
        min_volume: int = 1,  # only meaningful with VolumeCondition '2' (FAK with MinVolume)
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
            MinVolume=min_volume,
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
        order_sys_id: str = "",
    ) -> Frame:
        payload = generated.pack(
            "CThostFtdcInputOrderActionField",
            BrokerID=self.broker_id,
            InvestorID=self.investor_id,
            OrderRef=order_ref,
            OrderSysID=order_sys_id,
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

    def qry_investor_position_detail(self, instrument: str = "") -> List[Dict[str, Any]]:
        """One row per open lot (notes/04 B2).

        Each row is an independent 逐日盯市 unit: `OpenPrice` for a lot opened
        today, `LastSettlementPrice` for a carried one, and
        `PositionProfitByDate` computed from whichever applies. Summing those
        per-row PnL must equal the aggregate `PositionProfit` from
        `qry_investor_position` -- the property a client uses to verify the
        core is not blending lots of different ages.

        Rows arrive oldest-first within a position (先开先平 order).
        """
        rows = self._query_stream(
            REQ_QRY_INVESTOR_POSITION_DETAIL,
            RSP_QRY_INVESTOR_POSITION_DETAIL,
            generated.pack("CThostFtdcQryInvestorPositionDetailField", BrokerID=self.broker_id, InvestorID=self.investor_id, InstrumentID=instrument),
        )
        return [generated.unpack("CThostFtdcInvestorPositionDetailField", r) for r in rows]

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

    # ---- reference data (notes/04 G) ----
    #
    # These answer from the same rows the ledger charges with, so
    # `qry_instrument_margin_rate` cross-checks against
    # `qry_trading_account()["CurrMargin"]` the way it does on a real desk.
    #
    # Official behaviour worth knowing (6.7.13 API docs):
    #   * an empty `instrument` does **not** mean "every contract" — the core
    #     answers with the rates of the contracts this investor holds, because
    #     "目前无法通过一次查询得到所有合约保证金率". Walk the whole market by
    #     calling once per instrument.
    #   * `BrokerID` / `InvestorID` are mandatory; omitting them yields an empty
    #     stream, which is why these methods always fill them in.

    def qry_instrument_margin_rate(self, instrument: str = "") -> List[Dict[str, Any]]:
        """公司保证金率 — the rate the counter actually freezes with."""
        rows = self._query_stream(
            REQ_QRY_INSTRUMENT_MARGIN_RATE,
            RSP_QRY_INSTRUMENT_MARGIN_RATE,
            generated.pack(
                "CThostFtdcQryInstrumentMarginRateField",
                BrokerID=self.broker_id,
                InvestorID=self.investor_id,
                InstrumentID=instrument,
                HedgeFlag=HEDGE_FLAG_SPECULATION,
            ),
        )
        return [generated.unpack("CThostFtdcInstrumentMarginRateField", r) for r in rows]

    def qry_instrument_commission_rate(self, instrument: str = "") -> List[Dict[str, Any]]:
        """手续费率 — 开仓 / 平昨 / 平今, each ByMoney and ByVolume."""
        rows = self._query_stream(
            REQ_QRY_INSTRUMENT_COMMISSION_RATE,
            RSP_QRY_INSTRUMENT_COMMISSION_RATE,
            generated.pack(
                "CThostFtdcQryInstrumentCommissionRateField",
                BrokerID=self.broker_id,
                InvestorID=self.investor_id,
                InstrumentID=instrument,
            ),
        )
        return [generated.unpack("CThostFtdcInstrumentCommissionRateField", r) for r in rows]

    def qry_instrument_order_comm_rate(self, instrument: str = "") -> List[Dict[str, Any]]:
        """报单/撤单费 (中金所 only; an empty stream elsewhere is correct)."""
        rows = self._query_stream(
            REQ_QRY_INSTRUMENT_ORDER_COMM_RATE,
            RSP_QRY_INSTRUMENT_ORDER_COMM_RATE,
            generated.pack(
                "CThostFtdcQryInstrumentOrderCommRateField",
                BrokerID=self.broker_id,
                InvestorID=self.investor_id,
                InstrumentID=instrument,
            ),
        )
        return [generated.unpack("CThostFtdcInstrumentOrderCommRateField", r) for r in rows]

    def qry_broker_trading_params(self, currency: str = "CNY") -> List[Dict[str, Any]]:
        """交易参数 — `MarginPriceType` decides how 今仓 margin is priced."""
        rows = self._query_stream(
            REQ_QRY_BROKER_TRADING_PARAMS,
            RSP_QRY_BROKER_TRADING_PARAMS,
            generated.pack(
                "CThostFtdcQryBrokerTradingParamsField",
                BrokerID=self.broker_id,
                InvestorID=self.investor_id,
                CurrencyID=currency,
            ),
        )
        return [generated.unpack("CThostFtdcBrokerTradingParamsField", r) for r in rows]

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
