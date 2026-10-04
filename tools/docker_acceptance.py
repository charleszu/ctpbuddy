#!/usr/bin/env python3
"""运行 Linux Docker 验收；本机无 Docker 或 daemon 不可用时跳过。

跳过默认以 EXIT_SKIPPED(3) 退出并打印 SKIPPED 原因，不冒充通过；仅在调用方显式
传 --allow-skip 时才以 0 退出。
"""
from __future__ import annotations

import argparse
import shutil
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
EXIT_SKIPPED = 3


def skipped(reason: str, allow_skip: bool) -> int:
    print("SKIPPED: %s" % reason)
    return 0 if allow_skip else EXIT_SKIPPED


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description="Linux Docker 验收")
    parser.add_argument("--allow-skip", action="store_true",
                        help="无 Docker / daemon 不可用时以 0 退出（默认退出码 %d）" % EXIT_SKIPPED)
    args = parser.parse_args(argv)
    docker = shutil.which("docker")
    if not docker:
        return skipped("docker 不在 PATH；CI 可执行 docker compose build/run", args.allow_skip)
    command = [docker, "compose", "-f", str(REPO / "docker" / "compose.yml"), "run", "--rm", "acceptance"]
    try:
        probe = subprocess.run([docker, "info"], stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True)
    except OSError as exc:
        return skipped("无法启动 docker: %s" % exc, args.allow_skip)
    if probe.returncode != 0:
        return skipped("Docker daemon 不可用；CI 可执行 docker compose build/run", args.allow_skip)
    return subprocess.call(command, cwd=str(REPO))


if __name__ == "__main__":
    raise SystemExit(main())
