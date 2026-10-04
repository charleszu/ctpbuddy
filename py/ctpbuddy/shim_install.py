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
    with path.open("rb") as fh:
        for chunk in iter(lambda: fh.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def _no_links(path: Path) -> None:
    for part in (path,) + tuple(path.parents):
        if part.exists() or part.is_symlink():
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
        if path.is_file():
            try:
                data = json.loads(path.read_text(encoding="utf-8"))
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
        if not source.is_file():
            raise ShimInstallError("缺少受支持的 shim DLL: %s" % source)
        # Do not follow a directory or accept an arbitrary DLL.  Symlinks are
        # rejected because the source must be an actual build artifact here.
        if source.is_symlink():
            raise ShimInstallError("shim DLL 不得是符号链接: %s" % source)
        result.append({
            "name": name,
            "source": str(source),
            "sha256": _sha256(source),
            "size": source.stat().st_size,
        })
    return result


def _metadata(manifest: Dict[str, Any], shim_dir: Path) -> Dict[str, Any]:
    manifest_hash = None
    for name in MANIFEST_NAMES:
        path = shim_dir / name
        if path.is_file():
            manifest_hash = _sha256(path)
            break
    return {
        "manifest_sha256": manifest_hash,
        "version": manifest.get("version", "unknown"),
        "architecture": manifest.get("architecture", "unknown (未验证PE架构)"),
        "manifest_present": bool(manifest),
    }


def _timestamp() -> str:
    return datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S.%fZ")


def _write_json(path: Path, data: Dict[str, Any]) -> None:
    fd, tmp_name = tempfile.mkstemp(prefix=".%s-" % path.name, suffix=".tmp", dir=str(path.parent))
    try:
        with os.fdopen(fd, "w", encoding="utf-8", newline="\n") as fh:
            json.dump(data, fh, ensure_ascii=False, indent=2, sort_keys=True)
            fh.write("\n")
            fh.flush()
            os.fsync(fh.fileno())
        os.replace(tmp_name, path)
    except Exception:
        try:
            os.unlink(tmp_name)
        except OSError:
            pass
        raise


def _copy_atomic(source: Path, destination: Path) -> None:
    fd, tmp_name = tempfile.mkstemp(prefix=".%s-" % destination.name, suffix=".tmp", dir=str(destination.parent))
    try:
        with os.fdopen(fd, "wb") as out, source.open("rb") as inp:
            shutil.copyfileobj(inp, out)
            out.flush()
            os.fsync(out.fileno())
        shutil.copymode(str(source), tmp_name)
        os.replace(tmp_name, destination)
    except Exception:
        try:
            os.unlink(tmp_name)
        except OSError:
            pass
        raise


def _restore_script(target: Path, backup: Path) -> str:
    return """#!/usr/bin/env python3
import json
import shutil
from pathlib import Path

TARGET = Path(%r)
BACKUP = Path(%r)
manifest = json.loads((BACKUP / 'manifest.json').read_text(encoding='utf-8'))
for item in manifest['files']:
    destination = TARGET / item['name']
    saved = BACKUP / (item['name'] + '.backup')
    if item['existed']:
        shutil.copy2(saved, destination)
    elif destination.exists():
        destination.unlink()
print('restored', TARGET)
""" % (str(target), str(backup))


def install_shim(target_dir: str, shim_dir: str, apply: bool = False) -> Dict[str, Any]:
    target = _directory(target_dir, "target-dir")
    source_dir = _directory(shim_dir, "shim-dir")
    if _same_or_nested(target, source_dir):
        raise ShimInstallError("target-dir 与 shim-dir 不得重合或互相嵌套")
    marker = target / INSTALL_MANIFEST
    if marker.exists():
        raise ShimInstallError("目标目录已有 CTPBuddy shim 安装，请先 restore-shim")
    artifacts = _artifacts(source_dir)
    manifest_source = _load_manifest(source_dir)
    files = []
    for item in artifacts:
        destination = target / item["name"]
        if destination.is_symlink():
            raise ShimInstallError("目标 shim DLL 不得是符号链接: %s" % destination)
        files.append({
            "name": item["name"],
            "source": item["source"],
            "source_sha256": item["sha256"],
            "size": item["size"],
            "existed": destination.exists(),
            "target_sha256_before": _sha256(destination) if destination.is_file() else None,
        })
    result: Dict[str, Any] = {
        "action": "install-shim",
        "applied": bool(apply),
        "target_dir": str(target),
        "shim_dir": str(source_dir),
        "files": files,
        "metadata": _metadata(manifest_source, source_dir),
    }
    if not apply:
        return result
    backup = target / BACKUP_DIR / _timestamp()
    backup.mkdir(parents=True, exist_ok=False)
    try:
        for item in files:
            destination = target / item["name"]
            if item["existed"]:
                shutil.copy2(destination, backup / (item["name"] + ".backup"))
        backup_manifest = dict(result)
        backup_manifest.update({"backup_dir": str(backup), "files": files})
        _write_json(backup / "manifest.json", backup_manifest)
        for item in files:
            _copy_atomic(Path(item["source"]), target / item["name"])
            item["installed_sha256"] = _sha256(target / item["name"])
        result["backup_dir"] = str(backup)
        result["files"] = files
        _write_json(backup / "manifest.json", dict(result))
        result["restore_script"] = str(target / RESTORE_SCRIPT)
        _write_json(marker, result)
        script = target / RESTORE_SCRIPT
        script.write_text(_restore_script(target, backup), encoding="utf-8", newline="\n")
        try:
            script.chmod(script.stat().st_mode | stat.S_IXUSR)
        except OSError:
            pass
    except Exception:
        # Never remove originals or attempt a destructive rollback here.  The
        # backup remains available for manual recovery if a target is locked.
        raise
    return result


def restore_shim(target_dir: str) -> Dict[str, Any]:
    target = _directory(target_dir, "target-dir")
    marker = target / INSTALL_MANIFEST
    if not marker.is_file():
        raise ShimInstallError("目标目录没有 CTPBuddy shim 安装清单")
    try:
        install = json.loads(marker.read_text(encoding="utf-8"))
    except (OSError, ValueError) as exc:
        raise ShimInstallError("无法读取安装清单: %s" % exc)
    backup = Path(install.get("backup_dir", "")).resolve()
    backup_root = (target / BACKUP_DIR).resolve()
    if not backup.is_dir() or backup.parent != backup_root or os.path.commonpath((str(target), str(backup))) != str(target):
        raise ShimInstallError("安装清单中的备份目录无效")
    items = install.get("files")
    if not isinstance(items, list) or not items:
        raise ShimInstallError("安装清单中的文件列表无效")
    for item in items:
        if not isinstance(item, dict) or not _safe_name(item.get("name")):
            raise ShimInstallError("安装清单包含不受支持的文件名")
        destination = target / item["name"]
        expected = item.get("installed_sha256")
        if destination.is_file() and expected and _sha256(destination) != expected:
            raise ShimInstallError("目标文件 SHA256 已变化，拒绝恢复: %s" % destination)
    for item in install.get("files", []):
        destination = target / item["name"]
        saved = backup / (item["name"] + ".backup")
        if item.get("existed"):
            if not saved.is_file():
                raise ShimInstallError("缺少备份文件: %s" % saved)
            _copy_atomic(saved, destination)
        elif destination.exists():
            destination.unlink()
    marker.unlink()
    script = target / RESTORE_SCRIPT
    if script.exists():
        script.unlink()
    return {"action": "restore-shim", "target_dir": str(target), "backup_dir": str(backup)}
