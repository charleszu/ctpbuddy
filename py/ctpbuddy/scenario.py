"""Scenario DSL (DESIGN.md §7.4): scenario.yaml parsing, validation, compile.

YAML is the authoring format; the Rust core consumes the normalized JSON form
(`scenario.json`, produced by `compile_scenario` or sent inline over the ADMIN
`start_scenario` command). This module is the single parser/validator — the
core never sees YAML.

Supported YAML subset (strict; anything else is an error with a line number):

- block mappings (`key: value`, nested by indentation) and block sequences
  (`- item`, `- key: value` mapping items);
- scalars: plain, 'single' and "double" quoted, numbers, true/false/null;
- one-level flow mappings (`expect: { orders_filled: ">=1" }`) with scalar
  values only — no flow sequences, anchors, multi-line strings or tags;
- full-line `#` comments and trailing ` #` comments on plain scalars.

Schema (times resolve to the trading-day timeline: day-session ms since
midnight, night-session times from 18:00 negative):

    name: flash-crash
    source: {kind: csv, path: ticks.csv}   # path relative to the scenario dir
    transforms:
      - kind: freeze      # 停牌: [at, at+duration) 无行情
        at: "09:32:00"
        duration: 30s
      - kind: gap         # 跳空: at 起价格平移 shift（涨跌停价不动）
        at: "09:33:00"
        shift: -2.0
        instrument: rb2601  # 可选：只作用于该合约；缺省对全部合约生效
      - kind: liquidity   # 流动性: from 起五档挂量 × scale
        from: "09:31:00"
        scale: 0.5
    clock:
      time_scale: 0       # 0 = as fast as possible; else wall-ms per virtual-ms
      start: "09:30:00"   # drop ticks before this virtual time
    accounts:
      - investor: "001"   # auto-open with this initial balance
        balance: 2000000
    assertions:           # one-shot, evaluated when the virtual clock passes
      - after: 5s         # scenario t0 + after (t0 = first delivered tick)
        investor: "001"
        expect: { orders_filled: ">=1" }

Durations: `90s` / `5m` / `1h` / `1h30m` / plain number (seconds).
Times: `HH:MM:SS[.mmm]` or `YYYY-MM-DD HH:MM:SS[.mmm]` (date part accepted
for authoring compatibility; single-day replay uses the time part).
Every transform accepts an optional `instrument` (non-empty string): the
transform then applies to that contract only; omitted = every contract.

The normalized JSON form (`scenario.json`) uses resolved millisecond keys
(`at_ms` / `duration_ms` / `from_ms` / `start_ms` / `after_ms`) and is
re-validated by `normalize_json_spec` on every load, so a hand-edited or
stale cache is rejected here rather than by the core.
"""
from __future__ import annotations

import datetime
import json
import math
import os
import re
from typing import Any, Dict, List, Optional, Tuple

__all__ = [
    "ScenarioError",
    "parse_yaml",
    "load_scenario_spec",
    "compile_scenario",
    "normalize_spec",
    "normalize_json_spec",
    "parse_duration_ms",
    "parse_time_ms",
]

TRANSFORM_KINDS = ("freeze", "gap", "liquidity")

KNOWN_METRICS = (
    "balance",
    "available",
    "close_profit",
    "commission",
    "position_profit",
    "used_margin",
    "frozen_margin",
    "orders_filled",
    "open_orders",
    "fills",
)
KNOWN_OPS = (">=", "<=", ">", "<", "==", "!=")


class ScenarioError(Exception):
    """Raised for any DSL syntax/schema problem; message carries the line."""


# --------------------------------------------------------------------------
# mini YAML
# --------------------------------------------------------------------------

def _indent_of(raw: str) -> int:
    return len(raw) - len(raw.lstrip(" "))


def _strip_comment(s: str) -> str:
    """Drop a trailing ` #...` comment (only outside quotes)."""
    in_s = in_d = False
    for i, ch in enumerate(s):
        if ch == "'" and not in_d:
            in_s = not in_s
        elif ch == '"' and not in_s:
            in_d = not in_d
        elif ch == "#" and not in_s and not in_d and i > 0 and s[i - 1] in " \t":
            return s[:i].rstrip()
    return s.rstrip()


def _scalar(s: str, lineno: int) -> Any:
    s = s.strip()
    if len(s) >= 2 and s[0] == s[-1] and s[0] in "'\"":
        inner = s[1:-1]
        if s[0] == '"':
            inner = inner.replace('\\"', '"').replace("\\n", "\n").replace("\\t", "\t").replace("\\\\", "\\")
        return inner
    if s in ("true", "True"):
        return True
    if s in ("false", "False"):
        return False
    if s in ("null", "~", ""):
        return None
    try:
        return int(s)
    except ValueError:
        pass
    try:
        return float(s)
    except ValueError:
        pass
    return s


def _split_flow(inner: str, lineno: int) -> List[str]:
    parts, buf, depth, in_s, in_d = [], "", 0, False, False
    for ch in inner:
        if ch == "'" and not in_d:
            in_s = not in_s
        elif ch == '"' and not in_s:
            in_d = not in_d
        if ch in "{[" and not in_s and not in_d:
            depth += 1
        elif ch in "}]" and not in_s and not in_d:
            depth -= 1
        if ch == "," and depth == 0 and not in_s and not in_d:
            parts.append(buf)
            buf = ""
        else:
            buf += ch
    if buf.strip():
        parts.append(buf)
    return [p for p in parts if p.strip()]


def _parse_flow_map(s: str, lineno: int) -> Dict[str, Any]:
    s = s.strip()
    if not (s.startswith("{") and s.endswith("}")):
        raise ScenarioError("line %d: 期望 flow mapping '{ ... }'，得到 %r" % (lineno, s))
    out: Dict[str, Any] = {}
    inner = s[1:-1].strip()
    if not inner:
        return out
    for part in _split_flow(inner, lineno):
        key, sep, val = part.partition(":")
        if not sep:
            raise ScenarioError("line %d: flow mapping 缺少 ':' —— %r" % (lineno, part.strip()))
        k = _scalar(key, lineno)
        if not isinstance(k, str):
            raise ScenarioError("line %d: flow mapping 键必须是字符串" % lineno)
        out[k] = _scalar(val, lineno)
    return out


def _parse_map(lines: List[Tuple[int, str]], pos: int, indent: int) -> Tuple[Dict[str, Any], int]:
    out: Dict[str, Any] = {}
    while pos < len(lines):
        lineno, raw = lines[pos]
        cur = _indent_of(raw)
        if cur < indent:
            break
        if cur > indent:
            raise ScenarioError("line %d: 缩进不符合层级（期望 %d 空格，得到 %d）" % (lineno, indent, cur))
        s = _strip_comment(raw.strip())
        if s.startswith("-"):
            break
        key, sep, rest = s.partition(":")
        if not sep:
            raise ScenarioError("line %d: 期望 'key: value'，得到 %r" % (lineno, s))
        key = _scalar(key.strip(), lineno)
        if not isinstance(key, str):
            raise ScenarioError("line %d: 映射键必须是字符串" % lineno)
        rest = rest.strip()
        pos += 1
        if rest:
            out[key] = _parse_flow_map(rest, lineno) if rest.startswith("{") else _scalar(rest, lineno)
        elif pos < len(lines):
            nxt_indent = _indent_of(lines[pos][1])
            nxt_is_seq = _strip_comment(lines[pos][1].strip()).startswith("-")
            if nxt_indent > indent or (nxt_is_seq and nxt_indent == indent):
                # nested block — including the common "sequence at the key's
                # own indent" style
                val, pos = _parse_node(lines, pos, nxt_indent)
                out[key] = val
            else:
                out[key] = None
        else:
            out[key] = None
    return out, pos


def _parse_seq(lines: List[Tuple[int, str]], pos: int, indent: int) -> Tuple[List[Any], int]:
    out: List[Any] = []
    while pos < len(lines):
        lineno, raw = lines[pos]
        cur = _indent_of(raw)
        if cur < indent:
            break
        if cur > indent:
            raise ScenarioError("line %d: 缩进不符合层级（期望 %d 空格，得到 %d）" % (lineno, indent, cur))
        s = _strip_comment(raw.strip())
        if not s.startswith("-"):
            break
        after = s[1:]
        content = after.lstrip()
        pos += 1
        if not content:
            if pos < len(lines) and _indent_of(lines[pos][1]) > indent:
                val, pos = _parse_node(lines, pos, _indent_of(lines[pos][1]))
                out.append(val)
            else:
                out.append(None)
            continue
        key_col = indent + 1 + (len(after) - len(content))
        if content.startswith(("'", '"')) or ":" not in _strip_comment(content):
            out.append(_scalar(content, lineno))
            continue
        # "- key: value" mapping item: re-anchor the first line at the key
        # column and parse a mapping from there; following keys of the same
        # item live at that column, the next "- " stops it.
        lines[pos - 1] = (lineno, " " * key_col + content)
        val, pos = _parse_map(lines, pos - 1, key_col)
        out.append(val)
    return out, pos


def _parse_node(lines: List[Tuple[int, str]], pos: int, indent: int) -> Tuple[Any, int]:
    s = _strip_comment(lines[pos][1].strip())
    if s.startswith("-"):
        return _parse_seq(lines, pos, indent)
    return _parse_map(lines, pos, indent)


def parse_yaml(text: str) -> Dict[str, Any]:
    """Parse the constrained scenario YAML subset into nested dicts/lists."""
    lines: List[Tuple[int, str]] = []
    for lineno, raw in enumerate(text.splitlines(), 1):
        if not raw.strip() or raw.lstrip().startswith("#"):
            continue
        if "\t" in raw:
            raise ScenarioError("line %d: 不支持 tab 缩进（用空格）" % lineno)
        lines.append((lineno, raw))
    if not lines:
        return {}
    node, pos = _parse_node(lines, 0, _indent_of(lines[0][1]))
    if pos != len(lines):
        raise ScenarioError("line %d: 无法解析的残留内容 %r" % (lines[pos][0], lines[pos][1].strip()))
    if not isinstance(node, dict):
        raise ScenarioError("场景文件顶层必须是映射（key: value）")
    return node


# --------------------------------------------------------------------------
# time / duration
# --------------------------------------------------------------------------

_DURATION_RE = re.compile(r"^(?:(\d+)h)?(?:(\d+)m)?(?:(\d+)s)?$")


def parse_duration_ms(s: str) -> float:
    """`90s` / `5m` / `1h30m` / plain number (seconds) -> virtual ms."""
    s = str(s).strip()
    m = _DURATION_RE.match(s)
    if not m or not any(m.groups()):
        try:
            return float(s) * 1000.0
        except ValueError:
            raise ScenarioError("无法解析 duration: %r（期望 90s / 5m / 1h30m 或秒数）" % s)
    h, mi, se = (int(g) if g else 0 for g in m.groups())
    return (h * 3600 + mi * 60 + se) * 1000.0


#: Clock times at/after 18:00 are the night session that opens the *next*
#: trading day; they map to negative values so the timeline is monotonic across
#: midnight (mirror of `ctpbuddy_market::session_ms` in the Rust core).
NIGHT_START_MS = 18 * 3_600_000.0
DAY_MS = 86_400_000.0


def session_ms(clock_ms: float) -> float:
    return clock_ms - DAY_MS if clock_ms >= NIGHT_START_MS else clock_ms


def parse_time_ms(s: str) -> float:
    """`HH:MM:SS[.mmm]` or `YYYY-MM-DD HH:MM:SS[.mmm]` -> trading-day timeline ms.

    Day-session times are ms since midnight; night-session times (>= 18:00)
    are negative, e.g. 21:00 -> -10_800_000.
    """
    s = str(s).strip()
    if " " in s:
        s = s.rsplit(" ", 1)[1]  # single-day replay: the date part is ignored
    parts = s.split(":")
    if len(parts) != 3:
        raise ScenarioError("无法解析时间: %r（期望 HH:MM:SS）" % s)
    try:
        h, m = int(parts[0]), int(parts[1])
        sec = float(parts[2])
    except ValueError:
        raise ScenarioError("无法解析时间: %r（期望 HH:MM:SS）" % s)
    if not (0 <= h < 24 and 0 <= m < 60 and 0 <= sec < 60):
        raise ScenarioError("时间越界: %r" % s)
    return session_ms((h * 3600 + m * 60 + sec) * 1000.0)


def _num(v: Any, what: str) -> float:
    if isinstance(v, bool) or not isinstance(v, (int, float)):
        raise ScenarioError("%s 必须是数字，得到 %r" % (what, v))
    return float(v)


def _str(v: Any, what: str) -> str:
    if isinstance(v, bool) or v is None:
        raise ScenarioError("%s 必须是字符串，得到 %r" % (what, v))
    if isinstance(v, (int, float)):
        return str(v)
    if not isinstance(v, str):
        raise ScenarioError("%s 必须是字符串，得到 %r" % (what, v))
    return v


# --------------------------------------------------------------------------
# spec normalization / validation
# --------------------------------------------------------------------------

def _transform_instrument(item: Dict[str, Any], where: str) -> Optional[str]:
    """Optional per-transform `instrument` (non-empty string); None = all."""
    if "instrument" not in item or item["instrument"] is None:
        return None
    v = item["instrument"]
    if not isinstance(v, str) or not v.strip() or "\x00" in v:
        raise ScenarioError("%s.instrument 必须是非空字符串，得到 %r" % (where, v))
    return v.strip()


def _norm_transforms(raw: Any) -> List[Dict[str, Any]]:
    if raw is None:
        return []
    if not isinstance(raw, list):
        raise ScenarioError("transforms 必须是列表")
    out: List[Dict[str, Any]] = []
    for i, item in enumerate(raw):
        if not isinstance(item, dict):
            raise ScenarioError("transforms[%d] 必须是映射" % i)
        where = "transforms[%d]" % i
        kind = _str(item.get("kind"), where + ".kind")
        if kind == "freeze":
            entry = {
                "kind": "freeze",
                "at_ms": parse_time_ms(_str(item.get("at"), where + ".at")),
                "duration_ms": parse_duration_ms(item.get("duration", 0)),
            }
        elif kind == "gap":
            entry = {
                "kind": "gap",
                "at_ms": parse_time_ms(_str(item.get("at"), where + ".at")),
                "shift": _num(item.get("shift"), where + ".shift"),
            }
        elif kind == "liquidity":
            entry = {
                "kind": "liquidity",
                "from_ms": parse_time_ms(_str(item.get("from"), where + ".from")),
                "scale": _num(item.get("scale"), where + ".scale"),
            }
        else:
            raise ScenarioError("%s: 未知 kind %r（%s）" % (where, kind, "/".join(TRANSFORM_KINDS)))
        instrument = _transform_instrument(item, where)
        if instrument is not None:
            entry["instrument"] = instrument
        out.append(entry)
    return out


#: Normalized (scenario.json) transform fields per kind, besides `kind` and
#: the optional `instrument`.
_JSON_TRANSFORM_FIELDS = {
    "freeze": ("at_ms", "duration_ms"),
    "gap": ("at_ms", "shift"),
    "liquidity": ("from_ms", "scale"),
}


def _finite(v: Any, what: str) -> float:
    n = _num(v, what)
    if not math.isfinite(n):
        raise ScenarioError("%s 必须是有限数字，得到 %r" % (what, v))
    return n


def _check_json_transforms(raw: Any) -> List[Dict[str, Any]]:
    """Validate transforms already in normalized form (`*_ms` keys)."""
    if raw is None:
        return []
    if not isinstance(raw, list):
        raise ScenarioError("transforms 必须是列表")
    out: List[Dict[str, Any]] = []
    for i, item in enumerate(raw):
        where = "transforms[%d]" % i
        if not isinstance(item, dict):
            raise ScenarioError("%s 必须是映射" % where)
        kind = _str(item.get("kind"), where + ".kind")
        fields = _JSON_TRANSFORM_FIELDS.get(kind)
        if fields is None:
            raise ScenarioError("%s: 未知 kind %r（%s）" % (where, kind, "/".join(TRANSFORM_KINDS)))
        unknown = set(item) - set(fields) - {"kind", "instrument"}
        if unknown:
            raise ScenarioError("%s: 未知字段 %s" % (where, ", ".join(sorted(unknown))))
        entry: Dict[str, Any] = {"kind": kind}
        for k in fields:
            if k not in item:
                raise ScenarioError("%s(%s): 缺少 %s" % (where, kind, k))
            entry[k] = _finite(item[k], "%s.%s" % (where, k))
        if kind == "freeze" and entry["duration_ms"] < 0:
            raise ScenarioError("%s.duration_ms 不能为负" % where)
        instrument = _transform_instrument(item, where)
        if instrument is not None:
            entry["instrument"] = instrument
        out.append(entry)
    return out


def _check_json_clock(raw: Any) -> Dict[str, Any]:
    if raw is None:
        return {}
    if not isinstance(raw, dict):
        raise ScenarioError("clock 必须是映射")
    unknown = set(raw) - {"time_scale", "start_ms"}
    if unknown:
        raise ScenarioError("clock: 未知字段 %s（编译后形式为 time_scale/start_ms）" % ", ".join(sorted(unknown)))
    out: Dict[str, Any] = {}
    if raw.get("time_scale") is not None:
        ts = _finite(raw["time_scale"], "clock.time_scale")
        if ts < 0:
            raise ScenarioError("clock.time_scale 不能为负（0 = 尽快）")
        out["time_scale"] = ts
    if raw.get("start_ms") is not None:
        out["start_ms"] = _finite(raw["start_ms"], "clock.start_ms")
    return out


def _check_json_assertions(raw: Any) -> List[Dict[str, Any]]:
    if raw is None:
        return []
    if not isinstance(raw, list):
        raise ScenarioError("assertions 必须是列表")
    out: List[Dict[str, Any]] = []
    for i, item in enumerate(raw):
        where = "assertions[%d]" % i
        if not isinstance(item, dict):
            raise ScenarioError("%s 必须是映射" % where)
        unknown = set(item) - {"after_ms", "investor", "metric", "op", "value"}
        if unknown:
            raise ScenarioError("%s: 未知字段 %s" % (where, ", ".join(sorted(unknown))))
        metric = _str(item.get("metric"), where + ".metric")
        if metric not in KNOWN_METRICS:
            raise ScenarioError("%s: 未知指标 %r（可用: %s）" % (where, metric, "/".join(KNOWN_METRICS)))
        op = _str(item.get("op"), where + ".op")
        if op not in KNOWN_OPS:
            raise ScenarioError("%s: 未知比较符 %r（可用: %s）" % (where, op, " ".join(KNOWN_OPS)))
        after_ms = _finite(item.get("after_ms", 0), where + ".after_ms")
        if after_ms < 0:
            raise ScenarioError("%s.after_ms 不能为负" % where)
        out.append({
            "after_ms": after_ms,
            "investor": _str(item.get("investor"), where + ".investor").strip(),
            "metric": metric,
            "op": op,
            "value": _finite(item.get("value"), where + ".value"),
        })
    return out


def _norm_clock(raw: Any) -> Dict[str, Any]:
    if raw is None:
        return {}
    if not isinstance(raw, dict):
        raise ScenarioError("clock 必须是映射")
    out: Dict[str, Any] = {}
    if "time_scale" in raw and raw["time_scale"] is not None:
        ts = _num(raw["time_scale"], "clock.time_scale")
        if ts < 0:
            raise ScenarioError("clock.time_scale 不能为负（0 = 尽快）")
        out["time_scale"] = ts
    if "start" in raw and raw["start"] is not None:
        out["start_ms"] = parse_time_ms(_str(raw["start"], "clock.start"))
    return out


def _norm_positions(raw: Any, account: str, trading_day: Optional[str] = None) -> List[Dict[str, Any]]:
    if not isinstance(raw, list):
        raise ScenarioError("accounts[%s].positions 必须是列表" % account)
    out: List[Dict[str, Any]] = []
    seen = set()
    required = ("instrument", "exchange", "direction", "open_date", "trade_id", "open_price", "volume", "pre_settlement")
    for i, item in enumerate(raw):
        where = "accounts[%s].positions[%d]" % (account, i)
        if not isinstance(item, dict):
            raise ScenarioError("%s 必须是映射" % where)
        missing = [k for k in required if k not in item]
        if missing:
            raise ScenarioError("%s 缺少: %s" % (where, ", ".join(missing)))
        vals = {}
        for k in ("instrument", "exchange", "direction", "open_date", "trade_id"):
            if not isinstance(item[k], str) or "\x00" in item[k]:
                raise ScenarioError("%s.%s 必须是无 NUL 的字符串" % (where, k))
            vals[k] = item[k]
        if not vals["instrument"] or not vals["exchange"] or not vals["trade_id"]:
            raise ScenarioError("%s 的 instrument/exchange/trade_id 不能为空" % where)
        if vals["direction"] not in ("long", "short"):
            raise ScenarioError("%s.direction 必须是 long 或 short" % where)
        if not re.match(r"^\d{8}$", vals["open_date"]):
            raise ScenarioError("%s.open_date 必须是 YYYYMMDD" % where)
        try:
            datetime.date(int(vals["open_date"][:4]), int(vals["open_date"][4:6]), int(vals["open_date"][6:]))
        except ValueError:
            raise ScenarioError("%s.open_date 不是有效公历日期" % where)
        limits = {"instrument": 80, "exchange": 8, "open_date": 8, "trade_id": 20}
        for key, limit in limits.items():
            if len(vals[key]) > limit:
                raise ScenarioError("%s.%s 超过 CTP 字段长度 %d" % (where, key, limit))
        price = _num(item["open_price"], "%s.open_price" % where)
        volume = item["volume"]
        if isinstance(volume, bool) or not isinstance(volume, int) or volume <= 0 or volume > 2_147_483_647:
            raise ScenarioError("%s.volume 必须是 1..2147483647 的整数" % where)
        pre = _num(item["pre_settlement"], "%s.pre_settlement" % where)
        if not math.isfinite(price) or price <= 0 or not math.isfinite(pre) or pre <= 0:
            raise ScenarioError("%s.open_price/pre_settlement 必须是有限正数" % where)
        entry = dict(vals, open_price=price, volume=volume, pre_settlement=pre)
        if "margin" in item:
            margin = _num(item["margin"], "%s.margin" % where)
            if not math.isfinite(margin) or margin < 0:
                raise ScenarioError("%s.margin 必须是有限非负数" % where)
            entry["margin"] = margin
        key = tuple(entry[k] for k in ("instrument", "exchange", "direction", "open_date", "trade_id"))
        if key in seen:
            raise ScenarioError("%s 重复逐笔 key" % where)
        seen.add(key)
        out.append(entry)
    total = 0
    for entry in out:
        total += entry["volume"]
        if total > 2_147_483_647:
            raise ScenarioError("accounts[%s].positions 聚合 volume 超过 i32" % account)
    return out


def _norm_accounts(raw: Any) -> List[Dict[str, Any]]:
    if raw is None:
        return []
    if not isinstance(raw, list):
        raise ScenarioError("accounts 必须是列表")
    out: List[Dict[str, Any]] = []
    seen_accounts = set()
    for i, item in enumerate(raw):
        if not isinstance(item, dict):
            raise ScenarioError("accounts[%d] 必须是映射" % i)
        investor = _str(item.get("investor"), "accounts[%d].investor" % i).strip()
        if not investor or len(investor) > 12 or "\x00" in investor:
            raise ScenarioError("accounts[%d].investor 必须是长度不超过12的无 NUL 字符串" % i)
        if investor in seen_accounts:
            raise ScenarioError("accounts[%d].investor 重复" % i)
        seen_accounts.add(investor)
        entry: Dict[str, Any] = {"investor": investor}
        if "balance" in item and item["balance"] is not None:
            bal = _num(item["balance"], "accounts[%d].balance" % i)
            if not math.isfinite(bal) or bal <= 0:
                raise ScenarioError("accounts[%d].balance 必须为有限正数" % i)
            entry["balance"] = bal
        if "positions" in item:
            entry["positions"] = _norm_positions(item.get("positions"), investor)
        out.append(entry)
    return out


def _norm_assertions(raw: Any) -> List[Dict[str, Any]]:
    if raw is None:
        return []
    if not isinstance(raw, list):
        raise ScenarioError("assertions 必须是列表")
    out: List[Dict[str, Any]] = []
    for i, item in enumerate(raw):
        if not isinstance(item, dict):
            raise ScenarioError("assertions[%d] 必须是映射" % i)
        after_ms = parse_duration_ms(item.get("after", 0))
        investor = _str(item.get("investor"), "assertions[%d].investor" % i).strip()
        expect = item.get("expect")
        if not isinstance(expect, dict) or not expect:
            raise ScenarioError("assertions[%d].expect 必须是非空映射，如 { orders_filled: \">=1\" }" % i)
        for metric, expr in expect.items():
            metric = _str(metric, "assertions[%d].expect 的键" % i)
            if metric not in KNOWN_METRICS:
                raise ScenarioError(
                    "assertions[%d]: 未知指标 %r（可用: %s）" % (i, metric, "/".join(KNOWN_METRICS))
                )
            expr_s = _str(expr, "assertions[%d].expect[%s]" % (i, metric)).strip()
            m = re.match(r"^(>=|<=|==|!=|>|<|=)\s*([-+]?\d+(?:\.\d+)?)$", expr_s)
            if not m:
                raise ScenarioError(
                    "assertions[%d].expect[%s]: 无法解析 %r（期望如 \">=1\"）" % (i, metric, expr_s)
                )
            op = "==" if m.group(1) == "=" else m.group(1)
            out.append({
                "after_ms": after_ms,
                "investor": investor,
                "metric": metric,
                "op": op,
                "value": float(m.group(2)),
            })
    return out


_TOP_LEVEL = {"name", "source", "transforms", "clock", "accounts", "assertions"}


def _norm_head(doc: Any) -> Dict[str, Any]:
    """Shared top-level checks + `name` / `source` (same in both forms)."""
    if not isinstance(doc, dict):
        raise ScenarioError("顶层必须是映射")
    unknown = set(doc) - _TOP_LEVEL
    if unknown:
        raise ScenarioError("未知顶层字段: %s（可用: name/source/transforms/clock/accounts/assertions）" % ", ".join(sorted(unknown)))
    spec: Dict[str, Any] = {"name": _str(doc.get("name") or "", "name")}
    src = doc.get("source")
    if src is not None:
        if not isinstance(src, dict):
            raise ScenarioError("source 必须是映射")
        kind = _str(src.get("kind", "csv"), "source.kind")
        if kind != "csv":
            raise ScenarioError("source.kind %r 暂不支持（M2 仅 csv）" % kind)
        path = _str(src.get("path", "ticks.csv"), "source.path")
        spec["source"] = {"kind": "csv", "path": path}
    return spec


def _norm_tail(spec: Dict[str, Any], transforms: List[Dict[str, Any]], clock: Dict[str, Any],
               accounts: List[Dict[str, Any]], assertions: List[Dict[str, Any]]) -> Dict[str, Any]:
    spec["transforms"] = transforms
    if clock:
        spec["clock"] = clock
    if accounts:
        spec["accounts"] = accounts
    if assertions:
        spec["assertions"] = assertions
    return spec


def normalize_spec(doc: Dict[str, Any]) -> Dict[str, Any]:
    """Validate a parsed scenario.yaml and normalize it to the JSON spec form."""
    spec = _norm_head(doc)
    return _norm_tail(
        spec,
        _norm_transforms(doc.get("transforms")),
        _norm_clock(doc.get("clock")),
        _norm_accounts(doc.get("accounts")),
        _norm_assertions(doc.get("assertions")),
    )


def normalize_json_spec(doc: Dict[str, Any]) -> Dict[str, Any]:
    """Validate a spec already in the normalized JSON form (`scenario.json`).

    The equivalent of `normalize_spec` for the compiled cache: same schema
    rules, but the time fields are the resolved `*_ms` numbers, not the
    authoring strings. Returns the (re-)normalized spec.
    """
    spec = _norm_head(doc)
    return _norm_tail(
        spec,
        _check_json_transforms(doc.get("transforms")),
        _check_json_clock(doc.get("clock")),
        _norm_accounts(doc.get("accounts")),
        _check_json_assertions(doc.get("assertions")),
    )


# --------------------------------------------------------------------------
# loading / compiling
# --------------------------------------------------------------------------

def _yaml_path(scenario_dir: str) -> str:
    return os.path.join(scenario_dir, "scenario.yaml")


def _json_path(scenario_dir: str) -> str:
    return os.path.join(scenario_dir, "scenario.json")


def load_scenario_spec(scenario_dir: str) -> Dict[str, Any]:
    """Load the normalized spec for a scenario dir.

    Prefers `scenario.json` when it is at least as new as `scenario.yaml`
    (compiled cache), else parses `scenario.yaml`. Returns {} when neither
    exists (legacy ticks.csv-only scenario). Both paths are validated: the
    cached json goes through `normalize_json_spec`, so a corrupt or
    hand-edited cache fails here instead of being handed to the core.
    """
    ypath, jpath = _yaml_path(scenario_dir), _json_path(scenario_dir)
    if os.path.exists(jpath) and (
        not os.path.exists(ypath) or os.path.getmtime(jpath) >= os.path.getmtime(ypath)
    ):
        with open(jpath, "r", encoding="utf-8") as f:
            try:
                doc = json.load(f)
            except ValueError as e:
                raise ScenarioError("%s: 不是合法 JSON: %s" % (jpath, e))
        if not isinstance(doc, dict):
            raise ScenarioError("%s: 顶层必须是对象" % jpath)
        try:
            return normalize_json_spec(doc)
        except ScenarioError as e:
            raise ScenarioError("%s: %s" % (jpath, e))
    if os.path.exists(ypath):
        with open(ypath, "r", encoding="utf-8") as f:
            return normalize_spec(parse_yaml(f.read()))
    return {}


def compile_scenario(scenario_dir: str) -> Optional[str]:
    """Parse+validate scenario.yaml and write scenario.json next to it.

    Returns the json path, or None when the dir has no scenario.yaml.
    """
    ypath = _yaml_path(scenario_dir)
    if not os.path.exists(ypath):
        return None
    with open(ypath, "r", encoding="utf-8") as f:
        spec = normalize_spec(parse_yaml(f.read()))
    jpath = _json_path(scenario_dir)
    with open(jpath, "w", encoding="utf-8") as f:
        json.dump(spec, f, ensure_ascii=False, indent=2, sort_keys=True)
        f.write("\n")
    return jpath
