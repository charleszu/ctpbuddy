#!/usr/bin/env python3
"""M4 install-shim CLI e2e using temporary fixture directories only."""
from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
PYTHON = sys.executable
ENV = dict(os.environ, PYTHONPATH=os.path.join(REPO, "py"))


def run(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [PYTHON, "-m", "ctpbuddy.cli"] + list(args),
        cwd=REPO,
        env=ENV,
        text=True,
        capture_output=True,
    )


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="ctpbuddy-install-shim-") as root:
        target = os.path.join(root, "target")
        shim = os.path.join(root, "shim")
        os.mkdir(target)
        os.mkdir(shim)
        with open(os.path.join(target, "thosttraderapi_se.dll"), "wb") as fh:
            fh.write(b"original")
        for name in ("thosttraderapi_se.dll", "thostmduserapi_se.dll"):
            with open(os.path.join(shim, name), "wb") as fh:
                fh.write(("fixture-" + name).encode())
        with open(os.path.join(shim, "manifest.json"), "w", encoding="utf-8") as fh:
            json.dump({"version": "m4-fixture", "architecture": "x64"}, fh)

        dry = run("install-shim", "--target-dir", target, "--shim-dir", shim)
        assert dry.returncode == 0, dry.stderr
        assert "dry-run" in dry.stdout
        assert open(os.path.join(target, "thosttraderapi_se.dll"), "rb").read() == b"original"

        applied = run("install-shim", "--target-dir", target, "--shim-dir", shim, "--apply")
        assert applied.returncode == 0, applied.stderr
        assert "m4-fixture" in applied.stdout
        assert open(os.path.join(target, "thostmduserapi_se.dll"), "rb").read().startswith(b"fixture-")

        repeated = run("install-shim", "--target-dir", target, "--shim-dir", shim, "--apply")
        assert repeated.returncode != 0 and "已有" in repeated.stderr, repeated.stderr

        restored = run("uninstall-shim", "--target-dir", target)
        assert restored.returncode == 0, restored.stderr
        assert open(os.path.join(target, "thosttraderapi_se.dll"), "rb").read() == b"original"
        assert not os.path.exists(os.path.join(target, "thostmduserapi_se.dll"))
    print("M4 INSTALL-SHIM CLI E2E: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
