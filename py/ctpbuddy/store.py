"""JSONL journal 的可丢弃 SQLite 查询投影；不执行撮合或账本计算。"""
from __future__ import annotations

import json
import math
import os
import sqlite3
import tempfile
from pathlib import Path

from .journal import JournalError, hash_stream, journal_paths, load_events

SCHEMA_VERSION = 2
APPLICATION_ID = 0x43545042
TABLES = ("journal_event", "account", "order_record", "trade_record",
          "position_change", "position_snapshot", "account_snapshot", "audit_log", "settlement_report")
WEB_TABLES = ("account", "position_snapshot", "account_snapshot", "order_record",
              "trade_record", "audit_log", "settlement_report")
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
    last = 0
    for e in events:
        seq = e.get("seq")
        if isinstance(seq, bool) or not isinstance(seq, int) or seq != last + 1:
            raise JournalError("journal seq 必须从 1 连续递增；预期 %d，实际 %r" % (last + 1, seq))
        last = seq
        for key in ("trading_day", "ts_wall", "type"):
            if not isinstance(e.get(key), str):
                raise JournalError("seq %d: %s 必须为字符串" % (seq, key))
        vt = e.get("vt_ms")
        if isinstance(vt, bool) or not isinstance(vt, (int, float)) or not math.isfinite(vt):
            raise JournalError("seq %d: vt_ms 必须为有限数字" % seq)
        if "data" not in e or not isinstance(e["data"], (dict, type(None))):
            raise JournalError("seq %d: data 必须为对象或 null" % seq)
        for key in ("broker", "investor"):
            if key in e and not isinstance(e[key], str):
                raise JournalError("seq %d: %s 必须为字符串" % (seq, key))
        try:
            _json(e)
        except (ValueError, TypeError) as exc:
            raise JournalError("seq %d: %s" % (seq, exc)) from exc


def _project(conn, events):
    inserts = []
    for e in events:
        s, day, t = e["seq"], e["trading_day"], e["type"]
        b, i = e.get("broker", ""), e.get("investor", "")
        d = e["data"] or {}
        raw = _json(d)
        conn.execute("INSERT INTO journal_event VALUES (?,?,?,?,?,?,?,?)",
                     (s, day, e["vt_ms"], e["ts_wall"], t, b, i, _json(e["data"])))
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
        elif t == "fill":
            direction, offset = _code(d.get("direction")), _code(d.get("offset"))
            conn.execute("INSERT INTO trade_record VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?)", (
                s, day, b, i, d.get("trade_id"), d.get("order_sys_id"), d.get("order_ref"),
                d.get("instrument"), direction, offset, d.get("price"), d.get("volume"), raw))
            side = ("long" if direction == "0" else "short") if offset == "0" else ("short" if direction == "0" else "long")
            delta = d.get("volume")
            if delta is not None and offset != "0":
                delta = -delta
            conn.execute("INSERT INTO position_change VALUES (?,?,?,?,?,?,?,?,?)", (s, day, b, i, d.get("instrument"), side, offset, delta, raw))
        if t == "settlement_report":
            content_bytes = d.get("content_bytes", [])
            conn.execute("INSERT INTO settlement_report VALUES (?,?,?,?,?,?,?,?,?,?)", (
                s, day, b, i, d.get("settlement_id"), d.get("account_id", ""),
                d.get("currency_id", ""), d.get("source", "user_supplied"),
                _json(content_bytes), raw))
        if t in {"admin", "settings_updated", "scenario_loaded", "reset_account",
                 "deposit", "withdraw", "settle", "settlement", "settlement_report"}:
            conn.execute("INSERT INTO audit_log VALUES (?,?,?,?,?,?,?,?)", (
                s, day, b, i, e["ts_wall"], d.get("actor", d.get("operator")), t, raw))
        if t == "assertion" and d.get("metric") in {
            "balance", "available", "close_profit", "commission", "position_profit",
            "used_margin", "frozen_margin"}:
            conn.execute("INSERT INTO account_snapshot VALUES (?,?,?,?,?,?,?,?)", (s, 0, day, b, i, d["metric"], d.get("actual"), raw))
        if t in {"settlement", "settle"}:
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
                conn.executemany("INSERT INTO projection_meta VALUES (?,?)", [
                    ("schema_version", str(SCHEMA_VERSION)), ("journal_sha256", digest),
                    ("event_count", str(len(events))), ("last_seq", str(len(events)))])
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
    return {"schema_version": SCHEMA_VERSION, "events": len(events), "sha256": digest}


class Projection:
    """只读查询接口；缺库/版本不符不会隐式创建或迁移。"""
    def __init__(self, database):
        self.conn = sqlite3.connect(Path(database).resolve().as_uri() + "?mode=ro", uri=True)
        try:
            if (self.conn.execute("PRAGMA application_id").fetchone()[0] != APPLICATION_ID or
                    self.conn.execute("PRAGMA user_version").fetchone()[0] != SCHEMA_VERSION):
                raise JournalError("SQLite 投影 schema/version 不匹配，请 journal rebuild")
            self.conn.row_factory = sqlite3.Row
        except Exception:
            self.conn.close()
            raise

    def close(self):
        self.conn.close()

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
