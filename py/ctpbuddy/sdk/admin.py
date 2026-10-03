"""Admin control-plane client (JSON over ADMIN_REQ/ADMIN_RSP, port 5561)."""
from __future__ import annotations

import json
import socket
from typing import Any, Dict, Optional

from ..wire import ADMIN_REQ, ADMIN_RSP, Frame


class Admin:
    def __init__(self, addr: str = "127.0.0.1:5561", timeout: float = 10.0) -> None:
        host, _, port = addr.rpartition(":")
        self.sock = socket.create_connection((host, int(port)), timeout=timeout)
        self.sock.setsockopt(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)
        self._seq = 0

    def close(self) -> None:
        self.sock.close()

    def __enter__(self) -> "Admin":
        return self

    def __exit__(self, *exc: Any) -> None:
        self.close()

    def cmd(self, name: str, timeout: Optional[float] = None, **kwargs: Any) -> Dict[str, Any]:
        self._seq += 1
        body = {"cmd": name}
        body.update(kwargs)
        self.sock.sendall(Frame(ADMIN_REQ, self._seq, json.dumps(body).encode()).encode())
        f = Frame.decode_from(self.sock)
        if f is None:
            raise ConnectionError("admin connection closed")
        if f.msg_type != ADMIN_RSP:
            raise ConnectionError("expected ADMIN_RSP, got 0x%04x" % f.msg_type)
        v = json.loads(f.payload)
        if not v.get("ok"):
            raise RuntimeError(v.get("error", "admin command failed"))
        return v

    # convenience wrappers ---------------------------------------------------

    def ping(self) -> Dict[str, Any]:
        return self.cmd("ping")

    def status(self) -> Dict[str, Any]:
        return self.cmd("status")

    def settings(self) -> Dict[str, Any]:
        return self.cmd("settings_get")

    def update_settings(self, patch: Dict[str, Any]) -> Dict[str, Any]:
        return self.cmd("settings_update", patch=patch)

    def start_scenario(
        self,
        path: str,
        paused: bool = False,
        speed: Optional[float] = None,
        spec: Optional[Dict[str, Any]] = None,
    ) -> Dict[str, Any]:
        """Load a scenario dir. `spec` is the normalized scenario.json payload
        (see ctpbuddy.scenario) — applied inline by the core so a
        scenario.yaml can be driven without writing scenario.json first."""
        kw: Dict[str, Any] = {"path": path, "paused": paused}
        if speed is not None:
            kw["speed"] = speed
        if spec is not None:
            kw["spec"] = spec
        return self.cmd("start_scenario", **kw)

    def pause(self) -> Dict[str, Any]:
        return self.cmd("pause")

    def resume(self) -> Dict[str, Any]:
        return self.cmd("resume")

    def step(self) -> Dict[str, Any]:
        return self.cmd("step")

    def seek(self, at: Any) -> Dict[str, Any]:
        """Seek to a virtual time: `"HH:MM:SS"` string or ms since midnight."""
        return self.cmd("seek", at=at)

    def loop(self, on: bool = True) -> Dict[str, Any]:
        return self.cmd("loop", on=on)

    def set_speed(self, speed: float) -> Dict[str, Any]:
        return self.cmd("set_speed", speed=speed)

    def reset_account(self, investor: str = "") -> Dict[str, Any]:
        return self.cmd("reset_account", investor=investor)

    def shutdown(self) -> Dict[str, Any]:
        return self.cmd("shutdown")
