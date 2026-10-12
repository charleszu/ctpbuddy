#!/usr/bin/env python3
"""Native CURVE + ZAP acceptance; requires pyzmq and CURVE-enabled libzmq.

No skip path: CI must install both dependencies. Set CTPBUDDY_TEST_CURVE_SHIM=1
to additionally exercise the built C++ Shim and demo through secure DEALER/SUB.
"""
from __future__ import annotations

import json
import faulthandler
import os
from pathlib import Path
import struct
import subprocess
import sys
import tempfile
import time

import zmq

HERE = Path(__file__).resolve().parent
REPO = HERE.parent.parent
sys.path[:0] = [str(HERE), str(REPO / "py")]
from m1_smoke import find_core, free_port, wait_port, make_scenario, BROKER
from operations import eventually, server
from ctpbuddy.sdk import Admin
from ctpbuddy.wire import Frame, PING, PONG, AUTH, AUTH_RSP


def decode(data):
    magic, version, kind, req, length = struct.unpack("<2sBHII", data[:13])
    assert magic == b"CB" and version == 1 and length == len(data) - 13
    return Frame(kind, req, data[13:])


def connect_budget(env, client_keys, server_key):
    """Both ZMQ adapters disconnect/refuse excess admissions and recover."""
    from zmq.utils.monitor import recv_monitor_message
    for secure in (False, True):
        with tempfile.TemporaryDirectory(prefix="ctpbuddy-zmq-budget-") as directory:
            td = free_port()
            context = zmq.Context()
            peers = []
            try:
                with server(Path(directory), ["--connect-freq", "1", "--td", f"zmq://127.0.0.1:{td}"],
                            env=env if secure else None):
                    def dealer():
                        peer = context.socket(zmq.DEALER)
                        peer.linger = 0
                        peer.sndtimeo = 1000
                        peer.rcvtimeo = 3000
                        if secure:
                            peer.curve_publickey, peer.curve_secretkey = client_keys
                            peer.curve_serverkey = server_key
                        peers.append(peer)
                        return peer
                    first = dealer()
                    first.connect(f"tcp://127.0.0.1:{td}")
                    first.send(Frame(PING, 1).encode())
                    assert first.poll(3000) and decode(first.recv()).req_id == 1
                    denied = dealer()
                    monitor = denied.get_monitor_socket(events=zmq.EVENT_DISCONNECTED | zmq.EVENT_HANDSHAKE_FAILED_AUTH)
                    peers.append(monitor)
                    denied.connect(f"tcp://127.0.0.1:{td}")
                    try:
                        denied.send(Frame(PING, 2).encode())
                    except zmq.Again:
                        pass  # auth can refuse the pipe before send queues
                    assert monitor.poll(1000), "excess ZMQ connection was not refused"
                    assert recv_monitor_message(monitor)["event"] in (zmq.EVENT_DISCONNECTED, zmq.EVENT_HANDSHAKE_FAILED_AUTH)
                    assert not denied.poll(100)
                    denied.close()
                    monitor.close()
                    time.sleep(1.1)
                    recovered = dealer()
                    recovered.connect(f"tcp://127.0.0.1:{td}")
                    recovered.send(Frame(PING, 3).encode())
                    assert recovered.poll(3000) and decode(recovered.recv()).req_id == 3
            finally:
                for peer in peers:
                    peer.close()
                context.destroy(linger=0)
    print("[ok] plaintext/native ZMQ ConnectFreq refusal and recovery")


def main():
    faulthandler.dump_traceback_later(60, repeat=True)
    assert zmq.has("curve"), "pyzmq has no CURVE support"
    server_pub, server_secret = zmq.curve_keypair()
    client_pub, client_secret = zmq.curve_keypair()
    alien_pub, alien_secret = zmq.curve_keypair()
    td, ap, pub = free_port(), free_port(), free_port()
    library = os.environ.get("CTPBUDDY_ZMQ_LIBRARY")
    env = {k: v for k, v in os.environ.items() if not k.startswith("CTPBUDDY_")}
    env.update(CTPBUDDY_ZMQ_CURVE_SECRET_KEY=server_secret.decode(),
               CTPBUDDY_ZMQ_CURVE_CLIENT_KEYS=client_pub.decode(),
               CTPBUDDY_ZMQ_PUB_ADDR=f"tcp://127.0.0.1:{pub}",
               CTPBUDDY_ADMIN_TOKEN="curve-admin")
    if library:
        env["CTPBUDDY_ZMQ_LIBRARY"] = library
    context = zmq.Context()
    sockets = []
    def connect(kind=zmq.DEALER, keys=(client_pub, client_secret), server_key=server_pub):
        s = context.socket(kind)
        s.linger = 0
        s.sndtimeo = 1000
        s.rcvtimeo = 3000
        if keys:
            s.curve_publickey, s.curve_secretkey = keys
            s.curve_serverkey = server_key
        if kind == zmq.SUB:
            s.subscribe(b"sys/")
        s.connect(f"tcp://127.0.0.1:{pub if kind == zmq.SUB else td}")
        sockets.append(s)
        return s
    with tempfile.TemporaryDirectory(prefix="ctpbuddy-curve-") as root:
        root = Path(root)
        for invalid in ({"CTPBUDDY_ZMQ_CURVE_CLIENT_KEYS": ""},
                        {"CTPBUDDY_ZMQ_CURVE_SECRET_KEY": "~" * 40},
                        {"CTPBUDDY_ZMQ_LIBRARY": str(root / "missing-native-library.dll")}):
            result = subprocess.run([find_core(), "--td", "zmq://127.0.0.1:0", "--admin", "127.0.0.1:0",
                                     "--data-dir", str(root / "invalid")], env=dict(env, **invalid),
                                    capture_output=True, timeout=5)
            assert result.returncode != 0, "invalid security configuration accepted"
            assert server_secret not in result.stderr
        with (root / "server.log").open("w", encoding="utf-8") as log:
            proc = subprocess.Popen([find_core(), "--td", f"zmq://127.0.0.1:{td}",
                                     "--admin", f"127.0.0.1:{ap}", "--data-dir", str(root / "data")],
                                    env=env, stdout=log, stderr=log)
            try:
                wait_port(ap)
                with Admin(f"127.0.0.1:{ap}", token="curve-admin") as a:
                    dealer = connect()
                    dealer.send(Frame(PING, 7).encode())
                    assert dealer.poll(3000), "secure ping timeout"
                    f = decode(dealer.recv())
                    assert (f.msg_type, f.req_id) == (PONG, 7)
                    dealer.send(Frame(AUTH, 8, json.dumps({"broker_id": BROKER, "user_id": "curve001", "app_id": "curve-test"}).encode()).encode())
                    assert dealer.poll(3000)
                    assert json.loads(decode(dealer.recv()).payload)["ok"]
                    assert a.status()["security"]["zmq_curve_enabled"]
                    assert a.status()["connections"] == 2
                    # Plaintext, wrong server key and non-allowlisted public key all fail.
                    for denied in (connect(keys=None), connect(server_key=alien_pub), connect(keys=(alien_pub, alien_secret))):
                        try:
                            denied.send(Frame(PING, 9).encode())
                        except zmq.Again:
                            pass
                        assert not denied.poll(600), "unauthorized DEALER accepted"
                    assert a.status()["connections"] == 2
                    sub = connect(zmq.SUB)
                    plain_sub = connect(zmq.SUB, keys=None)
                    alien_sub = connect(zmq.SUB, keys=(alien_pub, alien_secret))
                    def published():
                        a.cmd("announce", message="CURVE test")
                        return sub.poll(100)
                    eventually(published)
                    topic, data = sub.recv_multipart()
                    assert topic == b"sys/announcement" and json.loads(decode(data).payload)["message"] == "CURVE test"
                    assert not plain_sub.poll(300) and not alien_sub.poll(300), "unauthorized SUB received private data"
                    dealer.close()
                    eventually(lambda: a.status()["connections"] == 1)
                    dealer = connect()
                    dealer.send(Frame(PING, 10).encode())
                    assert dealer.poll(3000) and decode(dealer.recv()).req_id == 10
                    dealer.close()
                    eventually(lambda: a.status()["connections"] == 1)
                    print("[ok] secure DEALER/PUB, plaintext/wrong-key/unknown-client refusal, disconnect/reconnect")
                    if os.environ.get("CTPBUDDY_TEST_CURVE_SHIM") == "1":
                        from m1_shim_e2e import find_demo, run_demo
                        find_demo()
                        scenario = root / "scenario"
                        scenario.mkdir()
                        make_scenario(str(scenario))
                        a.start_scenario(str(scenario), paused=True, speed=0)
                        client_env = dict(os.environ)
                        try:
                            os.environ.update(CTPBUDDY_ADDR=f"zmq://127.0.0.1:{td}",
                                CTPBUDDY_ZMQ_CURVE_SERVER_KEY=server_pub.decode(),
                                CTPBUDDY_ZMQ_CURVE_PUBLIC_KEY=client_pub.decode(),
                                CTPBUDDY_ZMQ_CURVE_SECRET_KEY=client_secret.decode(),
                                CTPBUDDY_ZMQ_PUB_ADDR=f"tcp://127.0.0.1:{pub}")
                            output = []
                            assert run_demo(f"zmq://127.0.0.1:{td}", a, output) == 0
                            assert "DEMO: PASS" in output
                            assert sum(line.startswith("[demo] RtnTrade ") for line in output) == 2, "PUB duplicated private SPI callbacks"
                            assert any(line.endswith("10 rtn orders, 2 rtn trades, 3 md ticks") for line in output), "PUB bypassed SPI session scope"
                        finally:
                            os.environ.clear()
                            os.environ.update(client_env)
                        print("[ok] secure C++ Shim full trading session")
                    a.shutdown()
                proc.wait(timeout=5)
                assert proc.returncode == 0
                from ctpbuddy.journal import load_events, verify_events
                events = load_events(str(root / "data" / "journal"))
                assert verify_events(events) == []
                assert any(event["type"] == "system_announcement" for event in events), "PUB notice missing canonical audit"
            finally:
                for s in sockets:
                    s.close()
                context.destroy(linger=0)
                if proc.poll() is None:
                    proc.kill()
                    proc.wait(timeout=5)
                if proc.returncode != 0:
                    print((root / "server.log").read_text(encoding="utf-8"))
    connect_budget(env, (client_pub, client_secret), server_pub)
    print("CURVE E2E: PASS")
    faulthandler.cancel_dump_traceback_later()


if __name__ == "__main__":
    main()
