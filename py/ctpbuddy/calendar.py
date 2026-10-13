"""离线、固定版本的期货交易日历；不联网，不推断夜盘规则。"""
from __future__ import annotations

import datetime as dt
import hashlib
import json
import re
from dataclasses import dataclass
from pathlib import Path
from types import MappingProxyType
from typing import Any, Mapping, Optional, Protocol, Tuple, Union

DateLike = Union[str, dt.date]


class CalendarError(ValueError):
    """快照无效或查询超出明确覆盖范围。"""


def _sha256(value: Any) -> str:
    if not isinstance(value, str) or not re.fullmatch(r"[0-9a-f]{64}", value):
        raise CalendarError("sha256 必须是 64 位小写十六进制字符串")
    return value


class CalendarProvider(Protocol):
    def is_trading_day(self, date: DateLike) -> bool: ...
    def next_trading_day(self, date: DateLike) -> str: ...
    def night_status(self, date: DateLike, exchange: str) -> str: ...
    def night_session(self, date: DateLike, exchange: str) -> Tuple[str, str]: ...


@dataclass(frozen=True)
class CalendarDay:
    natural_date: str
    is_trading_day: bool
    trading_day: Optional[str] = None
    night_trading_day: Optional[str] = None
    night_action_day: Optional[str] = None
    exchange: Optional[str] = None
    night_status: Optional[str] = None


def _date(value: Any, field: str, compact: bool = False) -> str:
    if isinstance(value, dt.datetime):
        raise CalendarError("%s 必须是日期，不接受时间戳" % field)
    if isinstance(value, dt.date):
        return value.isoformat()
    if not isinstance(value, str):
        raise CalendarError("%s 必须是日期字符串" % field)
    if compact or re.fullmatch(r"[0-9]{8}", value):
        if not re.fullmatch(r"[0-9]{8}", value):
            raise CalendarError("%s 必须是 YYYYMMDD" % field)
        value = "%s-%s-%s" % (value[:4], value[4:6], value[6:])
    if not re.fullmatch(r"[0-9]{4}-[0-9]{2}-[0-9]{2}", value):
        raise CalendarError("%s 必须是 YYYY-MM-DD" % field)
    try:
        return dt.date.fromisoformat(value).isoformat()
    except ValueError as exc:
        raise CalendarError("%s 不是有效日期" % field) from exc


def _keys(raw: Any, allowed: set, field: str) -> None:
    if not isinstance(raw, Mapping):
        raise CalendarError("%s 必须是对象" % field)
    if set(raw) - allowed:
        raise CalendarError("%s 含未知字段: %s" % (field, sorted(set(raw) - allowed)))


def _pairs(pairs: list) -> dict:
    result = {}
    for key, value in pairs:
        if key in result:
            raise CalendarError("JSON 重复字段: %s" % key)
        result[key] = value
    return result


class TradingCalendar:
    """每个 days 记录覆盖一个自然日；缺失日期不等于休市。

    exchanges 仅存用户明确供给的夜盘 ActionDay/TradingDay。覆盖按自然日
    整条替换，防止留下旧夜盘映射。SHA256 对规范化 JSON 内容计算。
    """

    SCHEMA = "ctpbuddy.trading-calendar/v1"

    def __init__(self, snapshot: Mapping[str, Any]) -> None:
        _keys(snapshot, {"schema", "version", "source", "days", "night_policy"}, "snapshot")
        if snapshot.get("schema") != self.SCHEMA:
            raise CalendarError("不支持的交易日历 schema")
        version = snapshot.get("version")
        if not isinstance(version, str):
            raise CalendarError("必须提供固定的非空 version，而非浮动分支")
        version = version.strip()
        if not version or version.lower() in {"latest", "head", "main", "master"}:
            raise CalendarError("必须提供固定的非空 version，而非浮动分支")
        source = snapshot.get("source")
        _keys(source, {"kind", "name", "revision", "license", "scope", "url"}, "source")
        for key in ("kind", "name", "revision", "license", "scope"):
            if not isinstance(source.get(key), str) or not source[key].strip():
                raise CalendarError("source.%s 必须是非空字符串" % key)
        if "url" in source and (not isinstance(source["url"], str) or not source["url"].strip()):
            raise CalendarError("source.url 必须是非空字符串")
        if source["kind"] not in {"fixture", "user", "github", "official"}:
            raise CalendarError("source.kind 必须是 fixture/user/github/official")
        if source["kind"] == "official" and "url" not in source:
            raise CalendarError("official 来源必须提供 source.url")
        if source["scope"] != "futures":
            raise CalendarError("仅接受经过用户校验的 futures 快照，不能直接使用股票日历")
        if source["kind"] == "github" and not re.fullmatch(r"[0-9a-f]{40}", source["revision"]):
            raise CalendarError("GitHub 数据源必须固定 40 位 commit SHA")
        rows = snapshot.get("days")
        if not isinstance(rows, list) or not rows:
            raise CalendarError("days 必须是非空数组")
        self._days = {}
        self._nights = {}
        for row in rows:
            _keys(row, {"date", "is_trading_day", "trading_day", "exchanges"}, "day")
            date = _date(row.get("date"), "date")
            if date in self._days:
                raise CalendarError("重复自然日: %s" % date)
            opened = row.get("is_trading_day")
            if type(opened) is not bool:
                raise CalendarError("is_trading_day 必须是布尔值")
            trading = row.get("trading_day")
            if opened:
                if _date(trading, "trading_day", True) != date:
                    raise CalendarError("日盘 trading_day 必须与自然日一致")
            elif trading is not None:
                raise CalendarError("休市自然日不能设置日盘 trading_day")
            self._days[date] = CalendarDay(date, opened, trading)
            exchanges = row.get("exchanges", {})
            if not isinstance(exchanges, Mapping):
                raise CalendarError("exchanges 必须是对象")
            for exchange, night in exchanges.items():
                if not isinstance(exchange, str) or not exchange.strip():
                    raise CalendarError("exchange 必须是非空字符串")
                _keys(night, {"status", "night_action_day", "night_trading_day"}, "night")
                status = night.get("status", "open")
                if status not in {"open", "closed"}:
                    raise CalendarError("night.status 必须是 open/closed")
                if status == "closed":
                    if night.get("night_action_day") is not None or night.get("night_trading_day") is not None:
                        raise CalendarError("关闭夜盘不能设置 ActionDay/TradingDay")
                    self._nights[(date, exchange)] = CalendarDay(date, opened, trading, exchange=exchange, night_status=status)
                    continue
                action = _date(night.get("night_action_day"), "night_action_day")
                night_trading = _date(night.get("night_trading_day"), "night_trading_day", True)
                self._nights[(date, exchange)] = CalendarDay(date, opened, trading, night_trading.replace("-", ""), action.replace("-", ""), exchange, status)
        policy = snapshot.get("night_policy")
        self._night_policy = None
        if policy is not None:
            _keys(policy, {"kind", "source_revision", "scope", "exchanges", "closed_dates", "mappings"}, "night_policy")
            if policy.get("kind") != "official-preholiday-closure":
                raise CalendarError("night_policy.kind 不受支持")
            if policy.get("scope") not in {"status-only", "status-and-mapping"}:
                raise CalendarError("night_policy.scope 必须是 status-only 或 status-and-mapping")
            for field in ("source_revision",):
                if not isinstance(policy.get(field), str) or not policy[field].strip():
                    raise CalendarError("night_policy.%s 必须是非空字符串" % field)
            exchanges = policy.get("exchanges")
            if not isinstance(exchanges, list) or not exchanges or any(not isinstance(item, str) or not item.strip() for item in exchanges):
                raise CalendarError("night_policy.exchanges 必须是非空字符串数组")
            closed_dates = policy.get("closed_dates")
            if not isinstance(closed_dates, list) or any(not isinstance(item, str) for item in closed_dates):
                raise CalendarError("night_policy.closed_dates 必须是字符串数组")
            normalized_closed = tuple(_date(item, "night_policy.closed_dates") for item in closed_dates)
            if len(set(normalized_closed)) != len(normalized_closed):
                raise CalendarError("night_policy.closed_dates 不能重复")
            mappings = policy.get("mappings")
            normalized_mappings = {}
            if policy["scope"] == "status-and-mapping":
                if not isinstance(mappings, Mapping) or not mappings:
                    raise CalendarError("status-and-mapping 必须提供 night_policy.mappings")
            elif mappings is not None:
                raise CalendarError("status-only 不能提供 night_policy.mappings")
            if mappings is not None:
                for exchange, mapping in mappings.items():
                    if not isinstance(exchange, str) or not exchange.strip():
                        raise CalendarError("night_policy.mappings 的交易所必须是非空字符串")
                    _keys(mapping, {"status", "action_day", "trading_day"}, "night_policy.mapping")
                    status = mapping.get("status", "open")
                    if status not in {"open", "not_applicable"}:
                        raise CalendarError("night_policy.mapping.status 必须是 open/not_applicable")
                    if status == "not_applicable":
                        if mapping.get("action_day") is not None or mapping.get("trading_day") is not None:
                            raise CalendarError("not_applicable 映射不能设置 ActionDay/TradingDay")
                        normalized_mappings[exchange] = MappingProxyType({"status": status})
                        continue
                    action_mode = mapping.get("action_day")
                    trading_mode = mapping.get("trading_day")
                    if action_mode not in {"natural_date", "next_trading_day"}:
                        raise CalendarError("night_policy.mapping.action_day 必须是 natural_date/next_trading_day")
                    if trading_mode not in {"same_trading_day", "next_trading_day"}:
                        raise CalendarError("night_policy.mapping.trading_day 必须是 same_trading_day/next_trading_day")
                    normalized_mappings[exchange] = MappingProxyType({
                        "status": status,
                        "action_day": action_mode,
                        "trading_day": trading_mode,
                    })
                if policy["scope"] == "status-and-mapping":
                    missing = sorted(set(exchanges) - set(normalized_mappings))
                    if missing:
                        raise CalendarError("night_policy.mappings 缺少状态覆盖交易所: %s" % missing)
            self._night_policy = MappingProxyType({
                "kind": policy["kind"],
                "source_revision": policy["source_revision"],
                "scope": policy["scope"],
                "exchanges": tuple(exchanges),
                "closed_dates": frozenset(normalized_closed),
                "mappings": MappingProxyType(normalized_mappings),
            })
        normalized = dict(snapshot)
        normalized["version"] = version
        self._json = json.dumps(normalized, ensure_ascii=False, sort_keys=True, separators=(",", ":"), allow_nan=False)
        self.sha256 = hashlib.sha256(self._json.encode("utf-8")).hexdigest()
        self.metadata = MappingProxyType({"schema": self.SCHEMA, "version": version, "source": MappingProxyType(dict(source)), "sha256": self.sha256})

    @property
    def coverage(self) -> Mapping[str, Any]:
        dates = sorted(self._days)
        expected = (dt.date.fromisoformat(dates[-1]) - dt.date.fromisoformat(dates[0])).days + 1
        return MappingProxyType({"start": dates[0], "end": dates[-1], "days": len(dates),
                                 "missing_days": expected - len(dates),
                                 "night_records": len(self._nights)})

    @classmethod
    def from_json(cls, value: Union[str, bytes, Mapping[str, Any]]) -> "TradingCalendar":
        try:
            raw = value if isinstance(value, Mapping) else json.loads(value, object_pairs_hook=_pairs)
            return cls(raw)
        except (json.JSONDecodeError, TypeError) as exc:
            raise CalendarError("交易日历 JSON 无法解析") from exc

    @classmethod
    def from_file(cls, path: Union[str, Path], expected_sha256: Optional[str] = None) -> "TradingCalendar":
        calendar = cls.from_json(Path(path).read_text(encoding="utf-8"))
        if expected_sha256 is not None and calendar.sha256 != expected_sha256:
            raise CalendarError("日历 SHA256 与固定值不符")
        return calendar

    def with_overrides(self, override: Union["TradingCalendar", Mapping[str, Any], str, Path]) -> "TradingCalendar":
        other = override if isinstance(override, TradingCalendar) else (self.from_file(override) if isinstance(override, (str, Path)) else self.from_json(override))
        base, patch = json.loads(self._json), json.loads(other._json)
        rows = {row["date"]: row for row in base["days"]}
        rows.update({row["date"]: row for row in patch["days"]})
        base["days"] = [rows[key] for key in sorted(rows)]
        base["version"] += "+" + patch["version"]
        base["source"] = patch["source"]
        return self.from_json(base)

    def day(self, date: DateLike, exchange: Optional[str] = None) -> CalendarDay:
        key = _date(date, "date")
        try:
            return self._days[key] if exchange is None else self._nights[(key, exchange)]
        except KeyError as exc:
            raise CalendarError("快照未覆盖 %s%s" % (key, " 的 %s 夜盘" % exchange if exchange else "")) from exc

    def is_trading_day(self, date: DateLike) -> bool:
        return self.day(date).is_trading_day

    def next_trading_day(self, date: DateLike) -> str:
        key = _date(date, "date", isinstance(date, str) and len(date) == 8)
        current = dt.date.fromisoformat(key)
        self.day(key)
        while current < dt.date.max:
            current += dt.timedelta(days=1)
            row = self.day(current)
            if row.is_trading_day:
                return row.trading_day
        raise CalendarError("快照中无下一交易日")

    def night_session(self, date: DateLike, exchange: str) -> Tuple[str, str]:
        """按实际自然日查询 CTP ActionDay/TradingDay，遵循逐交易所映射 contract。"""
        key = _date(date, "date")
        explicit = self._nights.get((key, exchange))
        if explicit is not None:
            if explicit.night_status == "closed":
                raise CalendarError("%s 的 %s 夜盘明确休市" % (key, exchange))
            return explicit.night_action_day, explicit.night_trading_day
        mapping = self._policy_mapping(exchange)
        if mapping.get("status") == "not_applicable":
            raise CalendarError("%s 没有 %s 夜盘" % (exchange, key))
        self._policy_night_status(key, exchange, mapping)
        day = self.day(key)
        action = self._resolve_night_day(key, day, mapping["action_day"])
        trading = self._resolve_night_day(key, day, mapping["trading_day"])
        return action, trading

    def night_status(self, date: DateLike, exchange: str) -> str:
        """返回显式或经固定官方规则确认的夜盘状态；其余情况拒绝推断。"""
        key = _date(date, "date")
        explicit = self._nights.get((key, exchange))
        if explicit is not None:
            return explicit.night_status or "open"
        mapping = self._policy_mapping(exchange)
        if mapping.get("status") == "not_applicable":
            return "not_applicable"
        day = self.day(key)
        if exchange in self._night_policy["exchanges"] and day.is_trading_day:
            return "closed" if key in self._night_policy["closed_dates"] else "open"
        raise CalendarError("快照未覆盖 %s 的 %s 夜盘" % (key, exchange))

    def _policy_mapping(self, exchange: str) -> Mapping[str, str]:
        policy = self._night_policy
        if policy is None:
            raise CalendarError("快照未覆盖夜盘映射: %s" % exchange)
        mapping = policy["mappings"].get(exchange)
        if mapping is None:
            raise CalendarError("快照未覆盖夜盘映射: %s" % exchange)
        return mapping

    def _policy_night_status(self, key: str, exchange: str, mapping: Mapping[str, str]) -> str:
        if mapping.get("status") == "not_applicable":
            return "not_applicable"
        if exchange not in self._night_policy["exchanges"]:
            raise CalendarError("快照未覆盖 %s 的 %s 夜盘" % (key, exchange))
        day = self.day(key)
        if not day.is_trading_day:
            raise CalendarError("快照明确标记 %s 为休市，不能查询 %s 夜盘" % (key, exchange))
        if key in self._night_policy["closed_dates"]:
            raise CalendarError("%s 的 %s 夜盘明确休市" % (key, exchange))
        return "open"

    def _resolve_night_day(self, key: str, day: CalendarDay, mode: str) -> str:
        if mode == "natural_date":
            return key.replace("-", "")
        if mode == "same_trading_day":
            if not day.trading_day:
                raise CalendarError("%s 没有可用的日盘 TradingDay" % key)
            return day.trading_day
        if mode == "next_trading_day":
            return self.next_trading_day(key)
        raise CalendarError("不支持的夜盘日期映射: %s" % mode)




def validate_manifest(
    value: Union[str, bytes, Mapping[str, Any]],
    *,
    snapshot_path: Optional[Union[str, Path]] = None,
) -> Mapping[str, Any]:
    """Validate a production calendar manifest and its snapshot provenance.

    A manifest is an audit record, not a replacement for the calendar schema.
    When ``snapshot_path`` is supplied, the referenced snapshot must be present
    in the manifest and its canonical SHA256/coverage must match exactly.
    """
    try:
        raw = value if isinstance(value, Mapping) else json.loads(value, object_pairs_hook=_pairs)
    except (json.JSONDecodeError, TypeError) as exc:
        raise CalendarError("交易日历 manifest JSON 无法解析") from exc
    _keys(raw, {
        "schema", "version", "created_at", "scope", "runtime", "status",
        "coverage", "day_session_basis", "snapshots", "not_authoritative", "gaps", "research",
    }, "manifest")
    if raw.get("schema") != "ctpbuddy.trading-calendar-manifest/v1":
        raise CalendarError("不支持的交易日历 manifest schema")
    for field in ("version", "created_at", "scope", "runtime", "status"):
        if not isinstance(raw.get(field), str) or not raw[field].strip():
            raise CalendarError("manifest.%s 必须是非空字符串" % field)
    if raw["scope"] != "futures":
        raise CalendarError("manifest.scope 必须是 futures")
    if raw["runtime"] != "offline-only":
        raise CalendarError("manifest.runtime 必须是 offline-only")
    if raw["status"] not in {"complete", "complete-day-session-partial-night", "complete-day-session-complete-night-contract", "partial"}:
        raise CalendarError("manifest.status 不受支持")
    coverage = raw.get("coverage")
    _keys(coverage, {"start", "end", "complete_range", "scope", "night_complete"}, "manifest.coverage")
    start = _date(coverage.get("start"), "manifest.coverage.start")
    end = _date(coverage.get("end"), "manifest.coverage.end")
    if start > end:
        raise CalendarError("manifest.coverage.start 不能晚于 end")
    if type(coverage.get("complete_range")) is not bool or type(coverage.get("night_complete")) is not bool:
        raise CalendarError("manifest.coverage.complete_range/night_complete 必须是布尔值")
    if coverage.get("scope") != "day-session-only":
        raise CalendarError("manifest.coverage.scope 必须是 day-session-only")
    basis = raw.get("day_session_basis")
    if basis is not None:
        _keys(basis, {"kind", "canonical_exchange", "scope", "exchanges"}, "manifest.day_session_basis")
        if basis.get("kind") != "uniform-six-exchange":
            raise CalendarError("manifest.day_session_basis.kind 必须是 uniform-six-exchange")
        if basis.get("canonical_exchange") != "SHFE":
            raise CalendarError("manifest.day_session_basis.canonical_exchange 必须是 SHFE")
        if basis.get("scope") != "day-session-only":
            raise CalendarError("manifest.day_session_basis.scope 必须是 day-session-only")
        if basis.get("exchanges") != ["SHFE", "INE", "CZCE", "DCE", "CFFEX", "GFEX"]:
            raise CalendarError("manifest.day_session_basis.exchanges 必须列出六所固定顺序")
    snapshots = raw.get("snapshots")
    if not isinstance(snapshots, list) or not snapshots:
        raise CalendarError("manifest.snapshots 必须是非空数组")
    normalized_snapshots = []
    for index, entry in enumerate(snapshots):
        field = "manifest.snapshots[%d]" % index
        _keys(entry, {"path", "sha256", "source_url", "source_revision", "sources", "facts_sha256", "authority", "scope"}, field)
        if not isinstance(entry.get("path"), str) or not entry["path"].strip():
            raise CalendarError("%s.path 必须是非空字符串" % field)
        _sha256(entry.get("sha256"))
        for key in ("source_revision", "authority", "scope"):
            if not isinstance(entry.get(key), str) or not entry[key].strip():
                raise CalendarError("%s.%s 必须是非空字符串" % (field, key))
        if "source_url" in entry and (not isinstance(entry["source_url"], str) or not entry["source_url"].strip()):
            raise CalendarError("%s.source_url 必须是非空字符串" % field)
        if "facts_sha256" in entry:
            _sha256(entry["facts_sha256"])
        if "sources" in entry:
            if not isinstance(entry["sources"], list) or not entry["sources"]:
                raise CalendarError("%s.sources 必须是非空数组" % field)
            for source_index, source in enumerate(entry["sources"]):
                source_field = "%s.sources[%d]" % (field, source_index)
                _keys(source, {
                    "exchange", "revision", "url", "verified", "title", "published_at",
                    "notice_no", "content_sha256",
                }, source_field)
                for key in ("exchange", "revision", "url", "verified"):
                    if not isinstance(source.get(key), str) or not source[key].strip():
                        raise CalendarError("%s.%s 必须是非空字符串" % (source_field, key))
                for key in ("title", "published_at", "notice_no"):
                    if key in source and (not isinstance(source[key], str) or not source[key].strip()):
                        raise CalendarError("%s.%s 必须是非空字符串" % (source_field, key))
                if "content_sha256" in source:
                    _sha256(source["content_sha256"])
        normalized_snapshots.append(entry)
    if "gaps" in raw:
        if not isinstance(raw["gaps"], list) or any(not isinstance(item, str) or not item.strip() for item in raw["gaps"]):
            raise CalendarError("manifest.gaps 必须是字符串数组")
    if "not_authoritative" in raw and not isinstance(raw["not_authoritative"], list):
        raise CalendarError("manifest.not_authoritative 必须是数组")
    if "research" in raw and (not isinstance(raw["research"], str) or not raw["research"].strip()):
        raise CalendarError("manifest.research 必须是非空字符串")

    if snapshot_path is not None:
        path = Path(snapshot_path)
        calendar = TradingCalendar.from_file(path)
        try:
            entry = next(item for item in normalized_snapshots if item["path"] == path.name)
        except StopIteration as exc:
            raise CalendarError("manifest 未登记快照: %s" % path.name) from exc
        if entry["sha256"] != calendar.sha256:
            raise CalendarError("manifest 快照 SHA256 不匹配: %s" % path.name)
        actual = calendar.coverage
        if actual["start"] != start or actual["end"] != end:
            raise CalendarError("manifest.coverage 与快照覆盖范围不匹配")
        if bool(coverage["complete_range"]) != (actual["missing_days"] == 0):
            raise CalendarError("manifest.complete_range 与快照缺失自然日不匹配")
        if bool(coverage["night_complete"]) and actual["night_records"] == 0:
            raise CalendarError("manifest.night_complete 不能在没有夜盘记录时为 true")
    return MappingProxyType(dict(raw))


__all__ = ["CalendarProvider", "CalendarDay", "CalendarError", "TradingCalendar", "validate_manifest"]
