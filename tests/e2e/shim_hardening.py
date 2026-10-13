"""Shim failure-path hardening e2e (review S1/S2/S3).

Drives the REAL demo binary + shim DLLs against deliberately broken fronts --
the failure paths the architecture review flagged as silent-hang classes:

1. heartbeat: a TCP front that accepts and then goes silent must yield
   OnFrontDisconnected within the (shrunk) pong deadline, never a hang;
2. payload size defense: a login answered with a wrong-size payload must
   surface as an explicit error callback, not a fake "success + NULL" the
   downstream would dereference;
3. zmq send queue: against a ROUTER that never reads, ReqOrderInsert floods
   past the queue cap must answer -2 (vendor "too many pending requests"),
   never silently drop frames (demo mode "flood").

Run: .venv/Scripts/python tests/e2e/shim_hardening.py   (build shim first)
SKIP conventions follow the repo standard: exit 3 when the demo binary (or a
standalone libzmq for the ZMQ case) is unavailable.
"""
import os
import queue
import socket
import subprocess
import sys
import threading
import time

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.join(REPO, "py"))

from ctpbuddy.wire import AUTH, AUTH_RSP, REQ_USER_LOGIN, RSP_USER_LOGIN, Frame  # noqa: E402

IS_WINDOWS = os.name == "nt"
SHIM_BIN = os.environ.get("CTPBUDDY_TEST_SHIM_BIN", os.path.join(REPO, "shim", "bin"))
DEMO_EXE = os.path.join(SHIM_BIN, "demo_td.exe" if IS_WINDOWS else "demo_td")
SKIP = 3


def require_demo() -> None:
    if not os.path.exists(DEMO_EXE):
        hint = "python shim/build_msvc.py --demo" if IS_WINDOWS else "python3 shim/build_linux.py --demo"
        print("SKIP: %s not found; build it first: %s" % (DEMO_EXE, hint))
        raise SystemExit(SKIP)


def run_demo(front, mode=None, env_extra=None):
    env = os.environ.copy()
    env.update(env_extra or {})
    args = [DEMO_EXE, front, "8888", "smoke001"] + ([mode] if mode else [])
    proc = subprocess.Popen(
        args,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        encoding="utf-8",
        errors="replace",
        text=True,
        bufsize=1,
        cwd=SHIM_BIN,
        env=env,
    )
    lines: "queue.Queue[str]" = queue.Queue()

    def pump() -> None:
        assert proc.stdout is not None
        for line in proc.stdout:
            lines.put(line.rstrip())
        lines.put("")  # EOF sentinel

    threading.Thread(target=pump, daemon=True).start()
    return proc, lines


def wait_for(lines, needle, timeout):
    """Collect demo output until a line contains `needle`; returns the seen
    lines (raising on EOF/timeout)."""
    deadline = time.monotonic() + timeout
    seen = []
    while time.monotonic() < deadline:
        try:
            line = lines.get(timeout=0.5)
        except queue.Empty:
            continue
        if line == "":
            break
        seen.append(line)
        print("  demo| " + line)
        if needle in line:
            return seen
    raise AssertionError("demo never printed %r; saw: %r" % (needle, seen))


def drain(lines):
    out = []
    while True:
        try:
            line = lines.get(timeout=0.3)
        except queue.Empty:
            break
        if line == "":
            break
        out.append(line)
        print("  demo| " + line)
    return out


def stop(proc) -> None:
    if proc.poll() is None:
        proc.terminate()
        try:
            proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait(timeout=10)


def test_heartbeat() -> None:
    """S1: a front that accepts and goes silent is declared dead within the
    pong deadline -- OnFrontDisconnected reaches the app, nothing hangs."""
    listener = socket.socket()
    listener.bind(("127.0.0.1", 0))
    listener.listen()

    def silent_front() -> None:
        peer, _ = listener.accept()  # accept once, then neither read nor write
        try:
            time.sleep(120)
        finally:
            peer.close()

    threading.Thread(target=silent_front, daemon=True).start()
    proc, lines = run_demo(
        "tcp://127.0.0.1:%d" % listener.getsockname()[1],
        env_extra={
            "CTPBUDDY_SHIM_PING_IDLE_SEC": "1",
            "CTPBUDDY_SHIM_PONG_DEADLINE_SEC": "3",
        },
    )
    try:
        t0 = time.monotonic()
        seen = wait_for(lines, "front disconnected (td)", timeout=30)
        elapsed = time.monotonic() - t0
        assert elapsed < 20, "disconnect took %.1fs -- deadline not enforced" % elapsed
        # the TCP peer is still perfectly alive: only the heartbeat declared
        # the link dead, which is exactly the half-open-connection case
        print("[ok] heartbeat: silent front -> OnFrontDisconnected in %.1fs" % elapsed)
        assert not any("DEMO: PASS" in l for l in seen)
    finally:
        stop(proc)
        listener.close()


def test_payload_size_mismatch() -> None:
    """S3: a login answered with a wrong-size payload completes as an explicit
    error callback carrying the mismatch message -- never a success+NULL."""
    listener = socket.socket()
    listener.bind(("127.0.0.1", 0))
    listener.listen()

    def lying_front() -> None:
        while True:
            try:
                peer, _ = listener.accept()
            except OSError:
                return
            with peer:
                while True:
                    try:
                        f = Frame.decode_from(peer)
                    except OSError:
                        break
                    if f is None:
                        break
                    if f.msg_type == AUTH:
                        body = b'{"ok":true,"broker_id":"8888","user_id":"smoke001"}'
                        peer.sendall(Frame(AUTH_RSP, f.req_id, body).encode())
                    elif f.msg_type == REQ_USER_LOGIN:
                        # wrong on purpose: 3 bytes where the login struct goes
                        peer.sendall(Frame(RSP_USER_LOGIN, f.req_id, b"abc").encode())

    threading.Thread(target=lying_front, daemon=True).start()
    proc, lines = run_demo("tcp://127.0.0.1:%d" % listener.getsockname()[1])
    try:
        seen = wait_for(lines, "payload size mismatch", timeout=30)
        # the mismatch must look like a rejected login, never a pass
        assert not any("DEMO: PASS" in l for l in seen)
        print("[ok] payload size defense: wrong-size login -> explicit error callback")
    finally:
        stop(proc)
        listener.close()


def test_zmq_queue_full() -> None:
    """S2: a ROUTER that never reads; the demo's flood mode issues far more
    requests than the (shrunk) outbound queue + DEALER HWM can hold. Every
    request beyond the cap must return -2, and none may be dropped silently."""
    try:
        import zmq  # noqa: F401
    except ImportError:
        print("SKIP: pyzmq unavailable (uv sync --group e2e)")
        raise SystemExit(SKIP)
    libzmq = os.environ.get("CTPBUDDY_ZMQ_LIBRARY", "")
    if not libzmq:
        for name in ("libzmq.dll", "libzmq.so.5", "libzmq.so"):
            candidate = os.path.join(SHIM_BIN, name)
            if os.path.exists(candidate):
                libzmq = candidate
                break
    if not libzmq:
        # pyzmq bundles libzmq statically -- it cannot be borrowed by the
        # shim's runtime dlopen. A standalone library is required (CI's
        # CURVE jobs build one; locally point CTPBUDDY_ZMQ_LIBRARY at it).
        print("SKIP: no standalone libzmq for the shim (set CTPBUDDY_ZMQ_LIBRARY)")
        raise SystemExit(SKIP)

    import zmq

    ctx = zmq.Context()
    router = ctx.socket(zmq.ROUTER)
    port = router.bind_to_random_port("tcp://127.0.0.1")  # bound, never read
    proc, lines = run_demo(
        "zmq://tcp://127.0.0.1:%d" % port,
        mode="flood",
        env_extra={
            "CTPBUDDY_ZMQ_LIBRARY": libzmq,
            "CTPBUDDY_SHIM_ZMQ_SEND_QUEUE": "2",
            "CTPBUDDY_SHIM_PING_IDLE_SEC": "0",  # this probe is about the queue only
        },
    )
    try:
        rc = proc.wait(timeout=120)
        out = drain(lines)
        assert rc == 0, "flood demo exited %d: %r" % (rc, out)
        refused = [l for l in out if "flood refused=" in l]
        assert refused, "flood summary missing: %r" % out
        count = int(refused[0].split("refused=")[1].split()[0])
        assert count > 0, "queue cap never answered -2 (silent frame-drop regression?)"
        print("[ok] zmq send queue: full queue refuses new requests with -2 (%d refused)" % count)
    finally:
        stop(proc)
        router.close(0)
        ctx.term()


def main() -> int:
    require_demo()
    test_heartbeat()
    test_payload_size_mismatch()
    try:
        test_zmq_queue_full()
    except SystemExit as exc:
        if exc.code == SKIP:
            # the first two probes passed; the ZMQ one needs a standalone libzmq
            print("\nSHIM HARDENING: PASS (zmq queue probe SKIPPED)")
            return SKIP
        raise
    print("\nSHIM HARDENING: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
