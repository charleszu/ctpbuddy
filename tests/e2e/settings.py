#!/usr/bin/env python
"""柜台设置 ADMIN/TCP/HTTP 回归：真实生效、持久化、严格拒绝与安全来源。"""
from __future__ import annotations
import json, os, socket, subprocess, sys, tempfile, threading, time, urllib.error, urllib.request
sys.path.insert(0, os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))), "py"))
from ctpbuddy.sdk import Admin, Client, CTPError
from ctpbuddy.wire import REQ_QRY_INSTRUMENT
from ctpbuddy.generated import structs
from ctpbuddy.web import make_server

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
CORE = os.environ.get("CTPBUDDY_CORE", os.path.join(ROOT, "core", "target", "debug", "ctpbuddy-server.exe"))

def port():
    s=socket.socket(); s.bind(("127.0.0.1",0)); p=s.getsockname()[1]; s.close(); return p

def wait_admin(addr):
    end=time.time()+10
    while time.time()<end:
        try:
            with Admin(addr) as a: return a.settings()
        except OSError: time.sleep(.05)
    raise AssertionError("ADMIN 未启动")

def main():
    data=tempfile.mkdtemp(prefix="ctpbuddy-settings-e2e-"); td, ap=port(), port()
    cmd=[CORE,"--td",f"127.0.0.1:{td}","--admin",f"127.0.0.1:{ap}","--data-dir",data]
    proc=subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    try:
        addr=f"127.0.0.1:{ap}"; initial=wait_admin(addr)
        assert initial["settings"]["max_user_sessions"] == 0
        assert initial["settings"]["self_trade_prevention"] is False  # 默认关闭（真实交易所不阻止自成交）
        with Admin(addr) as a:
            got=a.update_settings({"qry_freq":3,"order_freq":4,"max_user_sessions":2,"settlement_required":False,"self_trade_prevention":True,"initial_funds":123456.5})
            assert got["settings"]["order_freq"] == 4 and got["settings"]["initial_funds"] == 123456.5
            assert got["settings"]["self_trade_prevention"] is True
            for bad in ({"unknown":1},{"qry_freq":0},{"qry_freq":1.2},{"initial_funds":float("nan")},{"self_trade_prevention":1}):
                try: a.update_settings(bad); raise AssertionError("非法设置被接受: %r" % (bad,))
                except RuntimeError: pass
            assert a.settings()["settings"]["qry_freq"] == 3
        # session cap is a live CTP behavior, not merely a reported value
        c1, c2, c3 = (Client(f"127.0.0.1:{td}") for _ in range(3))
        try:
            for c in (c1, c2): c.auth("8888", "live-session-user"); c.login("8888", "live-session-user")
            c3.auth("8888", "live-session-user")
            try: c3.login("8888", "live-session-user"); raise AssertionError("会话上限未生效")
            except CTPError as e: assert e.error_id == 60, e
            c1.close(); c2.close(); c3.close()
        finally:
            for c in (c1, c2, c3):
                try: c.close()
                except OSError: pass
        with Admin(addr) as a: a.update_settings({"settlement_required":True})
        fresh=Client(f"127.0.0.1:{td}")
        try:
            fresh.auth("8888", "freshfunds"); fresh.login("8888", "freshfunds")
            assert abs(fresh.qry_trading_account()["Balance"]-123456.5) < 1e-6
            try: fresh.order_insert("rb2601", "0", "0", 1, 3500.0); raise AssertionError("结算门禁未生效")
            except CTPError as e: assert e.error_id == 42, e
            fresh.close()
        finally: fresh.close()
        # HTTP settings page and POST use the real ADMIN; save is verified by a second ADMIN read.
        web=make_server("127.0.0.1",0,addr,workspace=data); t=threading.Thread(target=web.serve_forever,daemon=True); t.start(); wp=web.server_address[1]
        base=f"http://127.0.0.1:{wp}"; host=f"127.0.0.1:{wp}"
        good=urllib.request.Request(base+"/",headers={"Host":host})
        assert urllib.request.urlopen(good).status == 200
        sess=json.loads(urllib.request.urlopen(urllib.request.Request(base+"/api/session",headers={"Host":host})).read())
        payload=json.dumps({"patch":{"qry_freq":5}}).encode()
        post=urllib.request.Request(base+"/api/settings",data=payload,method="POST",headers={"Host":host,"Origin":base,"Content-Type":"application/json","Content-Length":str(len(payload)),"X-CSRF-Token":sess["token"]})
        saved=json.loads(urllib.request.urlopen(post).read()); assert saved["settings"]["qry_freq"] == 5 and "audit" in saved
        with Admin(addr) as a: assert a.settings()["settings"]["qry_freq"] == 5
        bad_csrf=urllib.request.Request(base+"/api/settings",data=payload,method="POST",headers={"Host":host,"Origin":base,"Content-Type":"application/json","Content-Length":str(len(payload)),"X-CSRF-Token":"wrong"})
        try: urllib.request.urlopen(bad_csrf); raise AssertionError("错误 CSRF 被接受")
        except urllib.error.HTTPError as e: assert e.code == 403
        bad_origin=urllib.request.Request(base+"/api/settings",data=payload,method="POST",headers={"Host":host,"Origin":"http://evil.invalid","Content-Type":"application/json","Content-Length":str(len(payload)),"X-CSRF-Token":sess["token"]})
        try: urllib.request.urlopen(bad_origin); raise AssertionError("错误 Origin 被接受")
        except urllib.error.HTTPError as e: assert e.code == 403
        unknown=json.dumps({"patch":{"unknown":1}}).encode()
        bad_schema=urllib.request.Request(base+"/api/settings",data=unknown,method="POST",headers={"Host":host,"Origin":base,"Content-Type":"application/json","Content-Length":str(len(unknown)),"X-CSRF-Token":sess["token"]})
        try: urllib.request.urlopen(bad_schema); raise AssertionError("未知 schema 键被接受")
        except urllib.error.HTTPError as e: assert e.code == 400
        # 独立临时 core 的真实回放：HTTP 命令必须推进核心 tick，而非浏览器模拟。
        def replay(body=None):
            headers={"Host":host}
            if body is not None:
                raw=json.dumps(body).encode()
                headers.update({"Origin":base,"Content-Type":"application/json","X-CSRF-Token":sess["token"]})
            else: raw=None
            req=urllib.request.Request(base+("/api/replay" if body is not None else "/api/replay/status"),data=raw,headers=headers)
            return json.loads(urllib.request.urlopen(req,timeout=5).read())
        assert replay()["playback"]["loaded"] is False
        try: replay({"cmd":"step"}); raise AssertionError("无场景应拒绝")
        except urllib.error.HTTPError as e: assert e.code == 409
        from ctpbuddy.sources import CANONICAL_COLUMNS, write_canonical
        scenario=os.path.join(data,"scenarios","web-scenario")
        rows=[]
        for minute in range(3):
            row={column:"" for column in CANONICAL_COLUMNS}
            row.update(instrument="rb2601",exchange="SHFE",trading_day="20261002",update_time=f"09:{30+minute}:00",update_millisec="0",last_price=str(3500+minute),volume=str(minute+1),upper="3850",lower="3150",bid1="3498",ask1="3502",bidvol1="5",askvol1="5")
            rows.append(row)
        write_canonical(scenario,rows)
        def web_request(path, body=None):
            headers={"Host":host}
            raw=None
            if body is not None:
                raw=json.dumps(body).encode()
                headers.update({"Origin":base,"Content-Type":"application/json","X-CSRF-Token":sess["token"]})
            return json.loads(urllib.request.urlopen(urllib.request.Request(base+path,data=raw,headers=headers),timeout=5).read())
        catalog=web_request("/api/scenarios")
        assert catalog["scenarios"][0]["path"] == "scenarios/web-scenario"
        details=web_request("/api/scenario?path=scenarios/web-scenario")
        assert details["readonly"] and details["positions_state"] == "scenario_initial_only_not_live"
        for bad in ({"path":"../web-scenario","confirmed":True}, {"path":scenario,"confirmed":True},
                    {"path":"scenarios/web-scenario","confirmed":False},
                    {"path":"scenarios/web-scenario","confirmed":True,"cmd":"shutdown"}):
            try: web_request("/api/scenario/load",bad); raise AssertionError("非法加载被接受")
            except urllib.error.HTTPError as e: assert e.code == 400
        loaded=web_request("/api/scenario/load",{"path":"scenarios/web-scenario","confirmed":True})
        assert loaded["ok"] and loaded["paused"]
        with Admin(addr) as a: a.set_speed(.01)
        state=replay(); assert state["playback"]["idx"] == 0 and state["realtime"] is True
        with Admin(addr) as a:
            for bad in ({}, {"speed":True}, {"speed":"2"}, {"speed":-1}, {"speed":1001}, {"speed":1e309}, {"speed":2,"extra":1}):
                try: a.cmd("set_speed",**bad); raise AssertionError("核心非法速度被接受")
                except RuntimeError: pass
            assert a.status()["playback"]["speed"] == .01
        def wait_tick(idx):
            end=time.monotonic()+3
            while time.monotonic()<end:
                state=replay()
                if state["playback"]["idx"] == idx: return state
                time.sleep(.02)
            raise AssertionError("HTTP 回放没有推进 tick %d" % idx)
        replay({"cmd":"pause"}); replay({"cmd":"step"})
        stepped=wait_tick(1)
        assert stepped["playback"]["paused"] and stepped["playback"]["virtual_time"].startswith("09:30:00")
        with Admin(addr) as a: assert a.status()["playback"]["idx"] == 1
        time.sleep(.06); assert replay()["playback"]["idx"] == 1
        assert replay({"cmd":"set_speed","speed":.02})["playback"]["speed"] == .02
        assert replay({"cmd":"loop","on":True})["playback"]["looping"] is True
        replay({"cmd":"loop","on":False})
        assert replay({"cmd":"resume"})["playback"]["paused"] is False
        assert replay({"cmd":"pause"})["playback"]["paused"] is True
        replay({"cmd":"step"}); stepped=wait_tick(2)
        assert stepped["playback"]["virtual_time"].startswith("09:31:00")
        replay({"cmd":"set_speed","speed":0}); replay({"cmd":"resume"})
        finished=wait_tick(3); assert finished["playback"]["finished"]
        try: replay({"cmd":"step"}); raise AssertionError("已完成应拒绝")
        except urllib.error.HTTPError as e: assert e.code == 409
        print("[ok] WEB REPLAY: 实时 ADMIN status/pause/resume/step/speed/loop 与 tick/虚拟时间/完成边界")
        with Admin(addr) as a: a.shutdown()
        web.shutdown(); web.server_close()
        proc.wait(timeout=5)
        proc=subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
        restored=wait_admin(addr); assert restored["settings"]["initial_funds"] == 123456.5
        with Admin(addr) as a: a.shutdown()
        proc.wait(timeout=5)
        web=make_server("127.0.0.1",0,addr,workspace=data); t=threading.Thread(target=web.serve_forever,daemon=True); t.start(); wp=web.server_address[1]
        good=urllib.request.Request(f"http://127.0.0.1:{wp}/",headers={"Host":f"127.0.0.1:{wp}"})
        assert urllib.request.urlopen(good).status == 200
        evil=urllib.request.Request(f"http://127.0.0.1:{wp}/api/settings",headers={"Host":f"evil:{wp}"})
        try: urllib.request.urlopen(evil); raise AssertionError("DNS rebinding Host 被接受")
        except urllib.error.HTTPError as e: assert e.code == 403
        web.shutdown(); web.server_close()
        print("SETTINGS E2E: PASS")
        return 0
    finally:
        if proc.poll() is None: proc.kill(); proc.wait()

if __name__ == "__main__": raise SystemExit(main())
