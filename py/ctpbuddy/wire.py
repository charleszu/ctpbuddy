# Keep in sync with core/ctpbuddy-wire/src/frame.rs (framing) and
# core/ctpbuddy-wire/src/msgs.rs (message ids).
"""Wire framing and message ids for the CTPBuddy core protocol.

The Python side speaks the exact same frames as the C++ shim: magic 'CB',
version 1, msg_type (u16 LE), req_id (u32 LE), payload length (u32 LE),
payload. Transport-agnostic by construction (TCP in M1, ZeroMQ later).
"""
from __future__ import annotations

import socket
import struct
from typing import Iterator, Optional

MAGIC = b"CB"
WIRE_VERSION = 1
HEADER_LEN = 13
MAX_PAYLOAD = 1 << 20

# 0x00xx protocol
PING = 0x0001
PONG = 0x0002

# 0x01xx session core-extension (JSON payloads)
AUTH = 0x0101
AUTH_RSP = 0x0102
LOGOUT = 0x0103
LOGOUT_RSP = 0x0104

# 0x02xx admin control plane (JSON payloads)
ADMIN_REQ = 0x0201
ADMIN_RSP = 0x0202

# 0x10xx CTP passthrough
REQ_USER_LOGIN = 0x1001
RSP_USER_LOGIN = 0x1002
REQ_USER_LOGOUT = 0x1003
RSP_USER_LOGOUT = 0x1004
REQ_SETTLE_CONFIRM = 0x1005
RSP_SETTLE_CONFIRM = 0x1006
REQ_ORDER_INSERT = 0x1010
RSP_ORDER_INSERT = 0x1011
ERR_RTN_ORDER_INSERT = 0x1012
RTN_ORDER = 0x1013
RTN_TRADE = 0x1014
REQ_ORDER_ACTION = 0x1015
RSP_ORDER_ACTION = 0x1016
ERR_RTN_ORDER_ACTION = 0x1017
SUB_MD = 0x1020
RSP_SUB_MD = 0x1021
RTN_DEPTH_MD = 0x1022
UNSUB_MD = 0x1023
RSP_UNSUB_MD = 0x1024
RSP_ERROR = 0x1030

# 0x104x queries; a query stream is terminated by QRY_LAST.
REQ_QRY_INSTRUMENT = 0x1040
RSP_QRY_INSTRUMENT = 0x1041
REQ_QRY_TRADING_ACCOUNT = 0x1042
RSP_QRY_TRADING_ACCOUNT = 0x1043
REQ_QRY_INVESTOR_POSITION = 0x1044
RSP_QRY_INVESTOR_POSITION = 0x1045
REQ_QRY_ORDER = 0x1046
RSP_QRY_ORDER = 0x1047
REQ_QRY_TRADE = 0x1048
RSP_QRY_TRADE = 0x1049
QRY_LAST = 0x1050


class Frame:
    __slots__ = ("msg_type", "req_id", "payload")

    def __init__(self, msg_type: int, req_id: int, payload: bytes = b"") -> None:
        self.msg_type = msg_type
        self.req_id = req_id
        self.payload = payload

    def encode(self) -> bytes:
        head = struct.pack("<2sBHII", MAGIC, WIRE_VERSION, self.msg_type, self.req_id, len(self.payload))
        return head + self.payload

    @classmethod
    def decode_from(cls, sock: socket.socket) -> Optional["Frame"]:
        """Read exactly one frame. Returns None on clean EOF at a boundary."""
        header = _read_exact(sock, HEADER_LEN)
        if header is None:
            return None
        if header[0:2] != MAGIC:
            raise ValueError("bad magic")
        _, version, msg_type, req_id, length = struct.unpack("<2sBHII", header)
        if version != WIRE_VERSION:
            raise ValueError("unsupported wire version %d" % version)
        if length > MAX_PAYLOAD:
            raise ValueError("payload too large: %d" % length)
        payload = _read_exact(sock, length) if length else b""
        if payload is None:
            raise ValueError("truncated payload")
        return cls(msg_type, req_id, payload)

    def __repr__(self) -> str:  # pragma: no cover - debug aid
        return "Frame(0x%04x, req=%d, %d bytes)" % (self.msg_type, self.req_id, len(self.payload))


def _read_exact(sock: socket.socket, n: int) -> Optional[bytes]:
    """Read exactly n bytes. None = clean EOF at a boundary; truncated = error."""
    if n == 0:
        return b""
    buf = bytearray()
    while len(buf) < n:
        chunk = sock.recv(n - len(buf))
        if not chunk:
            if not buf:
                return None
            raise ValueError("truncated frame: got %d of %d bytes" % (len(buf), n))
        buf.extend(chunk)
    return bytes(buf)


def iter_frames(sock: socket.socket) -> Iterator[Frame]:
    while True:
        f = Frame.decode_from(sock)
        if f is None:
            return
        yield f
