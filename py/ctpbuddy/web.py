"""仅回环可用的柜台设置 Web；不提供任意 ADMIN 命令代理。"""
from __future__ import annotations

import ipaddress
import json
import math
import secrets
import socket
import sqlite3
import threading
from urllib.parse import parse_qs, urlsplit
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

from .scenario import ScenarioError, load_scenario_spec
from .sdk import Admin

MAX_BODY = 8192
MAX_SCENARIO_NAME = 160
PLAYBACK_COMMANDS = {"pause", "resume", "step", "set_speed", "loop"}
WEB_PROJECTION_TABLES = {"account", "position_snapshot", "account_snapshot", "order_record", "trade_record", "audit_log", "settlement_report"}
MAX_PLAYBACK_SPEED = 1000.0
MAX_SETTLEMENT_REPORTS = 16
MAX_SETTLEMENT_CONTENT = 512 * 1024


def _strict_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError("重复键: " + key)
        result[key] = value
    return result


def _workspace_path(workspace, relative):
    if (not isinstance(relative, str) or not relative or len(relative) > MAX_SCENARIO_NAME
            or any(c in relative for c in ("\\", ":", "\x00"))
            or relative.startswith("/") or any(p in ("", ".", "..") for p in relative.split("/"))):
        raise ValueError("仅允许无穿越的 workspace 相对路径")
    path = workspace.joinpath(relative)
    if not path.resolve().is_relative_to(workspace):
        raise ValueError("场景路径超出 workspace")
    return path


def _scenario_view(workspace, relative):
    path = _workspace_path(workspace, relative)
    if not relative.startswith("scenarios/") or len(relative.split("/")) != 2 or not path.is_dir():
        raise ValueError("场景必须位于 workspace/scenarios 的直接子目录")
    for name in ("scenario.json", "scenario.yaml", "ticks.csv"):
        candidate = path / name
        if candidate.exists() and not candidate.resolve().is_relative_to(workspace):
            raise ValueError("场景文件超出 workspace")
    spec = load_scenario_spec(str(path))
    source = spec.get("source", {"kind": "csv", "path": "ticks.csv"})
    if not isinstance(source, dict) or source.get("kind", "csv") != "csv":
        raise ValueError("Web 仅允许 CSV 场景源")
    source_path = source.get("path", "ticks.csv")
    if isinstance(source_path, str) and source_path.startswith("./"):
        source_path = source_path[2:]
    ticks = _workspace_path(workspace, relative + "/" + source_path) if isinstance(source_path, str) else None
    if ticks is None or not ticks.is_file():
        raise ValueError("场景 CSV 不可用")
    refdata = path / "refdata"
    if refdata.exists():
        if not refdata.resolve().is_relative_to(workspace):
            raise ValueError("RefData 超出 workspace")
        for item in refdata.rglob("*"):
            if not item.resolve().is_relative_to(workspace):
                raise ValueError("RefData 文件超出 workspace")
    json.dumps(spec, allow_nan=False)
    return path, spec, {"path": relative, "name": spec.get("name") or path.name,
                        "accounts": spec.get("accounts", []), "readonly": True,
                        "positions_state": "scenario_initial_only_not_live"}


def make_server(host="127.0.0.1", port=8080, admin="127.0.0.1:5561", database=None, workspace=None):
    if host == "loopback":
        host = "127.0.0.1"
    if host == "localhost":
        host = "127.0.0.1"
    if not ipaddress.ip_address(host).is_loopback:
        raise ValueError("Web 后台只允许回环地址")
    ahost, _, aport = admin.rpartition(":")
    addresses = socket.getaddrinfo(ahost.strip("[]"), int(aport), type=socket.SOCK_STREAM)
    if not addresses or any(not ipaddress.ip_address(a[4][0]).is_loopback for a in addresses):
        raise ValueError("ADMIN 必须是回环地址")
    workspace = Path(workspace or Path.cwd()).resolve()
    if not workspace.is_dir():
        raise ValueError("workspace 必须是已有目录")
    token = secrets.token_urlsafe(32)
    slots = threading.BoundedSemaphore(8)
    replay_write = threading.Lock()

    class Handler(BaseHTTPRequestHandler):
        def log_message(self, *args):
            pass

        def reply(self, code, data, kind="application/json; charset=utf-8"):
            if not isinstance(data, bytes):
                data = json.dumps(data, ensure_ascii=False, allow_nan=False).encode()
            self.send_response(code)
            self.send_header("Content-Type", kind)
            self.send_header("Content-Length", str(len(data)))
            self.send_header("Cache-Control", "no-store")
            self.send_header("X-Content-Type-Options", "nosniff")
            self.send_header("Content-Security-Policy", "default-src 'self'; script-src 'self'; style-src 'self'; connect-src 'self'; frame-ancestors 'none'; base-uri 'none'; form-action 'self'")
            self.end_headers()
            self.wfile.write(data)

        def valid_source(self, write=False):
            port = self.server.server_address[1]
            bind = self.server.server_address[0]
            authorities = {f"127.0.0.1:{port}", f"localhost:{port}"} if bind == "127.0.0.1" else {f"[{bind}]:{port}"}
            hosts = self.headers.get_all("Host", [])
            origin = self.headers.get("Origin")
            # Browsers and local preview proxies may resolve the same loopback
            # server through either localhost or 127.0.0.1. Treat those two
            # authorities as equivalent, but never accept a non-loopback host.
            if any(len(self.headers.get_all(name, [])) > 1 for name in ("Origin", "X-CSRF-Token", "Sec-Fetch-Site")):
                return False
            if len(hosts) != 1 or hosts[0] not in authorities:
                return False
            expected_origins = {"http://" + authority for authority in authorities}
            if origin is not None and origin not in expected_origins:
                return False
            if self.headers.get("Sec-Fetch-Site") not in (None, "same-origin", "none"):
                return False
            if write and (origin not in expected_origins or not secrets.compare_digest(self.headers.get("X-CSRF-Token", ""), token)):
                return False
            return True

        def do_GET(self):
            if not self.valid_source():
                return self.reply(403, {"error": "Host/Origin 校验失败"})
            assets = {"/": "settings.html", "/settings.js": "settings.js", "/settings.css": "settings.css"}
            if self.path in assets:
                name = assets[self.path]
                kind = "text/html" if name.endswith("html") else ("text/javascript" if name.endswith("js") else "text/css")
                return self.reply(200, (Path(__file__).parent / "assets" / name).read_bytes(), kind + "; charset=utf-8")
            if self.path == "/api/session":
                return self.reply(200, {"token": token})
            path = urlsplit(self.path).path
            if path == "/api/projection":
                return self.projection()
            if path == "/api/replay/status":
                return self.playback_status()
            if path == "/api/scenarios":
                return self.scenarios()
            if path == "/api/scenario":
                return self.scenario_details(query=parse_qs(urlsplit(self.path).query, keep_blank_values=True))
            if self.path != "/api/settings":
                return self.reply(404, {"error": "路径不存在"})
            self.invoke()

        def scenarios(self):
            if self.path != "/api/scenarios":
                return self.reply(400, {"error": "场景目录不接受查询参数"})
            rows = []
            root = workspace / "scenarios"
            if root.is_dir():
                for path in sorted(root.iterdir()):
                    if not path.is_dir() or path.is_symlink():
                        continue
                    relative = "scenarios/" + path.name
                    try:
                        _, _, view = _scenario_view(workspace, relative)
                    except (OSError, ScenarioError, ValueError, TypeError, json.JSONDecodeError):
                        continue
                    rows.append(view)
            return self.reply(200, {"workspace": ".", "scenarios": rows, "readonly": True})

        def scenario_details(self, query):
            if set(query) != {"path"} or len(query.get("path", [])) != 1:
                return self.reply(400, {"error": "场景查询必须提供单一 path"})
            try:
                _, _, view = _scenario_view(workspace, query["path"][0])
            except (OSError, ScenarioError, ValueError, TypeError, json.JSONDecodeError):
                return self.reply(400, {"error": "场景文件或路径无效"})
            return self.reply(200, view)

        def projection(self):
            if not database:
                return self.reply(404, {"error": "未配置 SQLite 投影：先执行 ctpbuddy journal rebuild JOURNAL --db DB，再使用 ctpbuddy web --db DB 启动"})
            query = parse_qs(urlsplit(self.path).query, keep_blank_values=True)
            allowed = {"table", "broker", "investor", "trading_day", "limit", "offset"}
            if set(query) - allowed or any(len(values) != 1 for values in query.values()):
                return self.reply(400, {"error": "投影查询参数无效"})
            table = query.get("table", ["account"])[0]
            if table not in WEB_PROJECTION_TABLES:
                return self.reply(400, {"error": "投影表不在 Web 白名单中"})
            def one(name):
                return query.get(name, [None])[0]
            try:
                from .journal import JournalError
                from .store import Projection
                with Projection(database) as projection:
                    rows = projection.query(table, broker=one("broker"), investor=one("investor"),
                                            trading_day=one("trading_day"), limit=int(one("limit") or 100),
                                            offset=int(one("offset") or 0))
                    snapshot = {row[0]: row[1] for row in projection.conn.execute("SELECT key,value FROM projection_meta")}
            except ValueError:
                return self.reply(400, {"error": "投影查询参数无效"})
            except (OSError, sqlite3.Error, RuntimeError, JournalError):
                # 不把 SQLite 的绝对路径或底层文件错误回显给浏览器。
                return self.reply(503, {"error": "SQLite 投影不可用"})
            return self.reply(200, {"table": table, "rows": rows, "snapshot": snapshot, "realtime": False})

        def do_POST(self):
            if not self.valid_source(write=True):
                return self.reply(403, {"error": "同源或 CSRF 校验失败"})
            route = urlsplit(self.path).path
            if self.path != route or route not in ("/api/settings", "/api/replay", "/api/scenario/load", "/api/admin/settlement_report", "/api/admin/settle_day"):
                return self.reply(404, {"error": "路径不存在"})
            if self.headers.get("Transfer-Encoding") or len(self.headers.get_all("Content-Length", [])) != 1:
                return self.reply(400, {"error": "需要单一 Content-Length"})
            if self.headers.get("Content-Type", "").split(";")[0] != "application/json":
                return self.reply(415, {"error": "仅支持 application/json"})
            try:
                n = int(self.headers["Content-Length"])
                if not 0 < n <= MAX_BODY:
                    return self.reply(413, {"error": "请求体超过限制"})
                self.connection.settimeout(3)
                body = self.rfile.read(n)
                if len(body) != n:
                    raise ValueError("请求体不完整")
                def invalid_constant(s):
                    raise ValueError("非有限数字: " + s)
                value = json.loads(body, object_pairs_hook=_strict_object, parse_constant=invalid_constant)
                json.dumps(value, allow_nan=False)
                if route == "/api/replay":
                    self.validate_playback(value)
                elif route == "/api/scenario/load":
                    self.validate_scenario_load(value)
                elif route == "/api/settings":
                    if not isinstance(value, dict) or set(value) != {"patch"} or not isinstance(value["patch"], dict):
                        raise ValueError("仅允许 patch 对象")
                elif self.path == "/api/admin/settlement_report":
                    self.validate_settlement_report(value)
                else:
                    self.validate_settle_day(value)
            except (ValueError, UnicodeError, OSError, ScenarioError, TypeError) as e:
                return self.reply(400, {"error": "场景文件或路径无效" if route == "/api/scenario/load" else str(e)})
            if route == "/api/replay":
                self.invoke(playback=value)
            elif route == "/api/scenario/load":
                self.invoke(scenario=value)
            elif route == "/api/settings":
                self.invoke(value["patch"])
            elif self.path == "/api/admin/settlement_report":
                reports = []
                for report in value["reports"]:
                    item = dict(report)
                    content = item.pop("content")
                    item["content_bytes"] = list(content.encode("gbk", "strict"))
                    reports.append(item)
                self.invoke(admin_cmd="settlement_report", admin_args={"reports": reports})
            else:
                self.invoke(admin_cmd="settle_day", admin_args={"settlement_prices": value["settlement_prices"], "next_trading_day": value["next_trading_day"]})

        def validate_settlement_report(self, value):
            if not isinstance(value, dict) or set(value) != {"confirmed", "reports"} or value["confirmed"] is not True:
                raise ValueError("结算报告写入必须明确 confirmed=true")
            reports = value["reports"]
            if not isinstance(reports, list) or not 0 < len(reports) <= MAX_SETTLEMENT_REPORTS:
                raise ValueError("reports 数量无效")
            for report in reports:
                if not isinstance(report, dict) or not {"broker", "investor", "trading_day", "settlement_id", "content"}.issubset(report):
                    raise ValueError("结算报告字段无效")
                if set(report) - {"broker", "investor", "trading_day", "settlement_id", "account_id", "currency_id", "content", "source"}:
                    raise ValueError("结算报告含未知字段")
                if not all(isinstance(report.get(k), str) and report[k] for k in ("broker", "investor", "trading_day")):
                    raise ValueError("结算报告身份字段无效")
                if not isinstance(report["settlement_id"], int) or isinstance(report["settlement_id"], bool) or report["settlement_id"] < 1:
                    raise ValueError("settlement_id 无效")
                content = report["content"]
                if not isinstance(content, str) or not content or len(content.encode("gbk", "strict")) > MAX_SETTLEMENT_CONTENT:
                    raise ValueError("结算报告正文无效")

        def validate_settle_day(self, value):
            if not isinstance(value, dict) or set(value) != {"confirmed", "settlement_prices", "next_trading_day"} or value["confirmed"] is not True:
                raise ValueError("日结写入必须明确 confirmed=true")
            prices, day = value["settlement_prices"], value["next_trading_day"]
            if not isinstance(prices, dict) or not prices or len(prices) > 1000 or not isinstance(day, str) or len(day) != 8 or not day.isdigit():
                raise ValueError("日结参数无效")
            for instrument, price in prices.items():
                if not isinstance(instrument, str) or not instrument or len(instrument) > 31 or not isinstance(price, (int, float)) or isinstance(price, bool) or not math.isfinite(price) or price <= 0:
                    raise ValueError("日结结算价无效")

        def validate_scenario_load(self, value):
            if not isinstance(value, dict) or set(value) != {"path", "confirmed"} or value["confirmed"] is not True:
                raise ValueError("场景加载必须明确 confirmed=true，且只提供 path")
            _scenario_view(workspace, value["path"])

        def validate_playback(self, value):
            if not isinstance(value, dict) or not isinstance(value.get("cmd"), str) or value["cmd"] not in PLAYBACK_COMMANDS:
                raise ValueError("未知回放命令")
            cmd = value["cmd"]
            fields = {"cmd", "speed"} if cmd == "set_speed" else ({"cmd", "on"} if cmd == "loop" else {"cmd"})
            if set(value) != fields:
                raise ValueError("回放命令字段无效")
            if cmd == "set_speed":
                speed = value["speed"]
                if type(speed) not in (int, float) or not 0 <= speed <= MAX_PLAYBACK_SPEED or not math.isfinite(speed):
                    raise ValueError("speed 必须为 0–1000 的有限数字（0 为不限速）")
            if cmd == "loop" and type(value["on"]) is not bool:
                raise ValueError("on 必须为布尔值")

        def playback_status(self):
            if self.path != "/api/replay/status":
                return self.reply(400, {"error": "回放状态不接受查询参数"})
            self.invoke(playback={"cmd": "status"})

        def replay_view(self, client):
            status = client.status()
            pb = status["playback"]
            return {"ok": True, "scenario": status["scenario"], "playback": dict(pb, finished=pb["loaded"] and pb["idx"] >= pb["total"]),
                    "accounts": status.get("accounts", []), "broker_id": status.get("broker_id"),
                    "open_orders": status.get("open_orders"), "positions_state": "admin_live_positions_unavailable",
                    "realtime": True, "speed_range": {"min": 0, "max": MAX_PLAYBACK_SPEED}}

        def invoke(self, patch=None, playback=None, scenario=None, admin_cmd=None, admin_args=None):
            if not slots.acquire(blocking=False):
                return self.reply(429, {"error": "并发请求过多"})
            writing = (playback is not None and playback["cmd"] != "status") or scenario is not None or admin_cmd is not None
            if writing and not replay_write.acquire(blocking=False):
                slots.release()
                return self.reply(429, {"error": "回放命令正在执行"})
            try:
                with Admin(admin, timeout=3) as client:
                    if admin_cmd is not None:
                        result = client.cmd(admin_cmd, **admin_args)
                    elif scenario is not None:
                        status = client.status()
                        if status.get("open_orders", 0) != 0 or any(a.get("used_margin", 0) != 0 for a in status.get("accounts", [])):
                            return self.reply(409, {"error": "加载前必须无活动订单且无持仓保证金；Web 不重置账户"})
                        if status.get("playback", {}).get("loaded") and not status["playback"].get("paused"):
                            return self.reply(409, {"error": "加载前必须暂停当前回放"})
                        relative = scenario["path"]
                        path, spec, _ = _scenario_view(workspace, relative)
                        result = client.start_scenario(str(path), paused=True, spec=spec)
                    elif playback is not None:
                        cmd = playback["cmd"]
                        if cmd == "status":
                            result = self.replay_view(client)
                        else:
                            state = self.replay_view(client)["playback"]
                            if not state["loaded"] or state["finished"]:
                                return self.reply(409, {"error": "未加载场景或回放已完成"})
                            if cmd == "step" and not state["paused"]:
                                return self.reply(409, {"error": "单步前必须暂停回放"})
                            kwargs = {key: value for key, value in playback.items() if key != "cmd"}
                            client.cmd(cmd, **kwargs)
                            result = self.replay_view(client)
                    else:
                        result = client.settings() if patch is None else client.update_settings(patch)
                self.reply(200, result)
            except RuntimeError as e:
                return self.reply(409 if playback is not None or admin_cmd or scenario is not None else 400,
                                  {"error": "核心拒绝场景加载，请检查本地核心日志" if scenario is not None else str(e)})
            except (OSError, ValueError, ConnectionError) as e:
                return self.reply(503 if playback is not None or admin_cmd else 502, {"error": "ADMIN 不可用"})
            finally:
                if writing:
                    replay_write.release()
                slots.release()

    class Server(ThreadingHTTPServer):
        daemon_threads = True
        def get_request(self):
            conn, addr = super().get_request()
            conn.settimeout(5)
            return conn, addr
    if ":" in host:
        Server.address_family = socket.AF_INET6
    return Server((host, port), Handler)


def serve(host="127.0.0.1", port=8080, admin="127.0.0.1:5561", database=None, workspace=None):
    server = make_server(host, port, admin, database, workspace)
    print("[ctpbuddy] 本地参数后台 http://%s:%d" % (server.server_address[0], server.server_address[1]), flush=True)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()
    return 0
