#!/usr/bin/env python3
"""发布包验收：默认使用临时 PE fixture，可选验收本机真实 Shim 产物。"""
from __future__ import annotations

import json
import os
from pathlib import Path
import struct
import subprocess
import sys
import tempfile

REPO = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO / "tools"))
from release_package import DLLS, build_package  # noqa: E402


def fake_pe(path: Path, dll: bool = True) -> None:
    data = bytearray(128)
    data[:2] = b"MZ"
    struct.pack_into("<I", data, 0x3C, 64)
    data[64:68] = b"PE\0\0"
    struct.pack_into("<H", data, 68, 0x8664)
    struct.pack_into("<H", data, 70, 1)
    struct.pack_into("<H", data, 86, 0x2000 if dll else 0)
    path.write_bytes(data)


def fixture() -> Path:
    root = Path(tempfile.mkdtemp(prefix="ctpbuddy-release-fixture-"))
    (root / "shim" / "bin").mkdir(parents=True)
    (root / "py").mkdir()
    (root / "LICENSE").write_text("fixture license\n", encoding="utf-8")
    (root / "py" / "pyproject.toml").write_text('[project]\nversion = "0.1.0-fixture"\n', encoding="utf-8")
    for name in DLLS:
        fake_pe(root / "shim" / "bin" / name)
    fake_pe(root / "shim" / "bin" / "demo_td.exe", dll=False)
    (root / "ctpsdk").mkdir()
    (root / "ctpsdk" / "secret.h").write_text("must not ship", encoding="utf-8")
    (root / "data").mkdir()
    (root / "data" / "account.json").write_text("must not ship", encoding="utf-8")
    subprocess.run(["git", "-C", str(root), "init", "-q"], check=True)
    subprocess.run(["git", "-C", str(root), "config", "user.email", "test@example.invalid"], check=True)
    subprocess.run(["git", "-C", str(root), "config", "user.name", "fixture"], check=True)
    subprocess.run(["git", "-C", str(root), "add", "."], check=True)
    subprocess.run(["git", "-C", str(root), "commit", "-qm", "fixture"], check=True)
    return root


def verify(root: Path) -> None:
    output = build_package(root)
    manifest = json.loads((output / "manifest.json").read_text(encoding="utf-8"))
    assert manifest["version"] == "0.1.0-fixture"
    assert manifest["sdk_version"] == "6.7.13"
    assert manifest["architecture"] == "x64"
    assert manifest["demo_included"] is True
    assert len(manifest["commit"]) >= 40
    assert (output / "LICENSE").is_file()
    assert (output / "INSTALL.txt").is_file()
    assert all((output / "shim" / item["name"]).is_file() for item in manifest["artifacts"])
    shipped = {p.relative_to(output).as_posix() for p in output.rglob("*") if p.is_file()}
    assert not any("ctpsdk" in name or name.startswith("data/") for name in shipped)
    assert not (output / "ctpsdk").exists()
    print("fixture release package: PASS (%s)" % output)


def main() -> int:
    root = fixture()
    try:
        verify(root)
        if os.environ.get("CTPBUDDY_TEST_REAL_RELEASE") == "1":
            output = build_package(REPO)
            print("real release package: PASS (%s)" % output)
        else:
            print("real release package: SKIP (set CTPBUDDY_TEST_REAL_RELEASE=1)")
        return 0
    finally:
        # fixture/output are temporary by design; real output is retained for inspection.
        import shutil
        shutil.rmtree(root, ignore_errors=True)


if __name__ == "__main__":
    raise SystemExit(main())
