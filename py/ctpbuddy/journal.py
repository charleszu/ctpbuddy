"""Journal reading + deterministic hashing (DESIGN.md §11.4, M2-3).

The JSONL journal written by the Rust core is the authoritative event
stream (DESIGN §11.2). Two hash levels serve two purposes:

- **full stream hash** (`hash_stream(events)`): every event, with
  `ts_wall` dropped — the wall clock is not reproducible, everything
  else is. Two identical drives of the same scenario must produce the
  same full hash: the M2-3 exit criterion "同一场景跑两次输出 hash 一致".
- **core hash** (`hash_stream(events, core=True)`): the semantic core
  used to compare a *replay* against its *recording*. Transport/ops
  noise (`server_start` / `server_stop` / `scenario_loaded` /
  `md_watermark` — absolute paths, release cadence) is dropped, `seq`
  is re-encoded 1..n over the filtered stream, and the absolute
  connection number (`front_id` in `session_auth` / `session_login`) is
  normalized to its first-appearance rank, so a replay that opens
  connections in the journal's first-appearance order compares equal
  regardless of the server's absolute connection numbering.
  `session_id` (a per-connection login counter) stays raw.

Canonical form: `json.dumps(..., sort_keys=True, separators=(",", ":"),
ensure_ascii=False)` — one event per line, `\n`-joined, sha256.
"""
from __future__ import annotations

import hashlib
import json
import os
from typing import Any, Dict, Iterable, Iterator, List, Optional, Sequence

#: Event types per DESIGN §11.4 (some not yet emitted by the core).
KNOWN_TYPES = frozenset({
    "server_start", "server_stop", "scenario_loaded",
    "session_auth", "session_login", "session_logout",
    "order_insert", "order_cancel", "order_update", "fill",
    "deposit", "withdraw", "reset_account", "settle", "admin",
    "md_watermark", "assertion",
})

#: Types excluded from the core hash: transport/ops noise, not semantics.
CORE_NOISE_TYPES = frozenset({
    "server_start", "server_stop", "scenario_loaded", "md_watermark",
})

#: Session events whose `data` carries connection-numbering fields.
_SESSION_TYPES = frozenset({"session_auth", "session_login"})


class JournalError(Exception):
    """A journal file could not be parsed."""


def journal_paths(path: str) -> List[str]:
    """A journal file, or every `*.jsonl` inside a journal directory."""
    if os.path.isdir(path):
        return [
            os.path.join(path, name)
            for name in sorted(os.listdir(path))
            if name.endswith(".jsonl")
        ]
    return [path]


def load_events(path: str) -> List[Dict[str, Any]]:
    """Parse a journal file (or a whole journal directory), in file order."""
    out: List[Dict[str, Any]] = []
    for p in journal_paths(path):
        with open(p, "r", encoding="utf-8") as f:
            for ln, line in enumerate(f, 1):
                line = line.strip()
                if not line:
                    continue
                try:
                    ev = json.loads(line)
                except json.JSONDecodeError as e:
                    raise JournalError("%s:%d: %s" % (p, ln, e))
                if not isinstance(ev, dict):
                    raise JournalError("%s:%d: event is not a JSON object" % (p, ln))
                out.append(ev)
    if os.path.isdir(path):
        # 回放可以倒退交易日，文件名顺序不等于事件序号顺序。
        out.sort(key=lambda ev: ev.get("seq", 0))
    return out


def canonical(event: Dict[str, Any]) -> str:
    """Canonical JSON for one event (ts_wall excluded)."""
    e = {k: v for k, v in event.items() if k != "ts_wall"}
    return json.dumps(e, sort_keys=True, ensure_ascii=False, separators=(",", ":"))


def core_normalize(events: Iterable[Dict[str, Any]]) -> List[Dict[str, Any]]:
    """Project events onto the semantic core (see module docstring).

    `front_id` is the server's absolute connection number (a recording
    whose clients connected before the admin session gets different
    numbers than a replay) — it is normalized to its first-appearance
    rank. `session_id` is a per-connection login counter, stable across a
    faithful replay, and stays raw so a re-login shows up as a diff.
    """
    front_rank: Dict[Any, int] = {}
    out: List[Dict[str, Any]] = []
    for seq, e in enumerate(
        (e for e in events if e.get("type") not in CORE_NOISE_TYPES), 1
    ):
        e = dict(e)
        e["seq"] = seq
        if e.get("type") in _SESSION_TYPES:
            data = dict(e.get("data") or {})
            if "front_id" in data:
                v = data["front_id"]
                data["front_id"] = front_rank.setdefault(v, len(front_rank) + 1)
            e["data"] = data
        out.append(e)
    return out


def hash_stream(
    events: Sequence[Dict[str, Any]],
    *,
    core: bool = False,
    only: Optional[Iterable[str]] = None,
    skip: Optional[Iterable[str]] = None,
) -> str:
    """sha256 over the canonical event stream.

    `core=True` hashes the semantic core (`core_normalize` first).
    `only` / `skip` restrict the event types (applied after `core`).
    """
    only_s = frozenset(only) if only is not None else None
    skip_s = frozenset(skip) if skip is not None else None
    stream = core_normalize(events) if core else events
    h = hashlib.sha256()
    for e in stream:
        t = e.get("type")
        if only_s is not None and t not in only_s:
            continue
        if skip_s is not None and t in skip_s:
            continue
        h.update(canonical(e).encode("utf-8"))
        h.update(b"\n")
    return h.hexdigest()


def verify_events(events: Sequence[Dict[str, Any]]) -> List[str]:
    """Structural problems in a recorded stream (empty list = healthy)."""
    problems: List[str] = []
    last = 0
    for i, e in enumerate(events, 1):
        for k in ("seq", "ts_wall", "trading_day", "vt_ms", "type", "data"):
            if k not in e:
                problems.append("line %d: missing field %r" % (i, k))
        t = e.get("type")
        if t not in KNOWN_TYPES:
            problems.append("line %d: unknown event type %r" % (i, t))
        s = e.get("seq")
        if isinstance(s, (int, float)) and not isinstance(s, bool):
            if s != last + 1:
                problems.append(
                    "line %d: seq %s breaks the chain (expected %d)" % (i, s, last + 1)
                )
            last = int(s)
        elif "seq" in e:
            problems.append("line %d: seq is not a number (%r)" % (i, s))
    return problems


def count_types(events: Iterable[Dict[str, Any]]) -> Dict[str, int]:
    out: Dict[str, int] = {}
    for e in events:
        t = e.get("type", "?")
        out[t] = out.get(t, 0) + 1
    return out


def fmt_vt(vt_ms: Any) -> str:
    """34260000 -> '09:31:00.000'; night-session (negative) values wrap back
    to the evening clock, -10800000 -> '21:00:00.000' (for humans, never hashed)."""
    try:
        ms = int(float(vt_ms))
    except (TypeError, ValueError):
        return str(vt_ms)
    if ms < 0:
        ms += 86_400_000
    h, rem = divmod(ms, 3_600_000)
    m, rem = divmod(rem, 60_000)
    s, msec = divmod(rem, 1_000)
    return "%02d:%02d:%02d.%03d" % (h, m, s, msec)


def iter_events(path: str) -> Iterator[Dict[str, Any]]:
    """Lazy variant of `load_events` (same order)."""
    for p in journal_paths(path):
        with open(p, "r", encoding="utf-8") as f:
            for ln, line in enumerate(f, 1):
                line = line.strip()
                if not line:
                    continue
                try:
                    ev = json.loads(line)
                except json.JSONDecodeError as e:
                    raise JournalError("%s:%d: %s" % (p, ln, e))
                if not isinstance(ev, dict):
                    raise JournalError("%s:%d: event is not a JSON object" % (p, ln))
                yield ev
