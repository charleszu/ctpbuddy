#!/usr/bin/env python3
"""Public-boundary acceptance: ADMIN auth, live Web, scheduling and layered flow."""
from __future__ import annotations

import contextlib
import datetime as dt
import http.client
import json
import os
from pathlib import Path
import shutil
import socket
import subprocess
import sys
import tempfile
import threading
import time

HERE = Path(__file__).resolve().parent
REPO = HERE.parent.parent
sys.path[:0] = [str(HERE), str(REPO / "py")]
from m2_flow import make_scenario, RB, M, BROKER, find_core, free_port, wait_port
from ctpbuddy.sdk import Admin, Client, CTPError
from ctpbuddy.web import make_server
from ctpbuddy.wire import Frame, ADMIN_REQ, PING, PONG, ERR_RTN_ORDER_INSERT


def eventually(predicate, timeout=5):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        value = predicate()
        if value:
            return value
        time.sleep(0.03)
    raise AssertionError("condition did not become true")


@contextlib.contextmanager
def server(root, extra=(), env=None, data=None):
    td, ap = free_port(), free_port()
    config = {k: v for k, v in os.environ.items() if not k.startswith("CTPBUDDY_")}
    config.update(env or {})
    data = Path(data or root / "data")
    with (root / "server.log").open("w", encoding="utf-8") as log:
        proc = subprocess.Popen([find_core(), "--td", f"127.0.0.1:{td}", "--admin",
                                 f"127.0.0.1:{ap}", "--data-dir", str(data), *extra],
                                env=config, stdout=log, stderr=log)
        try:
            wait_port(ap)
            yield f"127.0.0.1:{td}", f"127.0.0.1:{ap}", proc
        finally:
            if proc.poll() is None:
                try:
                    with Admin(f"127.0.0.1:{ap}", token=config.get("CTPBUDDY_ADMIN_TOKEN")) as a:
                        a.shutdown()
                    proc.wait(timeout=5)
                except Exception:
                    proc.kill()
                    proc.wait(timeout=5)
            if proc.returncode != 0:
                raise AssertionError((proc.returncode, (root / "server.log").read_text(encoding="utf-8")))


def raw_cmd(sock, text):
    sock.sendall(Frame(ADMIN_REQ, 1, text.encode()).encode())
    return json.loads(Frame.decode_from(sock).payload)


def admin_security(root):
    with server(root, env={"CTPBUDDY_ADMIN_TOKEN": "test-token"}) as (_, addr, _):
        with socket.create_connection(tuple([addr.split(":")[0], int(addr.split(":")[1])])) as s:
            assert not raw_cmd(s, '{"cmd":"shutdown"}')["ok"]
            assert not raw_cmd(s, '{"cmd":"auth","token":"wrong"}')["ok"]
            assert not raw_cmd(s, '{"cmd":"auth","token":"test-token","token":"wrong"}')["ok"]
            assert raw_cmd(s, '{"cmd":"auth","token":"test-token"}')["ok"]
            assert raw_cmd(s, '{"cmd":"status"}')["security"]["admin_token_required"]
        try:
            Admin(addr, token="wrong")
            raise AssertionError("wrong token accepted")
        except RuntimeError:
            pass
        with Admin(addr, token="test-token") as a:
            assert "test-token" not in json.dumps(a.status())
            assert a.ping()["pong"]
    bad_env = {k: v for k, v in os.environ.items() if not k.startswith("CTPBUDDY_")}
    bad_env["CTPBUDDY_ALLOW_REMOTE_ADMIN"] = "1"
    result = subprocess.run([find_core(), "--admin", "0.0.0.0:0", "--data-dir", str(root)],
                            env=bad_env, capture_output=True, timeout=5)
    assert result.returncode != 0 and b"CTPBUDDY_ADMIN_TOKEN" in result.stderr
    bad_env.pop("CTPBUDDY_ALLOW_REMOTE_ADMIN")
    for options, expected in ((["--auto-settle-at", "25:00"], b"auto settle time"),
                              (["--exchange-limits", '{"UNKNOWN":1}'], b"exchange limits"),
                              (["--ftd-max-comm-flux", "-1"], b"invalid --ftd-max-comm-flux")):
        result = subprocess.run([find_core(), "--td", "127.0.0.1:0", "--admin", "127.0.0.1:0",
                                 "--data-dir", str(root), *options], env=bad_env,
                                capture_output=True, timeout=5)
        assert result.returncode != 0 and expected in result.stderr


def ftd_and_connections(root):
    with server(root, ["--ftd-max-comm-flux", "1"]) as (addr, admin, _):
        host, port = addr.split(":")
        with socket.create_connection((host, int(port)), timeout=4) as first, socket.create_connection((host, int(port)), timeout=4) as other:
            start = time.monotonic()
            first.sendall(b"".join(Frame(PING, i).encode() for i in (1, 2, 3)))
            assert Frame.decode_from(first).req_id == 1
            other.sendall(Frame(PING, 4).encode())
            assert Frame.decode_from(other).req_id == 4
            with Admin(admin) as a:
                assert a.ping()["pong"]  # ADMIN is not queued by TD flow
            assert Frame.decode_from(first).req_id == 2
            assert time.monotonic() - start >= 0.85
            assert Frame.decode_from(first).req_id == 3
            assert time.monotonic() - start >= 1.85
            first.sendall(b"".join(Frame(PING, i).encode() for i in range(1000, 2026)))
            for req in (2024, 2025):
                rejected = Frame.decode_from(first)
                assert rejected.req_id == req and CTPError.from_frame(rejected).error_id == 90
    with server(root, ["--connect-freq", "1"]) as (addr, _, _):
        host, port = addr.split(":")
        with socket.create_connection((host, int(port)), timeout=2) as first:
            first.sendall(Frame(PING, 1).encode())
            assert Frame.decode_from(first).msg_type == PONG
            with socket.create_connection((host, int(port)), timeout=2) as denied:
                try:
                    denied.sendall(Frame(PING, 2).encode())
                    assert Frame.decode_from(denied) is None
                except (ConnectionResetError, ConnectionAbortedError, BrokenPipeError):
                    pass
        time.sleep(1.1)
        with socket.create_connection((host, int(port)), timeout=2) as recovered:
            recovered.sendall(Frame(PING, 3).encode())
            assert Frame.decode_from(recovered).req_id == 3


def session(addr, investor, broker=BROKER):
    c = Client(addr)
    c.auth(broker, investor)
    c.login(broker, investor)
    c.settle_confirm()
    return c


def live_and_exchange(root):
    scenario = root / "scenario"
    scenario.mkdir()
    make_scenario(str(scenario))
    with server(root, ["--scenario", str(scenario), "--speed", "0", "--order-freq", "100",
                       "--broker-id", "8888,9999", "--exchange-limits", '{"SHFE":2,"DCE":1}']) as (addr, admin, proc):
        with Admin(admin) as a:
            a.start_scenario(str(scenario), paused=True)
            for _ in range(3):
                a.step()
            clients = [session(addr, "live001"), session(addr, "live002"), session(addr, "live001", "9999")]
            try:
                c = clients[0]
                for ref in ("L1", "L2"):
                    c.order_insert(RB, direction="0", offset="0", volume=1, limit_price=3400, exchange="SHFE", order_ref=ref)
                funds_before = a.live_status(broker=BROKER, investor="live001")["accounts"]
                c.order_insert(RB, direction="0", offset="0", volume=1, limit_price=3400, exchange="SHFE", order_ref="DENIED")
                refused = c.wait_late(ERR_RTN_ORDER_INSERT)
                assert refused is not None and CTPError.from_frame(refused).error_id == 41
                rejected = a.live_status(broker=BROKER, investor="live001")
                assert rejected["accounts"] == funds_before
                assert any(o["order_ref"] == "DENIED" and o["submit_status"] == "4"
                           and "交易所每秒" in o["status_message"] for o in rejected["orders"])
                c.order_insert(RB, direction="0", offset="0", volume=1, limit_price=3400, exchange="", order_ref="")
                refused = c.wait_late(ERR_RTN_ORDER_INSERT)
                assert refused is not None and CTPError.from_frame(refused).error_id == 41
                assert any(o["order_ref"] == "4" and o["submit_status"] == "4" and o["exchange"] == "SHFE"
                           for o in a.live_status(broker=BROKER, investor="live001")["orders"])
                c.order_insert(M, direction="0", offset="0", volume=1, limit_price=2900, exchange="DCE", order_ref="D1")
                clients[1].order_insert(RB, direction="0", offset="0", volume=1, limit_price=3400, exchange="SHFE", order_ref="I2")
                clients[2].order_insert(RB, direction="0", offset="0", volume=1, limit_price=3400, exchange="SHFE", order_ref="B2")
                # SDK omits ExchangeID on cancel: catalog resolution must apply its budget.
                c.order_action(RB, "L1")
                c.order_action(RB, "L2")
                try:
                    c.order_action(RB, "L1")
                    raise AssertionError("cancel budget bypassed")
                except CTPError as e:
                    assert e.error_id == 41
                time.sleep(1.1)
                try:
                    c.order_insert(RB, direction="0", offset="0", volume=1, limit_price=3502, exchange="SHFE", order_ref="DENIED")
                    raise AssertionError("terminal exchange-refused ref was reused")
                except CTPError as e:
                    assert e.error_id == 22
                c.order_insert(RB, direction="0", offset="0", volume=1, limit_price=3502, exchange="SHFE", order_ref="FILL")
                live = eventually(lambda: (v if any(o["order_ref"] == "FILL" and o["status"] == "0" for o in v["orders"]) else None)
                                  if (v := a.live_status(broker=BROKER, investor="live001")) else None)
                assert len([o for o in live["orders"] if o["order_ref"] == "FILL"]) == 1
                assert all(o["broker"] == BROKER and o["investor"] == "live001" for o in live["orders"])
                assert all(o["status"] == "5" for o in live["orders"] if o["order_ref"] in ("L1", "L2"))
                assert live["position_count"] == live["trade_count"] == 1
                assert live["open_orders"] == 1
                assert a.live_status(investor="absent")["accounts"] == []
                assert a.live_status(investor="absent")["open_orders"] == 0
                page = a.live_status(broker=BROKER, investor="live001", limit=1, offset=1)
                assert len(page["orders"]) == 1 and page["order_count"] == live["order_count"]
                web = make_server(port=0, admin=admin, workspace=str(root))
                thread = threading.Thread(target=web.serve_forever, daemon=True)
                thread.start()
                try:
                    def get(path, headers=None):
                        conn = http.client.HTTPConnection("127.0.0.1", web.server_port, timeout=5)
                        try:
                            conn.request("GET", path, headers=headers or {})
                            response = conn.getresponse()
                            return response.status, json.loads(response.read())
                        finally:
                            conn.close()
                    code, view = get("/api/live?broker=8888&investor=live001&limit=1")
                    assert code == 200 and view["realtime"] and view["source_kind"] == "realtime_core"
                    assert view["metadata"]["journal_seq"] == live["journal_seq"]
                    for path in ("/api/live?limit=0", "/api/live?investor=x&investor=y", "/api/live?token=x"):
                        assert get(path)[0] == 400
                    assert get("/api/live", {"Host": "evil.example"})[0] == 403
                    a.shutdown()
                    proc.wait(timeout=5)
                    code, view = get("/api/live")
                    assert code == 503 and view == {"error": "实时核心不可用"}
                finally:
                    web.shutdown()
                    web.server_close()
                    thread.join(timeout=5)
            finally:
                for c in clients:
                    c.close()


def scheduled_expiry(root):
    today = dt.datetime.now(dt.timezone(dt.timedelta(hours=8))).date()
    day, next_day = today.strftime("%Y%m%d"), (today + dt.timedelta(days=1)).strftime("%Y%m%d")
    scenario = root / "scenario"
    scenario.mkdir()
    make_scenario(str(scenario))
    ticks = scenario / "ticks.csv"
    ticks.write_text(ticks.read_text(encoding="utf-8").replace("20261002", day), encoding="utf-8")
    shutil.copytree(REPO / "refdata", scenario / "refdata")
    instruments = scenario / "refdata" / "instruments.jsonl"
    rows = [json.loads(line) for line in instruments.read_text(encoding="utf-8").splitlines() if line]
    for row in rows:
        if row["instrument_id"] == RB:
            row["expire_date"] = day
    instruments.write_text("".join(json.dumps(row) + "\n" for row in rows), encoding="utf-8")
    prices = root / "prices.json"
    args = ["--scenario", str(scenario), "--speed", "0", "--auto-settle-at", "00:00", "--auto-settle-prices", str(prices)]
    with server(root, args) as (td_addr, addr, _):
        with Admin(addr) as a:
            a.start_scenario(str(scenario), paused=True, spec={"accounts": [{"investor": "expiry001", "balance": 1000000, "positions": [{
                "instrument": RB, "exchange": "SHFE", "direction": "long", "open_date": (today - dt.timedelta(days=1)).strftime("%Y%m%d"),
                "trade_id": "BOOT", "open_price": 3500, "volume": 2, "pre_settlement": 3500, "margin": 11200}]}]})
            eventually(lambda: a.status()["auto_settlement"]["last_error"])
            prices.write_text(json.dumps({"trading_day": "20000101", "settlement_prices": {RB: 3501}, "next_trading_day": next_day}), encoding="utf-8")
            eventually(lambda: "日期" in (a.status()["auto_settlement"]["last_error"] or ""))
            assert a.status()["positions"][0]["volume"] == 2
            prices.write_text(json.dumps({"trading_day": day, "settlement_prices": {}, "next_trading_day": next_day}), encoding="utf-8")
            eventually(lambda: "结算价" in (a.status()["auto_settlement"]["last_error"] or ""))
            assert a.status()["positions"][0]["volume"] == 2
            client = session(td_addr, "expiry001")
            a.step()
            client.order_insert(RB, direction="0", offset="0", volume=1, limit_price=3400,
                                exchange="SHFE", order_ref="BLOCK")
            a.resume()
            prices.write_text(json.dumps({"trading_day": day, "settlement_prices": {RB: 3501}, "next_trading_day": next_day}), encoding="utf-8")
            eventually(lambda: "暂停" in (a.status()["auto_settlement"]["last_error"] or ""))
            a.pause()
            eventually(lambda: "活动订单" in (a.status()["auto_settlement"]["last_error"] or ""))
            assert a.status()["positions"][0]["volume"] == 2
            client.order_action(RB, "BLOCK")
            client.close()
            done = eventually(lambda: (v if v["auto_settlement"]["completed_day"] == day else None) if (v := a.status()) else None)
            assert done["positions"] == [] and done["playback"]["trading_day"] == next_day
            assert done["auto_settlement"]["last_error"] is None
            time.sleep(1.1)
            assert a.status()["playback"]["trading_day"] == next_day
    events = [json.loads(line) for file in (root / "data" / "journal").glob("*.jsonl") for line in file.read_text(encoding="utf-8").splitlines() if line]
    assert len([e for e in events if e["type"] == "settlement"]) == 1
    assert len([e for e in events if e["type"] == "fill"]) == 1
    assert any(e["type"] == "order_update" for e in events)
    with server(root, ["--recover", "--auto-settle-at", "00:00", "--auto-settle-prices", str(prices)]) as (_, addr, _):
        with Admin(addr) as a:
            time.sleep(1.1)
            assert a.status()["playback"]["trading_day"] == next_day
            assert a.status()["positions"] == []


def main():
    for name, test in (("admin", admin_security), ("flow", ftd_and_connections), ("live", live_and_exchange), ("scheduler", scheduled_expiry)):
        with tempfile.TemporaryDirectory(prefix="ctpbuddy-ops-" + name) as root:
            test(Path(root))
        print("[ok] " + name)
    print("OPERATIONS E2E: PASS")


if __name__ == "__main__":
    main()
