"""Shared e2e harness (one copy of the helpers every suite used to carry).

Behavior contract: these are the byte-identical copies that used to live in
each suite; suites whose defaults differ (e.g. drain's quiet window) keep a
local thin wrapper and only import the rest.
"""
import os
import queue
import socket
import time

from ctpbuddy.sdk import Admin, Client
from ctpbuddy.wire import RTN_ORDER, RTN_TRADE

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

# Slow-machine knob: stretches the wall-clock waits that gate on the server's
# one-second flow windows.
E2E_SLOW = os.environ.get("CTPBUDDY_E2E_SLOW", "") not in ("", "0")
TIME_SCALE = 2.0 if E2E_SLOW else 1.0


def find_core() -> str:
    env = os.environ.get("CTPBUDDY_CORE")
    if env:
        return env
    for prof in ("debug", "release"):
        for name in ("ctpbuddy-server", "ctpbuddy-server.exe"):
            cand = os.path.join(REPO, "core", "target", prof, name)
            if os.path.exists(cand):
                return cand
    raise SystemExit("ctpbuddy-server binary not found; run `cargo build` in core/ or set CTPBUDDY_CORE")


def free_port() -> int:
    s = socket.socket()
    s.bind(("127.0.0.1", 0))
    port = s.getsockname()[1]
    s.close()
    return port


def wait_port(port: int, timeout: float = 10.0) -> None:
    deadline = time.time() + timeout
    while time.time() < deadline:
        try:
            socket.create_connection(("127.0.0.1", port), timeout=0.5).close()
            return
        except OSError:
            time.sleep(0.05)
    raise SystemExit("server did not open port %d" % port)


def close(a: float, b: float, tol: float = 1e-6) -> bool:
    return abs(a - b) <= tol


def drain(cli: Client, quiet: float = 0.35):
    """Collect push events until the queue stays empty for `quiet` seconds."""
    out = []
    while True:
        try:
            out.append(next(cli.events(timeout=quiet)))
        except StopIteration:
            break
        except queue.Empty:
            break
    return out


def st(evts, ref: str):
    """OrderStatus chars of every RTN_ORDER for `ref`, in arrival order."""
    return [chr(f["OrderStatus"]) for k, f in evts if k == RTN_ORDER and f["OrderRef"] == ref]


def tr(evts, ref: str):
    """(Volume, Price) of every RTN_TRADE for `ref`, in arrival order."""
    return [(f["Volume"], round(f["Price"], 6)) for k, f in evts if k == RTN_TRADE and f["OrderRef"] == ref]


def tids(evts, ref: str):
    return [f["TradeID"] for k, f in evts if k == RTN_TRADE and f["OrderRef"] == ref]


def rows(evts, ref: str):
    """Every RTN_ORDER dict for `ref`, in arrival order (statuses + volumes)."""
    return [f for k, f in evts if k == RTN_ORDER and f["OrderRef"] == ref]


def wait_idx(admin: Admin, n: int, timeout: float = 5.0) -> None:
    deadline = time.time() + timeout
    while time.time() < deadline:
        if admin.status()["playback"]["idx"] >= n:
            return
        time.sleep(0.02)
    raise AssertionError("playback did not reach tick %d" % n)


class Feed:
    """Per-investor accumulation of push events across the whole flow.

    A drain consumes the queue, so asserting on a single step's slice would
    miss the earlier pushes of the same order; the feed keeps the full
    sequence per investor and the assertions read like the documented
    §8.9 return sequences.
    """

    def __init__(self, clients, quiet: float = 0.35):
        self.clients = clients
        self.log = {name: [] for name in clients}
        self.quiet = quiet

    def pump(self, *names):
        for name in names:
            self.log[name].extend(drain(self.clients[name], self.quiet))

    def st(self, name, ref):
        return st(self.log[name], ref)

    def tr(self, name, ref):
        return tr(self.log[name], ref)

    def tids(self, name, ref):
        return tids(self.log[name], ref)

    def trades(self, name, ref):
        return [f for k, f in self.log[name] if k == RTN_TRADE and f["OrderRef"] == ref]

    def rows(self, name, ref):
        return rows(self.log[name], ref)
