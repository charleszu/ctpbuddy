"""仅回环可用的柜台设置 Web；不提供任意 ADMIN 命令代理。"""
from __future__ import annotations

import ipaddress
import json
import secrets
import socket
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

from .sdk import Admin

MAX_BODY = 8192


def _strict_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError("重复键: " + key)
        result[key] = value
    return result


def make_server(host="127.0.0.1", port=8080, admin="127.0.0.1:5561"):
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
    token = secrets.token_urlsafe(32)
    slots = threading.BoundedSemaphore(8)

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
            if self.path != "/api/settings":
                return self.reply(404, {"error": "路径不存在"})
            self.invoke()

        def do_POST(self):
            if not self.valid_source(write=True):
                return self.reply(403, {"error": "同源或 CSRF 校验失败"})
            if self.path != "/api/settings":
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
                if not isinstance(value, dict) or set(value) != {"patch"} or not isinstance(value["patch"], dict):
                    raise ValueError("仅允许 patch 对象")
                json.dumps(value, allow_nan=False)
            except (ValueError, UnicodeError, OSError) as e:
                return self.reply(400, {"error": str(e)})
            self.invoke(value["patch"])

        def invoke(self, patch=None):
            if not slots.acquire(blocking=False):
                return self.reply(429, {"error": "并发请求过多"})
            try:
                with Admin(admin, timeout=3) as client:
                    result = client.settings() if patch is None else client.update_settings(patch)
                self.reply(200, result)
            except RuntimeError as e:
                self.reply(400, {"error": str(e)})
            except (OSError, ValueError, ConnectionError) as e:
                self.reply(502, {"error": "ADMIN 不可用: " + str(e)})
            finally:
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


def serve(host="127.0.0.1", port=8080, admin="127.0.0.1:5561"):
    server = make_server(host, port, admin)
    print("[ctpbuddy] 本地参数后台 http://%s:%d" % (server.server_address[0], server.server_address[1]), flush=True)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()
    return 0
