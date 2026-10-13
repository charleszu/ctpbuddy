"""JSONL journal 的可丢弃 SQLite 查询投影；不执行撮合或账本计算。"""
from __future__ import annotations

from contextlib import closing

import json
import os
import sqlite3
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Dict, Mapping, Optional

from .journal import (
    AUDIT_TYPES,
    SETTLEMENT_TYPES,
    JournalError,
    fmt_vt,
    hash_stream,
    journal_paths,
    load_events,
    verify_events,
)
from .journal_contract import (
    ACCOUNT_SNAPSHOT_TYPES,
    CONTRACT_SHA256,
    CONTRACT_VERSION,
    POSITION_CHANGE_TYPES,
    SETTLEMENT_REPORT_TYPES,
    TRADE_RECORD_TYPES,
)

SCHEMA_VERSION = 2
APPLICATION_ID = 0x43545042
TABLES = ("journal_event", "account", "order_record", "trade_record",
          "position_change", "position_snapshot", "account_snapshot", "audit_log", "settlement_report")
READ_MODEL_METADATA_FIELDS = (
    "source_kind", "journal_seq", "virtual_time", "core_hash", "projection_hash",
    "journal_hash", "calendar_version", "calendar_sha256", "schema_version",
    "contract_version", "contract_sha256", "event_count", "last_seq",
)


def _calendar_metadata_pair(version, sha256, source):
    if version is None and sha256 is None:
        return None
    if (not isinstance(version, str) or not version.strip() or
            not isinstance(sha256, str) or len(sha256) != 64 or
            any(char not in "0123456789abcdefABCDEF" for char in sha256)):
        raise JournalError("%s calendar metadata 无效" % source)
    return version, sha256.lower()


@dataclass(frozen=True)
class ReadModelMetadata:
    """Stable provenance shared by live and journal-backed read models.

    A field is ``None`` when the source cannot provide it yet.  The shape is
    deliberately fixed so consumers do not need to know whether data came
    from ADMIN or the SQLite projection.
    """

    source_kind: str
    journal_seq: Optional[int] = None
    virtual_time: Optional[str] = None
    core_hash: Optional[str] = None
    projection_hash: Optional[str] = None
    journal_hash: Optional[str] = None
    calendar_version: Optional[str] = None
    calendar_sha256: Optional[str] = None
    schema_version: Optional[str] = None
    contract_version: Optional[str] = None
    contract_sha256: Optional[str] = None
    event_count: Optional[int] = None
    last_seq: Optional[int] = None

    def as_dict(self) -> Dict[str, Any]:
        """Return the canonical JSON-friendly field order and shape."""
        return {field: getattr(self, field) for field in READ_MODEL_METADATA_FIELDS}

    @classmethod
    def from_projection(cls, metadata: Mapping[str, str]) -> "ReadModelMetadata":
        """Normalize and validate metadata stored by ``journal rebuild``."""
        required = {
            "contract_version", "contract_sha256", "schema_version",
            "journal_sha256", "core_sha256", "event_count", "last_seq",
        }
        missing = sorted(key for key in required if key not in metadata)
        if missing:
            raise JournalError("SQLite 投影 read-model metadata 缺失: " + ",".join(missing))

        def number(key: str) -> int:
            value = metadata[key]
            try:
                parsed = int(value)
            except (TypeError, ValueError) as exc:
                raise JournalError("SQLite 投影 read-model metadata 无效: " + key) from exc
            if parsed < 0:
                raise JournalError("SQLite 投影 read-model metadata 无效: " + key)
            return parsed

        if metadata["contract_version"] != str(CONTRACT_VERSION) or metadata["contract_sha256"] != CONTRACT_SHA256:
            raise JournalError("SQLite 投影 read-model contract/version 不匹配，请 journal rebuild")
        if metadata["schema_version"] != str(SCHEMA_VERSION):
            raise JournalError("SQLite 投影 read-model schema/version 不匹配，请 journal rebuild")
        last_seq = number("last_seq")
        event_count = number("event_count")
        calendar = _calendar_metadata_pair(
            metadata.get("calendar_version"), metadata.get("calendar_sha256"), "SQLite 投影")
        core_hash = metadata["core_sha256"]
        if (not isinstance(core_hash, str) or len(core_hash) != 64 or
                any(char not in "0123456789abcdefABCDEF" for char in core_hash)):
            raise JournalError("SQLite 投影 read-model metadata 无效: core_sha256")
        virtual_time = metadata.get("virtual_time")
        if virtual_time is not None and not isinstance(virtual_time, str):
            raise JournalError("SQLite 投影 read-model metadata 无效: virtual_time")
        return cls(
            source_kind="sqlite_projection",
            journal_seq=last_seq,
            virtual_time=virtual_time,
            core_hash=core_hash.lower(),
            journal_hash=metadata["journal_sha256"],
            calendar_version=calendar[0] if calendar else None,
            calendar_sha256=calendar[1] if calendar else None,
            schema_version=metadata["schema_version"],
            contract_version=metadata["contract_version"],
            contract_sha256=metadata["contract_sha256"],
            event_count=event_count,
            last_seq=last_seq,
        )

    @classmethod
    def from_admin_status(cls, status: Mapping[str, Any]) -> "ReadModelMetadata":
        """Normalize the subset of provenance currently exposed by ADMIN."""
        playback = status.get("playback")
        if not isinstance(playback, Mapping):
            raise JournalError("ADMIN read-model metadata 缺少 playback")
        journal_seq = status.get("journal_seq")
        if journal_seq is not None:
            try:
                journal_seq = int(journal_seq)
            except (TypeError, ValueError) as exc:
                raise JournalError("ADMIN read-model metadata journal_seq 无效") from exc
            if journal_seq < 0:
                raise JournalError("ADMIN read-model metadata journal_seq 无效")
        virtual_time = playback.get("virtual_time")
        if virtual_time is not None and not isinstance(virtual_time, str):
            raise JournalError("ADMIN read-model metadata virtual_time 无效")

        def text(key: str) -> Optional[str]:
            value = status.get(key)
            if value is not None and not isinstance(value, str):
                raise JournalError("ADMIN read-model metadata %s 无效" % key)
            return value

        calendar = _calendar_metadata_pair(
            status.get("calendar_version"), status.get("calendar_sha256"), "ADMIN")
        contract_version = text("contract_version")
        if contract_version is not None and contract_version != str(CONTRACT_VERSION):
            raise JournalError("ADMIN read-model metadata contract/version 不匹配")
        return cls(
            source_kind="admin_live",
            journal_seq=journal_seq,
            virtual_time=virtual_time,
            core_hash=text("core_hash"),
            contract_version=contract_version,
            calendar_version=calendar[0] if calendar else None,
            calendar_sha256=calendar[1] if calendar else None,
        )
SCHEMA = """
CREATE TABLE projection_meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
CREATE TABLE journal_event (
 seq INTEGER PRIMARY KEY, trading_day TEXT NOT NULL, vt_ms REAL NOT NULL,
 ts_wall TEXT NOT NULL, type TEXT NOT NULL, broker_id TEXT NOT NULL,
 investor_id TEXT NOT NULL, data TEXT NOT NULL);
CREATE INDEX event_account ON journal_event(broker_id, investor_id, seq);
CREATE TABLE account (
 broker_id TEXT NOT NULL, investor_id TEXT NOT NULL,
 first_seq INTEGER NOT NULL, last_seq INTEGER NOT NULL,
 PRIMARY KEY(broker_id, investor_id));
CREATE TABLE order_record (
 seq INTEGER PRIMARY KEY, trading_day TEXT NOT NULL, broker_id TEXT NOT NULL,
 investor_id TEXT NOT NULL, order_key TEXT, order_ref TEXT, order_sys_id TEXT,
 instrument_id TEXT, exchange_id TEXT, direction TEXT, offset_flag TEXT,
 price REAL, volume_total_original INTEGER, volume_traded INTEGER,
 volume_total INTEGER, status TEXT, submit_status TEXT, accepted INTEGER,
 error_id INTEGER, msg TEXT, update_seq INTEGER, data TEXT NOT NULL);
CREATE INDEX order_account ON order_record(broker_id, investor_id, trading_day, seq);
CREATE TABLE trade_record (
 seq INTEGER PRIMARY KEY, trading_day TEXT NOT NULL, broker_id TEXT NOT NULL,
 investor_id TEXT NOT NULL, trade_id TEXT, order_sys_id TEXT, order_ref TEXT,
 instrument_id TEXT, direction TEXT, offset_flag TEXT, price REAL, volume INTEGER,
 data TEXT NOT NULL);
CREATE INDEX trade_account ON trade_record(broker_id, investor_id, trading_day, seq);
CREATE TABLE position_change (
 seq INTEGER PRIMARY KEY, trading_day TEXT NOT NULL, broker_id TEXT NOT NULL,
 investor_id TEXT NOT NULL, instrument_id TEXT, side TEXT, offset_flag TEXT,
 volume_delta INTEGER, data TEXT NOT NULL);
CREATE TABLE position_snapshot (
 seq INTEGER NOT NULL, ordinal INTEGER NOT NULL, trading_day TEXT NOT NULL,
 broker_id TEXT NOT NULL, investor_id TEXT NOT NULL, instrument_id TEXT,
 side TEXT, open_date TEXT, trade_id TEXT, open_price REAL, volume INTEGER,
 last_settlement_price REAL, data TEXT NOT NULL, PRIMARY KEY(seq, ordinal));
CREATE TABLE account_snapshot (
 seq INTEGER NOT NULL, ordinal INTEGER NOT NULL, trading_day TEXT NOT NULL,
 broker_id TEXT NOT NULL, investor_id TEXT NOT NULL, metric TEXT NOT NULL,
 value REAL, data TEXT NOT NULL, PRIMARY KEY(seq, ordinal, metric));
CREATE TABLE audit_log (
 seq INTEGER PRIMARY KEY, trading_day TEXT NOT NULL, broker_id TEXT NOT NULL,
 investor_id TEXT NOT NULL, ts_wall TEXT NOT NULL, actor TEXT,
 action TEXT NOT NULL, detail TEXT NOT NULL);
CREATE TABLE settlement_report (
 seq INTEGER PRIMARY KEY, trading_day TEXT NOT NULL, broker_id TEXT NOT NULL,
 investor_id TEXT NOT NULL, settlement_id INTEGER, account_id TEXT,
 currency_id TEXT, source TEXT, content_bytes TEXT NOT NULL, data TEXT NOT NULL);
"""


def _json(value):
    return json.dumps(value, sort_keys=True, ensure_ascii=False,
                      separators=(",", ":"), allow_nan=False)


def _code(value):
    if value is None:
        return None
    if isinstance(value, (int, float)):
        n = int(value)
        return chr(n) if n >= 48 else str(n)
    return str(value)


def _validate(events):
    contract_problems = verify_events(events)
    if contract_problems:
        raise JournalError("journal contract validation failed: " + "; ".join(contract_problems[:8]))
    for e in events:
        try:
            _json(e)
        except (ValueError, TypeError) as exc:
            raise JournalError("seq %d: %s" % (e.get("seq"), exc)) from exc


def _project(conn, events):
    inserts = []
    for e in events:
        s, day, t = e["seq"], e["trading_day"], e["type"]
        b, i = e.get("broker", ""), e.get("investor", "")
        d = e["data"] or {}
        raw = _json(d)
        raw_data = raw if e["data"] is not None else "null"
        conn.execute("INSERT INTO journal_event VALUES (?,?,?,?,?,?,?,?)",
                     (s, day, e["vt_ms"], e["ts_wall"], t, b, i, raw_data))
        if b and i:
            conn.execute("INSERT INTO account VALUES (?,?,?,?) ON CONFLICT(broker_id,investor_id) DO UPDATE SET last_seq=excluded.last_seq", (b, i, s, s))
        if t == "order_insert":
            outcome = d.get("outcome") or {}
            sys_id = d.get("final_order_sys_id", d.get("order_sys_id", outcome.get("order_sys_id")))
            conn.execute("INSERT INTO order_record VALUES (" + ",".join("?" * 22) + ")", (
                s, day, b, i, d.get("order_key"), d.get("order_ref"), sys_id,
                d.get("instrument"), d.get("exchange"), _code(d.get("direction")),
                _code(d.get("offset")), d.get("limit_price"), d.get("volume"),
                None, None, None, d.get("submit_status"), outcome.get("accepted"),
                outcome.get("error_id"), outcome.get("msg"), None, raw))
            inserts.append((e, sys_id))
        elif t in TRADE_RECORD_TYPES or t in POSITION_CHANGE_TYPES:
            direction, offset = _code(d.get("direction")), _code(d.get("offset"))
            side = ("long" if direction == "0" else "short") if offset == "0" else ("short" if direction == "0" else "long")
            delta = d.get("volume")
            if delta is not None and offset != "0":
                delta = -delta
            if t in TRADE_RECORD_TYPES:
                conn.execute("INSERT INTO trade_record VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?)", (
                    s, day, b, i, d.get("trade_id"), d.get("order_sys_id"), d.get("order_ref"),
                    d.get("instrument"), direction, offset, d.get("price"), d.get("volume"), raw))
            if t in POSITION_CHANGE_TYPES:
                conn.execute("INSERT INTO position_change VALUES (?,?,?,?,?,?,?,?,?)", (s, day, b, i, d.get("instrument"), side, offset, delta, raw))
        if t in SETTLEMENT_REPORT_TYPES:
            content_bytes = d.get("content_bytes", [])
            conn.execute("INSERT INTO settlement_report VALUES (?,?,?,?,?,?,?,?,?,?)", (
                s, day, b, i, d.get("settlement_id"), d.get("account_id", ""),
                d.get("currency_id", ""), d.get("source", "user_supplied"),
                _json(content_bytes), raw))
        if t in AUDIT_TYPES:
            conn.execute("INSERT INTO audit_log VALUES (?,?,?,?,?,?,?,?)", (
                s, day, b, i, e["ts_wall"], d.get("actor", d.get("operator")), t, raw))
        if t in ACCOUNT_SNAPSHOT_TYPES and d.get("metric") in {
            "balance", "available", "close_profit", "commission", "position_profit",
            "used_margin", "frozen_margin"}:
            conn.execute("INSERT INTO account_snapshot VALUES (?,?,?,?,?,?,?,?)", (s, 0, day, b, i, d["metric"], d.get("actual"), raw))
        if t in SETTLEMENT_TYPES:
            for ordinal, a in enumerate(d.get("accounts", [])):
                ab, ai = a.get("broker", b), a["investor"]
                conn.execute("INSERT INTO account VALUES (?,?,?,?) ON CONFLICT(broker_id,investor_id) DO UPDATE SET last_seq=excluded.last_seq", (ab, ai, s, s))
                for metric in ("pre_balance", "used_margin"):
                    if metric in a:
                        conn.execute("INSERT INTO account_snapshot VALUES (?,?,?,?,?,?,?,?)", (s, ordinal, d.get("next_trading_day", day), ab, ai, metric, a[metric], _json(a)))
                for pordinal, p in enumerate(a.get("positions", [])):
                    conn.execute("INSERT INTO position_snapshot VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?)", (
                        s, ordinal * 1000000 + pordinal, d.get("next_trading_day", day), ab, ai,
                        p.get("instrument"), p.get("side"), p.get("open_date"), p.get("trade_id"),
                        p.get("open_price"), p.get("volume"), p.get("last_settlement_price"), _json(p)))
    # 核心先记录回报/成交，再记录请求总结；不能让后来的总结覆盖最终状态。
    for e in events:
        if e["type"] != "order_update":
            continue
        d = e["data"] or {}
        candidates = [(ins, sys_id) for ins, sys_id in inserts
                      if (ins["trading_day"], ins.get("broker", ""), ins.get("investor", "")) ==
                      (e["trading_day"], e.get("broker", ""), e.get("investor", ""))
                      and ins["data"].get("order_ref") == d.get("order_ref")]
        if d.get("order_sys_id"):
            candidates = [(ins, sid) for ins, sid in candidates if sid == d["order_sys_id"]]
        else:
            if d.get("order_key"):
                candidates = [(ins, sid) for ins, sid in candidates
                              if ins["data"].get("order_key") == d["order_key"]]
            else:
                candidates = [(ins, sid) for ins, sid in candidates if ins["seq"] >= e["seq"]]
        if candidates:
            ins, _ = min(candidates, key=lambda pair: pair[0]["seq"])
            conn.execute("UPDATE order_record SET status=?, volume_traded=?, volume_total=?, update_seq=? WHERE seq=?",
                         (d.get("status"), d.get("volume_traded"), d.get("volume_total"), e["seq"], ins["seq"]))


def rebuild(journal, database):
    """从完整 journal 建新库，成功后原子替换；失败不改已有库。"""
    target = Path(database).resolve()
    sources = [Path(p).resolve() for p in journal_paths(journal)]
    if target in sources:
        raise JournalError("数据库路径不能覆盖 journal")
    if not sources:
        raise JournalError("journal 目录中没有 JSONL 文件")
    events = load_events(journal)
    _validate(events)
    digest = hash_stream(events)
    core_digest = hash_stream(events, core=True)
    virtual_time = fmt_vt(events[-1]["vt_ms"]) if events else None
    fd, pending = tempfile.mkstemp(prefix=".ctpbuddy-projection-", suffix=".db", dir=target.parent)
    os.close(fd)
    try:
        conn = sqlite3.connect(pending)
        try:
            conn.execute("PRAGMA journal_mode=WAL")
            conn.execute("PRAGMA synchronous=NORMAL")
            conn.execute("PRAGMA application_id=%d" % APPLICATION_ID)
            conn.execute("PRAGMA user_version=%d" % SCHEMA_VERSION)
            conn.executescript(SCHEMA)
            with conn:
                _project(conn, events)
                metadata_rows = [
                    ("contract_version", str(CONTRACT_VERSION)),
                    ("contract_sha256", CONTRACT_SHA256),
                    ("schema_version", str(SCHEMA_VERSION)), ("journal_sha256", digest),
                    ("core_sha256", core_digest),
                    ("event_count", str(len(events))), ("last_seq", str(len(events)))]
                if virtual_time is not None:
                    metadata_rows.append(("virtual_time", virtual_time))
                calendar_rows = []
                for event in events:
                    data = event.get("data")
                    if event.get("type") != "server_start" or not isinstance(data, dict):
                        continue
                    pair = (data.get("calendar_version"), data.get("calendar_sha256"))
                    if pair != (None, None) and pair not in calendar_rows:
                        calendar_rows.append(pair)
                if len(calendar_rows) > 1:
                    raise JournalError("journal 含冲突的 calendar metadata")
                if calendar_rows:
                    calendar_version, calendar_sha256 = next(iter(calendar_rows))
                    calendar_version, calendar_sha256 = _calendar_metadata_pair(
                        calendar_version, calendar_sha256, "journal")
                    metadata_rows.extend([
                        ("calendar_version", calendar_version),
                        ("calendar_sha256", calendar_sha256),
                    ])
                conn.executemany("INSERT INTO projection_meta VALUES (?,?)", metadata_rows)
            conn.execute("PRAGMA wal_checkpoint(TRUNCATE)")
            conn.execute("PRAGMA journal_mode=DELETE")
        finally:
            conn.close()
        # 不与已有 WAL 写者竞争；查询投影不应有其他写者。
        if any(Path(str(target) + suffix).exists() for suffix in ("-wal", "-shm", "-journal")):
            raise JournalError("数据库存在活动侧文件；请关闭使用者后重建")
        os.replace(pending, target)
    finally:
        for path in (pending, pending + "-wal", pending + "-shm"):
            if os.path.exists(path):
                os.unlink(path)
    return {
        "schema_version": SCHEMA_VERSION,
        "contract_version": CONTRACT_VERSION,
        "contract_sha256": CONTRACT_SHA256,
        "events": len(events),
        "sha256": digest,
        "core_sha256": core_digest,
        "virtual_time": virtual_time,
    }


def upgrade(journal, database):
    """Upgrade a known projection from its original recording, with a backup.

    Unlike rebuild, refuse an unrelated journal. An existing projection can
    describe a prefix of a growing recording; that prefix must still match.
    """
    target = Path(database).resolve()
    events = load_events(journal)
    _validate(events)
    if not target.is_file():
        return dict(rebuild(journal, target), upgraded=True, backup=None)
    if any(Path(str(target) + suffix).exists() for suffix in ("-wal", "-shm", "-journal")):
        raise JournalError("数据库存在活动侧文件；请关闭使用者后升级")
    with closing(sqlite3.connect(target.as_uri() + "?mode=ro", uri=True)) as conn:
        try:
            if conn.execute("PRAGMA application_id").fetchone()[0] != APPLICATION_ID:
                raise JournalError("目标不是 CTPBuddy 投影")
            metadata = dict(conn.execute("SELECT key,value FROM projection_meta"))
            count = int(metadata["event_count"])
            version = int(metadata["contract_version"])
            if version < 1 or version > CONTRACT_VERSION or not 0 <= count <= len(events):
                raise JournalError("投影版本或 journal 长度不支持升级")
            if hash_stream(events[:count]) != metadata["journal_sha256"]:
                raise JournalError("journal 与已有投影来源不匹配；请提供原始完整录制")
        except (KeyError, ValueError, TypeError) as exc:
            raise JournalError("投影缺少有效来源元数据，请显式 journal rebuild") from exc
        if (count == len(events) and metadata.get("contract_version") == str(CONTRACT_VERSION)
                and metadata.get("contract_sha256") == CONTRACT_SHA256
                and metadata.get("schema_version") == str(SCHEMA_VERSION)):
            with Projection(target):
                pass
            return dict(upgraded=False, backup=None, events=count, contract_version=CONTRACT_VERSION)
        fd, backup = tempfile.mkstemp(prefix=target.name + ".pre-v%d-" % version,
                                      suffix=".bak", dir=target.parent)
        os.close(fd)
        try:
            with closing(sqlite3.connect(backup)) as saved:
                conn.backup(saved)
        except Exception:
            os.unlink(backup)
            raise
    result = rebuild(journal, target)
    return dict(result, upgraded=True, backup=backup)


class Projection:
    """只读查询接口；缺库/版本不符不会隐式创建或迁移。"""
    def __init__(self, database):
        self.conn = sqlite3.connect(Path(database).resolve().as_uri() + "?mode=ro", uri=True)
        try:
            if (self.conn.execute("PRAGMA application_id").fetchone()[0] != APPLICATION_ID or
                    self.conn.execute("PRAGMA user_version").fetchone()[0] != SCHEMA_VERSION):
                raise JournalError("SQLite 投影 schema/version 不匹配，请 journal rebuild")
            metadata = {row[0]: row[1] for row in self.conn.execute(
                "SELECT key,value FROM projection_meta")}
            self._read_model_metadata = ReadModelMetadata.from_projection(metadata)
            self.conn.row_factory = sqlite3.Row
        except Exception:
            self.conn.close()
            raise

    def close(self):
        self.conn.close()

    def metadata(self):
        """Return the immutable rebuild metadata recorded with this projection."""
        return {row[0]: row[1] for row in self.conn.execute(
            "SELECT key,value FROM projection_meta ORDER BY key")}

    def read_model_metadata(self) -> ReadModelMetadata:
        """Return validated provenance through the canonical read-model seam."""
        return self._read_model_metadata

    def __enter__(self):
        return self

    def __exit__(self, *exc):
        self.close()

    def query(self, table, *, broker=None, investor=None, trading_day=None, limit=100, offset=0):
        if table not in TABLES:
            raise ValueError("未知投影表: %s" % table)
        if not isinstance(limit, int) or isinstance(limit, bool) or not 1 <= limit <= 10000:
            raise ValueError("limit 必须在 1..10000")
        if not isinstance(offset, int) or isinstance(offset, bool) or offset < 0:
            raise ValueError("offset 必须为非负整数")
        clauses, values = [], []
        for column, value in (("broker_id", broker), ("investor_id", investor), ("trading_day", trading_day)):
            if value is not None:
                if table == "account" and column == "trading_day":
                    clauses.append("EXISTS (SELECT 1 FROM journal_event e WHERE e.broker_id=account.broker_id AND e.investor_id=account.investor_id AND e.trading_day=?)")
                    values.append(value)
                    continue
                clauses.append(column + "=?")
                values.append(value)
        order = "broker_id,investor_id" if table == "account" else ("seq,ordinal" if table in {"account_snapshot", "position_snapshot"} else "seq")
        if table == "account_snapshot":
            order += ",metric"
        sql = "SELECT * FROM " + table + (" WHERE " + " AND ".join(clauses) if clauses else "")
        return [dict(row) for row in self.conn.execute(sql + " ORDER BY " + order + " LIMIT ? OFFSET ?", values + [limit, offset])]
