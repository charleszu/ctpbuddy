#!/usr/bin/env python3
"""运行 Linux Docker 验收；本机无 Docker 或 daemon 不可用时安全跳过。"""
from __future__ import annotations

import shutil
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]


def main() -> int:
    docker = shutil.which("docker")
    if not docker:
        print("SKIP: docker 不在 PATH；CI 可执行 docker compose build/run")
        return 0
    command = [docker, "compose", "-f", str(REPO / "docker" / "compose.yml"), "run", "--rm", "acceptance"]
    try:
        probe = subprocess.run([docker, "info"], stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True)
    except OSError as exc:
        print("SKIP: 无法启动 docker: %s" % exc)
        return 0
    if probe.returncode != 0:
        print("SKIP: Docker daemon 不可用；CI 可执行 docker compose build/run")
        return 0
    return subprocess.call(command, cwd=str(REPO))


if __name__ == "__main__":
    raise SystemExit(main())
