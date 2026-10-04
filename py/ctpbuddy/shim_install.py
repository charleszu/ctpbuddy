"""安全的 CTPBuddy shim DLL 安装与恢复（仅标准库）。"""
from __future__ import annotations

import hashlib
import json
import os
import shutil
import stat
import tempfile
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List

SUPPORTED_SHIMS = ("thosttraderapi_se.dll", "thostmduserapi_se.dll")
INSTALL_MANIFEST = ".ctpbuddy-shim-install.json"
MANIFEST_NAMES = ("ctpbuddy-shim-manifest.json", "shim-manifest.json", "manifest.json")
BACKUP_DIR = ".ctpbuddy-backup"
RESTORE_SCRIPT = "restore_shim.py"


class ShimInstallError(Exception):
    pass


def _sha256(path: Path) -> str:
    h = hashlib.sha256()
    with open(str(path), "rb") as fh:
        for chunk in iter(lambda: fh.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def _no_links(path: Path) -> None:
    for part in (path,) + tuple(path.parents):
        if os.path.lexists(str(part)):
            info = part.lstat()
            if stat.S_ISLNK(info.st_mode) or getattr(info, "st_file_attributes", 0) & 0x400:
                raise ShimInstallError("拒绝符号链接或重解析点: %s" % part)


def _directory(path: str, label: str) -> Path:
    p = Path(os.path.abspath(path))
    _no_links(p)
    if not p.is_dir():
        raise ShimInstallError("%s必须是已存在的目录: %s" % (label, path))
    return p.resolve()


def _same_or_nested(a: Path, b: Path) -> bool:
    try:
        return os.path.commonpath((str(a), str(b))) == str(a) or os.path.commonpath((str(a), str(b))) == str(b)
    except ValueError:
        return False


def _safe_name(name: Any) -> bool:
    return isinstance(name, str) and name in SUPPORTED_SHIMS and Path(name).name == name


def _load_manifest(shim_dir: Path) -> Dict[str, Any]:
    for name in MANIFEST_NAMES:
        path = shim_dir / name
        _no_links(path)
        if path.is_file():
            try:
                with open(str(path), "r", encoding="utf-8", newline="") as fh:
                    data = json.load(fh)
            except (OSError, ValueError) as exc:
                raise ShimInstallError("无法读取 shim manifest: %s" % exc)
            if not isinstance(data, dict):
                raise ShimInstallError("shim manifest 必须是 JSON 对象")
            return data
    return {}


def _artifacts(shim_dir: Path) -> List[Dict[str, Any]]:
    manifest = _load_manifest(shim_dir)
    listed = manifest.get("artifacts")
    if listed is not None and not isinstance(listed, list):
        raise ShimInstallError("shim manifest 的 artifacts 必须是列表")
    names = list(SUPPORTED_SHIMS)
    if listed is not None:
        names = []
        for item in listed:
            name = item.get("name") if isinstance(item, dict) else item
            if not _safe_name(name):
                raise ShimInstallError("manifest 包含不受支持的 shim DLL: %s" % name)
            if name not in names:
                names.append(name)
    result = []
    for name in names:
        source = shim_dir / name
        _no_links(source)
        if not source.is_file() or source.is_symlink():
            raise ShimInstallError("shim DLL 不得是目录或符号链接: %s" % source)
        result.append({"name": name, "source": str(source), "sha256": _sha256(source), "size": source.stat().st_size})
    return result


def _metadata(manifest: Dict[str, Any], shim_dir: Path) -> Dict[str, Any]:
    manifest_hash = None
    for name in MANIFEST_NAMES:
        path = shim_dir / name
        if path.is_file():
            manifest_hash = _sha256(path)
            break
    declared = manifest.get("architecture", "unknown")
    return {
        "manifest_sha256": manifest_hash,
        "version": manifest.get("version", "unknown"),
        "architecture": "manifest声明: %s（未验证PE）" % declared,
        "manifest_present": bool(manifest),
    }


def _timestamp() -> str:
    return datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S.%fZ")


def _write_json(path: Path, data: Dict[str, Any]) -> None:
    _no_links(path)
    fd, tmp_name = tempfile.mkstemp(prefix=".%s-" % path.name, suffix=".tmp", dir=str(path.parent))
    try:
        with os.fdopen(fd, "w", encoding="utf-8", newline="\n") as fh:
            json.dump(data, fh, ensure_ascii=False, indent=2, sort_keys=True)
            fh.write("\n")
            fh.flush()
            os.fsync(fh.fileno())
        os.replace(tmp_name, str(path))
    except Exception:
        try:
            os.unlink(tmp_name)
        except OSError:
            pass
        raise


def _copy_atomic(source: Path, destination: Path) -> None:
    _no_links(source)
    _no_links(destination.parent)
    if os.path.lexists(str(destination)):
        _no_links(destination)
    fd, tmp_name = tempfile.mkstemp(prefix=".%s-" % destination.name, suffix=".tmp", dir=str(destination.parent))
    try:
        with os.fdopen(fd, "wb") as out, open(str(source), "rb") as inp:
            shutil.copyfileobj(inp, out)
            out.flush()
            os.fsync(out.fileno())
        shutil.copymode(str(source), tmp_name)
        os.replace(tmp_name, str(destination))
    except Exception:
        try:
            os.unlink(tmp_name)
        except OSError:
            pass
        raise


def _create_text(path: Path, text: str) -> None:
    _no_links(path)
    with open(str(path), "x", encoding="utf-8", newline="\n") as fh:
        fh.write(text)
        fh.flush()
        os.fsync(fh.fileno())


def _hash_value(value: Any) -> bool:
    return isinstance(value, str) and len(value) == 64 and all(c in "0123456789abcdef" for c in value)


def _within(path: Path, root: Path) -> bool:
    try:
        return os.path.commonpath((str(path), str(root))) == str(root)
    except ValueError:
        return False


def _validate_item(item: Any, target: Path, backup: Path) -> Dict[str, Any]:
    if not isinstance(item, dict) or not _safe_name(item.get("name")):
        raise ShimInstallError("安装清单包含不受支持的文件名")
    if type(item.get("existed")) is not bool or not _hash_value(item.get("installed_sha256")):
        raise ShimInstallError("安装清单中的状态或哈希无效")
    if item["existed"] and (not _hash_value(item.get("target_sha256_before")) or
                            item.get("backup_sha256") != item["target_sha256_before"]):
        raise ShimInstallError("安装清单中的原件/备份 SHA256 无效")
    if item.get("status") not in ("planned", "installed", "restoring", "restored"):
        raise ShimInstallError("安装清单中的文件状态无效")
    name = item["name"]
    destination = target / name
    saved = backup / (name + ".backup")
    _no_links(destination)
    _no_links(saved)
    if not _within(destination, target) or not _within(saved, backup):
        raise ShimInstallError("安装清单包含路径穿越")
    return {"item": item, "destination": destination, "saved": saved}


def _restore_script(target: Path) -> str:
    return """#!/usr/bin/env python3
# 需要可导入的 ctpbuddy 包；恢复逻辑由经过同样校验的模块执行。
import sys
try:
    from ctpbuddy.shim_install import restore_shim
except ImportError:
    print("需要安装或通过 PYTHONPATH 提供 ctpbuddy 包，然后运行 restore-shim", file=sys.stderr)
    sys.exit(1)

try:
    result = restore_shim(%r)
except Exception as exc:
    print("restore-shim failed: %%s" %% exc, file=sys.stderr)
    sys.exit(1)
print("restored: %%s" %% result["target_dir"])
""" % str(target)


def _read_install(target: Path) -> Dict[str, Any]:
    marker = target / INSTALL_MANIFEST
    _no_links(marker)
    if not marker.is_file():
        raise ShimInstallError("目标目录没有 CTPBuddy shim 安装清单")
    try:
        with open(str(marker), "r", encoding="utf-8", newline="") as fh:
            install = json.load(fh)
    except (OSError, ValueError) as exc:
        raise ShimInstallError("无法读取安装清单: %s" % exc)
    if not isinstance(install, dict):
        raise ShimInstallError("安装清单必须是 JSON 对象")
    backup_value = install.get("backup_dir")
    if not isinstance(backup_value, str):
        raise ShimInstallError("安装清单中的备份目录无效")
    backup = Path(os.path.abspath(backup_value))
    backup_root = target / BACKUP_DIR
    _no_links(backup_root)
    if not _within(backup, backup_root) or backup.parent != backup_root:
        raise ShimInstallError("安装清单中的备份目录无效")
    _no_links(backup)
    if not backup.is_dir():
        raise ShimInstallError("备份目录不存在")
    items = install.get("files")
    if not isinstance(items, list) or not items:
        raise ShimInstallError("安装清单中的文件列表无效")
    names = [item.get("name") for item in items if isinstance(item, dict)]
    if len(names) != len(items) or any(not _safe_name(name) for name in names) or len(set(names)) != len(items):
        raise ShimInstallError("安装清单中的文件名重复或无效")
    script = target / RESTORE_SCRIPT
    _no_links(script)
    if os.path.lexists(str(script)) and (not script.is_file() or _sha256(script) != install.get("restore_script_sha256")):
        raise ShimInstallError("恢复脚本 SHA256 已变化，拒绝恢复")
    for item in items:
        checked = _validate_item(item, target, backup)
        if item.get("existed"):
            if not checked["saved"].is_file() or _sha256(checked["saved"]) != item.get("backup_sha256"):
                raise ShimInstallError("备份文件缺失或 SHA256 不匹配: %s" % checked["saved"])
    backup_manifest = backup / "manifest.json"
    _no_links(backup_manifest)
    if not backup_manifest.is_file() or install.get("backup_manifest_sha256") != _sha256(backup_manifest):
        raise ShimInstallError("备份 manifest 缺失或 SHA256 不匹配")
    with open(str(backup_manifest), "r", encoding="utf-8") as fh:
        try:
            plan = json.load(fh)
        except ValueError as exc:
            raise ShimInstallError("备份 manifest 无效: %s" % exc)
    if not isinstance(plan, dict) or not isinstance(plan.get("files"), list):
        raise ShimInstallError("备份 manifest 文件列表无效")
    fixed = ("name", "existed", "installed_sha256", "target_sha256_before", "backup_sha256")
    original = plan["files"]
    if len(original) != len(items) or any(
        not isinstance(a, dict) or any(a.get(k) != b.get(k) for k in fixed)
        for a, b in zip(original, items)
    ):
        raise ShimInstallError("安装清单与备份恢复计划不一致")
    return install


def _current_hash(path: Path) -> Any:
    _no_links(path)
    if not os.path.lexists(str(path)):
        return None
    if not path.is_file():
        raise ShimInstallError("目标路径不是普通文件: %s" % path)
    return _sha256(path)


def install_shim(target_dir: str, shim_dir: str, apply: bool = False) -> Dict[str, Any]:
    target = _directory(target_dir, "target-dir")
    source_dir = _directory(shim_dir, "shim-dir")
    if _same_or_nested(target, source_dir):
        raise ShimInstallError("target-dir 与 shim-dir 不得重合或互相嵌套")
    marker = target / INSTALL_MANIFEST
    script = target / RESTORE_SCRIPT
    _no_links(marker)
    _no_links(script)
    if os.path.lexists(str(marker)):
        raise ShimInstallError("目标目录已有 CTPBuddy shim 安装清单，请先 restore-shim")
    if os.path.lexists(str(script)):
        raise ShimInstallError("目标目录已有 restore_shim.py，拒绝覆盖")
    artifacts = _artifacts(source_dir)
    manifest_source = _load_manifest(source_dir)
    files = []
    for item in artifacts:
        destination = target / item["name"]
        _no_links(destination)
        exists = os.path.lexists(str(destination))
        if exists and not destination.is_file():
            raise ShimInstallError("目标 shim DLL 不是普通文件: %s" % destination)
        files.append({
            "name": item["name"], "source": item["source"], "source_sha256": item["sha256"],
            "size": item["size"], "existed": exists,
            "target_sha256_before": _sha256(destination) if exists else None,
            "installed_sha256": item["sha256"],
            "status": "planned",
        })
    result: Dict[str, Any] = {
        "action": "install-shim", "applied": bool(apply), "state": "planned",
        "target_dir": str(target), "shim_dir": str(source_dir), "files": files,
        "metadata": _metadata(manifest_source, source_dir),
    }
    if not apply:
        return result

    backup_root = target / BACKUP_DIR
    _no_links(backup_root)
    backup_root.mkdir(exist_ok=True)
    backup = backup_root / _timestamp()
    backup.mkdir(exist_ok=False)
    _no_links(backup)
    # 独占占用标记；准备阶段失败时没有任何 DLL 被替换。
    result["state"] = "preparing"
    result["backup_dir"] = str(backup)
    result["restore_script"] = str(script)
    script_text = _restore_script(target)
    result["restore_script_sha256"] = hashlib.sha256(script_text.encode("utf-8")).hexdigest()
    _create_text(marker, json.dumps(result, ensure_ascii=False, indent=2) + "\n")
    modified = False
    try:
        _create_text(script, script_text)
        for item in files:
            if item["existed"]:
                saved = backup / (item["name"] + ".backup")
                _copy_atomic(target / item["name"], saved)
                item["backup_sha256"] = _sha256(saved)
                if item["backup_sha256"] != item["target_sha256_before"]:
                    raise ShimInstallError("备份预验证失败: %s" % saved)
        result["state"] = "planned"
        _create_text(backup / "manifest.json", json.dumps(result, ensure_ascii=False, indent=2) + "\n")
        result["backup_manifest_sha256"] = _sha256(backup / "manifest.json")
        _write_json(marker, result)
        # 统一预验证全部备份和全部当前/源文件，然后才改第一个 DLL。
        _read_install(target)
        for item in files:
            source = Path(item["source"])
            _no_links(source)
            if _sha256(source) != item["installed_sha256"] or _current_hash(target / item["name"]) != item["target_sha256_before"]:
                raise ShimInstallError("安装前文件 SHA256 已变化")
        for item in files:
            _copy_atomic(Path(item["source"]), target / item["name"])
            modified = True
            if _sha256(target / item["name"]) != item["installed_sha256"]:
                raise ShimInstallError("安装后的 SHA256 不匹配")
            item["status"] = "installed"
            result["state"] = "installing"
            _write_json(backup / "manifest.json", result)
            result["backup_manifest_sha256"] = _sha256(backup / "manifest.json")
            _write_json(marker, result)
        result["state"] = "installed"
        _write_json(backup / "manifest.json", result)
        result["backup_manifest_sha256"] = _sha256(backup / "manifest.json")
        _write_json(marker, result)
    except Exception as exc:
        if not modified:
            try:
                marker.unlink()
                script.unlink()
            except OSError:
                pass
        raise ShimInstallError("安装失败: %s；可运行 restore-shim --target-dir %s；未改 DLL 时已清理占用状态" % (exc, target)) from exc
    return result


def restore_shim(target_dir: str) -> Dict[str, Any]:
    target = _directory(target_dir, "target-dir")
    install = _read_install(target)
    backup = Path(os.path.abspath(install["backup_dir"]))
    # planned 可能在替换后、状态落盘前中断，只允许原件或预先记录的 shim 哈希。
    checked = []
    for item in install["files"]:
        entry = _validate_item(item, target, backup)
        current = _current_hash(entry["destination"])
        before = item["target_sha256_before"]
        allowed = {item["installed_sha256"]}
        if item["status"] in ("planned", "restoring", "restored"):
            allowed.add(before)
        if current not in allowed:
            raise ShimInstallError("目标文件 SHA256 已变化，拒绝恢复: %s" % entry["destination"])
        checked.append((item, entry, current != before))
    marker = target / INSTALL_MANIFEST
    script = target / RESTORE_SCRIPT
    for item, entry, changed in checked:
        if changed:
            item["status"] = "restoring"
            _write_json(marker, install)
            destination, saved = entry["destination"], entry["saved"]
            if item["existed"]:
                _copy_atomic(saved, destination)
            else:
                _no_links(destination)
                destination.unlink()
        item["status"] = "restored"
        _write_json(marker, install)
    _no_links(marker)
    _no_links(script)
    if os.path.lexists(str(script)):
        script.unlink()
    marker.unlink()
    return {"action": "restore-shim", "target_dir": str(target), "backup_dir": str(backup)}
