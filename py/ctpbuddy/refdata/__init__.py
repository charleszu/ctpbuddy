"""Reference-data (合约与费率) provider plugins.

Market data (行情) and reference data (参考数据) are the same kind of problem
from the core's point of view: the Rust counter must not own authoritative
values, it must consume a normalized file someone else produced. A trading
desk already *has* authoritative ref data — the exchange's daily settlement
parameter file, the broker's rate export, a Parquet dump of last week's
queries — so CTPBuddy does not invent numbers. It defines the shape and lets
the user feed it.

The shapes are **not** invented here. Each record mirrors the CTP query
response struct that a real client would receive for the same information
(notes/04 G):

===================  ==========================================
provider method      CTP query it mirrors
===================  ==========================================
instruments()        ReqQryInstrument
margin_rates()       ReqQryInstrumentMarginRate
commission_rates()   ReqQryInstrumentCommissionRate
order_comm_rates()   ReqQryInstrumentOrderCommRate (申报费)
trading_params()     ReqQryBrokerTradingParams (MarginPriceType)
===================  ==========================================

Field names are the snake_case of the CTP field names, so
``LongMarginRatioByMoney`` -> ``long_margin_ratio_by_money`` and a row read
here can be traced back to the header it came from without a mapping table.
Values use the CTP-native encoding where one exists (``exchange_id`` is
``SHFE``, ``hedge_flag`` is ``b'1'``) — the provider's job is to translate a
vendor export, not to re-invent CTP's own vocabulary.

A provider implements any subset of the five methods; the core treats a
missing table as "this desk has no such rule" (margin 0 is a legitimate
answer) rather than as an error.

Built-in providers:
- :func:`csv_dir` — reads a directory of CSVs, one per table
- :func:`jsonl_dir` — reads the same tables as JSONL (the canonical
  interchange format the Rust core consumes directly)

Write your own by subclassing :class:`RefDataProvider`, then point the
scenario at it::

    # refdata.yaml
    ref_data:
      kind: plugin
      provider: my_pkg.my_mod:MyProvider
      options: {path: /data/exchange/20260313}
"""
from __future__ import annotations

import csv
import json
import math
import os
from typing import Any, Dict, Iterable, Iterator, List, Optional

# --- canonical table names (one JSONL file per table) ------------------------
INSTRUMENTS = "instruments.jsonl"
MARGIN_RATES = "margin_rates.jsonl"
COMMISSION_RATES = "commission_rates.jsonl"
ORDER_COMM_RATES = "order_comm_rates.jsonl"
TRADING_PARAMS = "trading_params.jsonl"

ALL_TABLES = (
    INSTRUMENTS,
    MARGIN_RATES,
    COMMISSION_RATES,
    ORDER_COMM_RATES,
    TRADING_PARAMS,
)

# --- field orders (also the CSV header order) -------------------------------
# Mirrors the CTP response structs field-by-field, in struct order, so the
# correspondence stays checkable by eye against the SDK headers.

INSTRUMENT_FIELDS = [
    "instrument_id",          # InstrumentID
    "exchange_id",            # ExchangeID
    "exchange_inst_id",       # ExchangeInstID
    "instrument_name",        # InstrumentName
    "product_id",             # ProductID
    "product_class",          # ProductClass   '1' future '5' option
    "delivery_year",          # DeliveryYear
    "delivery_month",         # DeliveryMonth
    "create_date",            # CreateDate
    "open_date",              # OpenDate
    "expire_date",            # ExpireDate
    "start_deliv_date",       # StartDelivDate
    "end_deliv_date",         # EndDelivDate
    "inst_life_phase",        # InstLifePhase
    "is_trading",             # IsTrading
    "position_type",          # PositionType
    "position_date_type",     # PositionDateType
    "volume_multiple",        # VolumeMultiple
    "price_tick",             # PriceTick
    "min_limit_order_volume",   # MinLimitOrderVolume
    "max_limit_order_volume",   # MaxLimitOrderVolume
    "min_market_order_volume",  # MinMarketOrderVolume
    "max_market_order_volume",  # MaxMarketOrderVolume
    "long_margin_ratio",      # LongMarginRatio   (交易所率，仅展示)
    "short_margin_ratio",     # ShortMarginRatio  (交易所率，仅展示)
    "max_margin_side_algorithm",  # MaxMarginSideAlgorithm 大单边标志
    "options_type",           # OptionsType
    "underlying_multiple",    # UnderlyingMultiple
    "combination_type",       # CombinationType
]

MARGIN_RATE_FIELDS = [
    "broker_id",              # BrokerID
    "investor_id",            # InvestorID
    "hedge_flag",             # HedgeFlag   '1' 投机 '2' 套保 '3' 套利
    "exchange_id",            # ExchangeID
    "instrument_id",          # InstrumentID
    "invest_unit_id",         # InvestUnitID
    "investor_range",         # InvestorRange
    "is_relative",            # IsRelative
    # 公司保证金率 —— 实际冻结/占用计算采用这一组（notes/04 C3）
    "long_margin_ratio_by_money",   # LongMarginRatioByMoney
    "long_margin_ratio_by_volume",  # LongMarginRatioByVolume
    "short_margin_ratio_by_money",  # ShortMarginRatioByMoney
    "short_margin_ratio_by_volume", # ShortMarginRatioByVolume
]

COMMISSION_RATE_FIELDS = [
    "broker_id",              # BrokerID
    "investor_id",            # InvestorID
    "exchange_id",            # ExchangeID
    "instrument_id",          # InstrumentID
    "invest_unit_id",         # InvestUnitID
    "investor_range",         # InvestorRange
    "biz_type",               # BizType
    "open_ratio_by_money",    # OpenRatioByMoney
    "open_ratio_by_volume",   # OpenRatioByVolume
    "close_ratio_by_money",   # CloseRatioByMoney
    "close_ratio_by_volume",  # CloseRatioByVolume
    # 平今费率与平昨费率分开：即使大商所也严格区分，统一用平昨会有较大
    # 偏差（notes/04 B3 / D1）
    "close_today_ratio_by_money",  # CloseTodayRatioByMoney
    "close_today_ratio_by_volume", # CloseTodayRatioByVolume
]

ORDER_COMM_RATE_FIELDS = [
    "broker_id",              # BrokerID
    "investor_id",            # InvestorID
    "hedge_flag",             # HedgeFlag
    "exchange_id",            # ExchangeID
    "instrument_id",          # InstrumentID
    "invest_unit_id",         # InvestUnitID
    "investor_range",         # InvestorRange
    # 申报费：报单 + 撤单各一笔（notes/04 D2）。哪一交易所有这张表由数据决定，
    # 核心不按交易所写死白名单。
    "order_comm_by_volume",        # OrderCommByVolume
    "order_action_comm_by_volume", # OrderActionCommByVolume
    "order_comm_by_trade",         # OrderCommByTrade
    "order_action_comm_by_trade",  # OrderActionCommByTrade
]

# BrokerTradingParams is a single row per broker (not per instrument), so it
# has no *_FIELDS list — the core reads the whole object.
TRADING_PARAM_FIELDS = [
    "broker_id",                   # BrokerID
    "investor_id",                 # InvestorID
    "account_id",                  # AccountID
    "currency_id",                 # CurrencyID
    "margin_price_type",           # MarginPriceType
    "algorithm",                   # Algorithm
    "avail_include_close_profit",  # AvailIncludeCloseProfit
    "option_royalty_price_type",   # OptionRoyaltyPriceType
]

#: MarginPriceType 合法值（notes/04 C2）。昨仓恒用昨结算价，只有今仓受它影响。
MARGIN_PRICE_TYPES = {
    "1": "昨结算价",
    "2": "最新价",
    "3": "成交均价",
    "4": "开仓价",
}

#: 默认值：官方口径的昨结算价（'1'）——与「昨仓恒用昨结算价」自洽，且在
#: 最新价波动时不产生今仓保证金抖动，是最不容易让下游误判的缺省。
DEFAULT_MARGIN_PRICE_TYPE = "1"

#: 默认的 broker / investor 填值。真实 CTP 的费率查询要求 BrokerID +
#: InvestorID (+ HedgeFlag) 必填，"不填的话返回值就为空"（notes/04 G1）。
DEFAULT_BROKER_ID = "9999"
DEFAULT_INVESTOR_ID = "001"

#: 投机标志 THOST_FTDC_HF_Speculation —— 费率查询的必填 HedgeFlag。
HEDGE_FLAG_SPECULATION = "1"


class RefDataProvider:
    """Base class: implement the five `*_table` methods you have data for.

    Every method returns an iterable of dicts keyed by the snake_case field
    names above. Returning an empty iterable is a valid answer — it means
    "this desk has no such rule" (e.g. nobody charges 申报费 there), and the core
    will answer the corresponding query with an empty stream.
    """

    def instruments(self) -> Iterable[Dict[str, Any]]:
        """ReqQryInstrument rows. The only required table."""
        return []

    def margin_rates(self) -> Iterable[Dict[str, Any]]:
        """ReqQryInstrumentMarginRate rows (公司保证金率)."""
        return []

    def commission_rates(self) -> Iterable[Dict[str, Any]]:
        """ReqQryInstrumentCommissionRate rows."""
        return []

    def order_comm_rates(self) -> Iterable[Dict[str, Any]]:
        """ReqQryInstrumentOrderCommRate rows (申报费：报单 + 撤单各一笔)."""
        return []

    def trading_params(self) -> Optional[Dict[str, Any]]:
        """ReqQryBrokerTradingParams — one row, or None for the default."""
        return None


class DictProvider(RefDataProvider):
    """Provider over already-materialized lists (tests, small desks)."""

    def __init__(
        self,
        instruments: Optional[List[Dict[str, Any]]] = None,
        margin_rates: Optional[List[Dict[str, Any]]] = None,
        commission_rates: Optional[List[Dict[str, Any]]] = None,
        order_comm_rates: Optional[List[Dict[str, Any]]] = None,
        trading_params: Optional[Dict[str, Any]] = None,
    ) -> None:
        self._instruments = instruments or []
        self._margin_rates = margin_rates or []
        self._commission_rates = commission_rates or []
        self._order_comm_rates = order_comm_rates or []
        self._trading_params = trading_params

    def instruments(self):
        return self._instruments

    def margin_rates(self):
        return self._margin_rates

    def commission_rates(self):
        return self._commission_rates

    def order_comm_rates(self):
        return self._order_comm_rates

    def trading_params(self):
        return self._trading_params


class JsonlDirProvider(RefDataProvider):
    """Canonical JSONL directory — the format the Rust core reads directly.

    This is both a provider and the interchange format: `ctpbuddy refdata
    export` writes one of these from any other provider, and the core
    consumes it with no Python in the loop (same split as the market-data
    plugin protocol, DESIGN §7.3).
    """

    def __init__(self, path: str) -> None:
        self.path = path

    def _read(self, name: str) -> List[Dict[str, Any]]:
        full = os.path.join(self.path, name)
        if not os.path.exists(full):
            return []
        rows: List[Dict[str, Any]] = []
        with open(full, "r", encoding="utf-8") as f:
            for lineno, line in enumerate(f, start=1):
                line = line.strip()
                if not line or line.startswith("#"):
                    continue
                try:
                    rows.append(json.loads(line))
                except ValueError as e:
                    raise ValueError("%s line %d: %s" % (full, lineno, e)) from None
        return rows

    def instruments(self):
        return self._read(INSTRUMENTS)

    def margin_rates(self):
        return self._read(MARGIN_RATES)

    def commission_rates(self):
        return self._read(COMMISSION_RATES)

    def order_comm_rates(self):
        return self._read(ORDER_COMM_RATES)

    def trading_params(self):
        rows = self._read(TRADING_PARAMS)
        return rows[0] if rows else None


class CsvDirProvider(RefDataProvider):
    """A directory of CSVs, one per table, header = snake_case field names.

    Convenient when the source is a spreadsheet export (every desk has one).
    A table whose file is absent yields an empty table, so a directory with
    just `instruments.csv` is a valid (if unpriced) provider.
    """

    def __init__(self, path: str) -> None:
        self.path = path

    def _read(self, stem: str) -> List[Dict[str, Any]]:
        full = os.path.join(self.path, stem + ".csv")
        if not os.path.exists(full):
            return []
        with open(full, "r", encoding="utf-8-sig", newline="") as f:
            return [
                {(k or "").strip(): (v or "").strip() for k, v in row.items()}
                for row in csv.DictReader(f)
                if any((v or "").strip() for v in row.values())
            ]

    def instruments(self):
        return self._read("instruments")

    def margin_rates(self):
        return self._read("margin_rates")

    def commission_rates(self):
        return self._read("commission_rates")

    def order_comm_rates(self):
        return self._read("order_comm_rates")

    def trading_params(self):
        rows = self._read("trading_params")
        return rows[0] if rows else None


def csv_dir(path: str) -> RefDataProvider:
    return CsvDirProvider(path)


def jsonl_dir(path: str) -> RefDataProvider:
    return JsonlDirProvider(path)


def load_provider(spec: Dict[str, Any]) -> RefDataProvider:
    """Resolve a `ref_data:` block into a provider instance.

    ``kind: csv`` / ``kind: jsonl`` are the built-ins; ``kind: plugin`` names
    a Python object as ``package.module:Attribute`` (a class to instantiate
    with no args, or a zero-arg factory returning a provider). Options are
    passed as keyword arguments when the target accepts them.
    """
    kind = (spec.get("kind") or "csv").strip()
    opts = dict(spec.get("options") or {})
    if "path" in spec:
        opts.setdefault("path", spec["path"])

    if kind == "csv":
        if "path" not in opts:
            raise ValueError("ref_data kind=csv 需要 path")
        return CsvDirProvider(opts["path"])
    if kind == "jsonl":
        if "path" not in opts:
            raise ValueError("ref_data kind=jsonl 需要 path")
        return JsonlDirProvider(opts["path"])
    if kind == "plugin":
        target = spec.get("provider")
        if not target:
            raise ValueError("ref_data kind=plugin 需要 provider（package.module:Attribute）")
        return _instantiate(str(target), opts)
    raise ValueError("ref_data kind 未知: %r（支持 csv / jsonl / plugin）" % kind)


def _instantiate(target: str, opts: Dict[str, Any]) -> RefDataProvider:
    if ":" not in target:
        raise ValueError("provider 需写成 package.module:Attribute，实际 %r" % target)
    module_name, attr = target.split(":", 1)
    import importlib

    try:
        module = importlib.import_module(module_name)
    except ImportError as e:
        raise ValueError("provider 模块导入失败 %s: %s" % (module_name, e)) from None
    try:
        obj = getattr(module, attr)
    except AttributeError:
        raise ValueError("provider 模块 %s 没有 %s" % (module_name, attr)) from None
    if isinstance(obj, type):
        return _call_flexible(obj, opts)
    if callable(obj):
        return _call_flexible(obj, opts)
    if isinstance(obj, RefDataProvider):
        return obj
    raise ValueError("provider %s 既不是类也不是可调用对象" % target)


def _call_flexible(target: Any, opts: Dict[str, Any]) -> RefDataProvider:
    """Instantiate a provider class / call a provider factory with `opts`.

    Every option is passed as a keyword. A provider that wants fewer just
    declares fewer parameters; one that wants anything declares ``**kwargs``.
    A provider whose parameter names do not match the option keys raises
    ``TypeError`` naming the mismatch — deliberately not swallowed, because
    retrying without arguments would yield an empty-but-valid dataset, the
    worst possible failure mode for reference data.
    """
    return target(**opts)


# --- validation --------------------------------------------------------------

def _check_rows(rows: Iterable[Dict[str, Any]], fields: List[str], table: str) -> List[str]:
    problems: List[str] = []
    seen = set()
    for i, row in enumerate(rows, start=1):
        for key in ("instrument_id",):
            if key in fields and not str(row.get(key, "")).strip():
                problems.append("%s line %d: 缺少 %s" % (table, i, key))
        iid = str(row.get("instrument_id", "")).strip()
        if iid and iid in seen:
            problems.append("%s line %d: instrument_id 重复 %r" % (table, i, iid))
        seen.add(iid)
        # a numeric field the core will parse as a number must be one here:
        # writing 0 for "abc" would silently turn a typo into a zero margin
        for k, v in row.items():
            if k in _NUMERIC and not _is_blank(v):
                try:
                    _to_number(v)
                except (TypeError, ValueError):
                    problems.append("%s line %d: %s=%r 不是数字" % (table, i, k, v))
    return problems


def _table(provider: Any, name: str) -> List[Dict[str, Any]]:
    """Read one table off a provider, tolerating a partial implementation.

    Providers are duck-typed on purpose: a desk that only has contract and
    margin data should not have to write three methods returning `[]`. A
    missing method means "this desk has no such rule", which is a legitimate
    answer (a desk charging no 申报费), not an error.
    """
    fn = getattr(provider, name, None)
    if fn is None:
        return []
    result = fn()
    return list(result) if result else []


def _trading_params(provider: Any) -> Optional[Dict[str, Any]]:
    fn = getattr(provider, "trading_params", None)
    return fn() if fn is not None else None


def validate(provider: RefDataProvider) -> List[str]:
    """Return a list of problems; empty means the core can load this."""
    problems: List[str] = []
    instruments = _table(provider, "instruments")
    if not instruments:
        problems.append("instruments 表为空（这是唯一的必填表）")
    problems += _check_rows(instruments, INSTRUMENT_FIELDS, INSTRUMENTS)
    for rows, fields, name in (
        (_table(provider, "margin_rates"), MARGIN_RATE_FIELDS, MARGIN_RATES),
        (_table(provider, "commission_rates"), COMMISSION_RATE_FIELDS, COMMISSION_RATES),
        (_table(provider, "order_comm_rates"), ORDER_COMM_RATE_FIELDS, ORDER_COMM_RATES),
    ):
        problems += _check_rows(rows, fields, name)
    tp = _trading_params(provider)
    if tp is not None:
        mpt = str(tp.get("margin_price_type", DEFAULT_MARGIN_PRICE_TYPE) or DEFAULT_MARGIN_PRICE_TYPE)
        if mpt not in MARGIN_PRICE_TYPES:
            problems.append(
                "trading_params.margin_price_type=%r 非法（合法值 %s）"
                % (mpt, "/".join(sorted(MARGIN_PRICE_TYPES)))
            )
    return problems


def write_jsonl(provider: RefDataProvider, out_dir: str) -> List[str]:
    """Materialize a provider to the canonical JSONL directory.

    Returns the list of files written. Values are normalized to JSON-native
    types (numbers as numbers, empty strings dropped) so the Rust side parses
    exactly one shape.
    """
    problems = validate(provider)
    if problems:
        raise ValueError("ref data 校验失败:\n  " + "\n  ".join(problems))
    os.makedirs(out_dir, exist_ok=True)
    written: List[str] = []

    def dump(name: str, rows: Iterable[Dict[str, Any]]) -> None:
        full = os.path.join(out_dir, name)
        with open(full, "w", encoding="utf-8", newline="\n") as f:
            for i, row in enumerate(rows, start=1):
                f.write(json.dumps(_normalize(row, "%s line %d" % (name, i)),
                                   ensure_ascii=False, sort_keys=True) + "\n")
        written.append(full)

    dump(INSTRUMENTS, _table(provider, "instruments"))
    dump(MARGIN_RATES, _table(provider, "margin_rates"))
    dump(COMMISSION_RATES, _table(provider, "commission_rates"))
    dump(ORDER_COMM_RATES, _table(provider, "order_comm_rates"))
    tp = _trading_params(provider)
    dump(TRADING_PARAMS, [tp] if tp else [])
    return written


_NUMERIC = {
    "delivery_year", "delivery_month", "volume_multiple",
    "min_limit_order_volume", "max_limit_order_volume",
    "min_market_order_volume", "max_market_order_volume",
    "price_tick", "long_margin_ratio", "short_margin_ratio",
    "underlying_multiple", "is_relative", "is_trading",
    "long_margin_ratio_by_money", "long_margin_ratio_by_volume",
    "short_margin_ratio_by_money", "short_margin_ratio_by_volume",
    "open_ratio_by_money", "open_ratio_by_volume",
    "close_ratio_by_money", "close_ratio_by_volume",
    "close_today_ratio_by_money", "close_today_ratio_by_volume",
    "order_comm_by_volume", "order_action_comm_by_volume",
    "order_comm_by_trade", "order_action_comm_by_trade",
}


def _is_blank(v: Any) -> bool:
    return v is None or (isinstance(v, str) and not v.strip())


def _to_number(v: Any) -> Any:
    """Strict numeric coercion for `_NUMERIC` fields (int when integral text).

    Raises TypeError / ValueError on anything that is not a number: bools,
    non-numeric text, NaN / inf (the core's JSON has no such values).
    """
    if isinstance(v, bool):
        raise TypeError("bool is not a number")
    if isinstance(v, (int, float)):
        n: Any = v
    else:
        s = str(v).strip()
        n = float(s) if "." in s or "e" in s.lower() else int(s)
    if isinstance(n, float) and not math.isfinite(n):
        raise ValueError("non-finite number")
    return n


def _normalize(row: Dict[str, Any], where: str = "") -> Dict[str, Any]:
    """CSV hands back strings; JSON wants numbers where CTP has numbers.

    A numeric field that does not parse raises ValueError naming the
    row/field (`where` is a "table line N" prefix) — never a silent 0.
    """
    out: Dict[str, Any] = {}
    for k, v in row.items():
        if _is_blank(v):
            continue
        if k in _NUMERIC:
            try:
                out[k] = _to_number(v)
            except (TypeError, ValueError):
                raise ValueError("%s%s=%r 不是数字" % (where + ": " if where else "", k, v)) from None
        else:
            out[k] = v
    return out


def iter_tables(provider: RefDataProvider) -> Iterator[tuple]:
    """(table name, rows) for every table — used by the CLI's report mode."""
    yield INSTRUMENTS, _table(provider, "instruments")
    yield MARGIN_RATES, _table(provider, "margin_rates")
    yield COMMISSION_RATES, _table(provider, "commission_rates")
    yield ORDER_COMM_RATES, _table(provider, "order_comm_rates")
    tp = _trading_params(provider)
    yield TRADING_PARAMS, [tp] if tp else []
