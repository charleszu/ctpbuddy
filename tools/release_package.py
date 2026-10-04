#!/usr/bin/env python3
"""仅打包本机已构建的 Windows Shim；不构建、不执行、不发布。"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import stat
import struct
import subprocess
import tempfile

REPO = Path(__file__).resolve().parents[1]
DLLS = ("thosttraderapi_se.dll", "thostmduserapi_se.dll")
OPTIONAL = ("demo_td.exe",)
MACHINES = {0x014C: "x86", 0x8664: "x64", 0xAA64: "arm64"}


def no_links(path):
    for part in (path,) + tuple(path.parents):
        if os.path.lexists(str(part)):
            info = part.lstat()
            if stat.S_ISLNK(info.st_mode) or getattr(info, "st_file_attributes", 0) & 0x400:
                raise ValueError("拒绝链接或重解析点: %s" % part)


def pe_info(path):
    no_links(path)
    with path.open("rb") as stream:
        header = stream.read(64)
        if len(header) != 64 or header[:2] != b"MZ":
            raise ValueError("不是 PE 文件: %s" % path.name)
        offset = struct.unpack_from("<I", header, 0x3C)[0]
        if offset < 64 or offset > path.stat().st_size - 24:
            raise ValueError("非法 PE 偏移: %s" % path.name)
        stream.seek(offset)
        coff = stream.read(24)
        if coff[:4] != b"PE\0\0":
            raise ValueError("非法 PE 签名: %s" % path.name)
        machine = struct.unpack_from("<H", coff, 4)[0]
        characteristics = struct.unpack_from("<H", coff, 22)[0]
        if machine not in MACHINES:
            raise ValueError("不支持的 PE machine: 0x%04x" % machine)
        if bool(characteristics & 0x2000) != (path.suffix.lower() == ".dll"):
            raise ValueError("PE DLL 标记与文件类型不匹配: %s" % path.name)
    return "0x%04x" % machine, MACHINES[machine]


def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def build_package(repo=REPO, source=None, dist_parent=None):
    repo = Path(repo).absolute()
    # 来源固定为仓库自身的 shim/bin，不能通过参数指向原始 SDK。
    expected = repo / "shim" / "bin"
    source = Path(source).absolute() if source is not None else expected
    no_links(source)
    if source != expected or not source.is_dir():
        raise ValueError("来源只能是已存在的 <repo>/shim/bin")
    no_links(repo / "LICENSE")
    license_text = (repo / "LICENSE").read_text(encoding="utf-8")
    project = (repo / "py" / "pyproject.toml").read_text(encoding="utf-8")
    version_match = re.search(r'^version\s*=\s*"([^"\n]+)"', project, re.MULTILINE)
    if not version_match or not re.fullmatch(r"[0-9A-Za-z.+_-]+", version_match[1]):
        raise ValueError("无法读取安全的项目版本")
    version = version_match[1]
    commit = subprocess.check_output(["git", "-C", str(repo), "rev-parse", "HEAD"], text=True).strip()
    if not re.fullmatch(r"[0-9a-f]{40,64}", commit):
        raise ValueError("非法 commit")
    dirty = bool(subprocess.check_output(["git", "-C", str(repo), "status", "--porcelain"], text=True))
    names = list(DLLS) + [name for name in OPTIONAL if os.path.lexists(str(source / name))]
    metadata = []
    for name in names:
        path = source / name
        machine, architecture = pe_info(path)
        metadata.append({"name": name, "sha256": sha256(path), "size": path.stat().st_size,
                         "pe_machine": machine, "architecture": architecture})
    architectures = {item["architecture"] for item in metadata}
    if len(architectures) != 1:
        raise ValueError("产物架构不一致")
    if dist_parent is not None:
        dist_parent = Path(dist_parent).absolute()
        no_links(dist_parent)
        if not dist_parent.is_dir():
            raise ValueError("dist 父目录必须已存在")
    destination = Path(tempfile.mkdtemp(prefix="ctpbuddy-%s-dist-" % version, dir=dist_parent))
    try:
        shim = destination / "shim"
        shim.mkdir()
        for item in metadata:
            original = source / item["name"]
            no_links(original)
            target = shim / item["name"]
            shutil.copyfile(original, target)
            if sha256(target) != item["sha256"]:
                raise ValueError("复制期间源产物发生变化")
        architecture = metadata[0]["architecture"]
        manifest = {"schema": "ctpbuddy.release/v1", "version": version, "commit": commit,
                    "working_tree_dirty": dirty, "sdk_version": "6.7.13", "architecture": architecture,
                    "artifacts": metadata, "demo_included": "demo_td.exe" in names,
                    "provenance": "本机预构建 shim/bin；commit 为打包时源码状态，不证明二进制构建来源"}
        (destination / "manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        # 安装器只接受 DLL 白名单；demo 留在总 manifest，不进入安装 manifest。
        install_manifest = dict(manifest, artifacts=[item for item in metadata if item["name"] in DLLS])
        (shim / "ctpbuddy-shim-manifest.json").write_text(json.dumps(install_manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        (destination / "LICENSE").write_text(license_text, encoding="utf-8")
        (destination / "INSTALL.txt").write_text(
            "CTPBuddy Windows Shim %s / %s / SDK 6.7.13\n"
            "本包仅含自编译 Shim 与可选 demo，不含官方 SDK、头文件、导入库、核心服务或数据。\n"
            "先核对 manifest.json 中逐文件 SHA256、PE machine 与客户端架构。\n"
            "commit 是打包时源码状态；预构建二进制可能较旧，须单独保留构建记录。\n"
            "自行安装 Python 层与 Rust core，使用 6.7.13 兼容头文件编译客户端。\n"
            "停止目标客户端；先 dry-run：\n"
            "  ctpbuddy install-shim --target-dir <测试客户端目录> --shim-dir <本包>/shim\n"
            "审核路径和备份后追加 --apply；不要对真实生产客户端操作。\n"
            "恢复：ctpbuddy restore-shim --target-dir <测试客户端目录> --apply\n"
            "MSVC /MD 产物需要对应架构的 Visual C++ 运行库。\n"
            "demo 存在时仅可在 Windows 中连接本地仿真核心运行；Linux Docker 不执行 Windows Shim。\n"
            "核心服务、refdata 和测试数据由用户另行准备，本包绝不附带真实账户/行情数据。\n" % (version, architecture), encoding="utf-8")
        return destination
    except Exception:
        shutil.rmtree(destination)
        raise


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--dist-parent", type=Path, help="已存在的父目录；默认系统临时目录，每次创建新 dist")
    args = parser.parse_args()
    try:
        print(build_package(dist_parent=args.dist_parent))
    except (OSError, ValueError, subprocess.CalledProcessError) as exc:
        parser.exit(1, "打包失败: %s\n" % exc)


if __name__ == "__main__":
    main()
