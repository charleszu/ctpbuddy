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


class CalendarProvider(Protocol):
    def is_trading_day(self, date: DateLike) -> bool: ...
    def next_trading_day(self, date: DateLike) -> str: ...
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
        _keys(snapshot, {"schema", "version", "source", "days"}, "snapshot")
        if snapshot.get("schema") != self.SCHEMA:
            raise CalendarError("不支持的交易日历 schema")
        version = snapshot.get("version")
        if not isinstance(version, str) or not version.strip() or version.lower() in {"latest", "head", "main", "master"}:
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
                _date(night.get("night_trading_day"), "night_trading_day", True)
                self._nights[(date, exchange)] = CalendarDay(date, opened, trading, night["night_trading_day"], action.replace("-", ""), exchange, status)
        self._json = json.dumps(snapshot, ensure_ascii=False, sort_keys=True, separators=(",", ":"), allow_nan=False)
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
        """按实际自然日查询显式 CTP ActionDay/TradingDay，不做兜底推断。"""
        row = self.day(date, exchange)
        if row.night_status == "closed":
            raise CalendarError("%s 的 %s 夜盘明确休市" % (_date(date, "date"), exchange))
        return row.night_action_day, row.night_trading_day


def load_calendar(path: Union[str, Path], override: Optional[Union[str, Path, Mapping[str, Any]]] = None, expected_sha256: Optional[str] = None) -> TradingCalendar:
    calendar = TradingCalendar.from_file(path, expected_sha256)
    return calendar.with_overrides(override) if override is not None else calendar


__all__ = ["CalendarProvider", "CalendarDay", "CalendarError", "TradingCalendar", "load_calendar"]
