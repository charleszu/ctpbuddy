"""离线生成固定公告摘录的日盘快照；不联网，不推断夜盘。"""
from pathlib import Path
import datetime as dt
import hashlib
import json

ROOT = Path(__file__).parent
SOURCES = [
    {"exchange": "SHFE", "revision": "2025-12-17-[2025]157", "url": "https://www.shfe.com.cn/publicnotice/notice/202512/t20251217_829805.html", "title": "上海期货交易所关于2026年休市安排的公告", "published_at": "2025-12-17", "notice_no": "〔2025〕157号", "content_sha256": "3d502c5a3f27b93e5f15968bb732fe8f11daabc29fcd224b5d2be30ea3f2b018", "verified": "official-page"},
    {"exchange": "INE", "revision": "2025-12-17-[2025]114", "url": "https://www.ine.cn/publicnotice/notice/202512/t20251217_829804.html", "title": "上海国际能源交易中心关于2026年休市安排的公告", "published_at": "2025-12-17", "notice_no": "〔2025〕114号", "content_sha256": "6c760411d649c724b9bd7f90190a69c578d520fc78a94b8dba06fbd1d449635a", "verified": "official-page"},
    {"exchange": "CZCE", "revision": "2025-12-17-[2025]939", "url": "https://www.czce.com.cn/cn/gyjys/jysdt/ggytz/webinfo/2025/12/528a2b72a3c949ae88ee93391e1d6c0f.htm?t=aged&updatetime=short", "verified": "official-search-excerpt"},
    {"exchange": "DCE", "revision": "2026-holiday-page-extract-20261004", "url": "http://www.dce.com.cn/dce/channel/list/226.html", "verified": "official-search-excerpt; 页面注明仅做参考，以交易所公布为准；未取得固定公告正文"},
    {"exchange": "GFEX", "revision": "2025-12-17-[2025]414", "url": "https://www.gfex.com.cn/gfex/tzts/202512/0134914f8c674fa68c2ba9041a5cd798.shtml", "verified": "official-search-excerpt"},
    {"exchange": "CFFEX", "revision": "2025-12-17-[2025]55", "url": "http://www.cffex.com.cn/cn/jystz/20251217/47006.html", "title": "中国金融期货交易所关于2026年部分节假日休市安排的通知", "published_at": "2025-12-17", "notice_no": "中金所发〔2025〕55号", "content_sha256": "48a77f47e6d9ac45f33e096fc8ff121ba1f3b356e25c527fa7025018df24330e", "verified": "official-page"},
]
HOLIDAYS = [("2026-01-01", "2026-01-03"), ("2026-02-15", "2026-02-23"),
            ("2026-04-04", "2026-04-06"), ("2026-05-01", "2026-05-05"),
            ("2026-06-19", "2026-06-21"), ("2026-09-25", "2026-09-27"),
            ("2026-10-01", "2026-10-07")]
NIGHT_CLOSED = ["2026-02-13", "2026-04-03", "2026-04-30", "2026-06-18", "2026-09-24", "2026-09-30"]


def canonical(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"), allow_nan=False).encode()


def build():
    rows = []
    date = dt.date(2026, 1, 1)
    while date.year == 2026:
        natural = date.isoformat()
        opened = date.weekday() < 5 and not any(start <= natural <= end for start, end in HOLIDAYS)
        row = {"date": natural, "is_trading_day": opened}
        if opened:
            row["trading_day"] = date.strftime("%Y%m%d")
        if natural in NIGHT_CLOSED:
            row["exchanges"] = {exchange: {"status": "closed"} for exchange in ("SHFE", "INE", "CZCE", "DCE")}
        rows.append(row)
        date += dt.timedelta(days=1)
    snapshot = {"schema": "ctpbuddy.trading-calendar/v1", "version": "cn-futures-day-2026-20261013-r1",
                "source": {"kind": "official", "name": "SHFE 2026 官方日盘安排；统一六所日盘 contract",
                           "revision": "2025-12-17-[2025]157-html-sha256-3d502c5a3f27b93e5f15968bb732fe8f11daabc29fcd224b5d2be30ea3f2b018",
                           "license": "官方公告固定摘录；原网页再分发许可未单独确认",
                           "scope": "futures",
                           "url": "https://www.shfe.com.cn/publicnotice/notice/202512/t20251217_829805.html"},
                "night_policy": {
                    "kind": "official-preholiday-closure",
                    "source_revision": "2025-12-17-[2025]157-html-sha256-3d502c5a3f27b93e5f15968bb732fe8f11daabc29fcd224b5d2be30ea3f2b018",
                    "scope": "status-only",
                    "exchanges": ["SHFE", "INE", "CZCE", "DCE"],
                    "closed_dates": NIGHT_CLOSED,
                },
                "days": rows}
    sha = hashlib.sha256(canonical(snapshot)).hexdigest()
    path = ROOT / "cn-futures-day-2026.snapshot.json"
    path.write_text(json.dumps(snapshot, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    manifest = json.loads((ROOT / "MANIFEST.json").read_text(encoding="utf-8"))
    manifest.update(version="2026-10-04-r2", status="complete-day-session-partial-night",
                    coverage={"start": "2026-01-01", "end": "2026-12-31", "complete_range": True,
                              "scope": "day-session-only", "night_complete": False})
    manifest["day_session_basis"] = {
        "kind": "uniform-six-exchange",
        "canonical_exchange": "SHFE",
        "scope": "day-session-only",
        "exchanges": ["SHFE", "INE", "CZCE", "DCE", "CFFEX", "GFEX"],
    }
    manifest["snapshots"] = [manifest["snapshots"][0], {"path": path.name, "sha256": sha,
        "source_revision": "2025-12-17-[2025]157-html-sha256-3d502c5a3f27b93e5f15968bb732fe8f11daabc29fcd224b5d2be30ea3f2b018", "sources": SOURCES,
        "facts_sha256": hashlib.sha256(canonical({"sources": SOURCES, "holidays": HOLIDAYS, "night_closed": NIGHT_CLOSED})).hexdigest(),
        "authority": "SHFE 官方 2026 日盘休市安排；按统一六所日盘 contract 展开，夜盘仍不完整", "scope": "2026全年自然日对应期货日盘可交易状态；六所公开安排同日休市"}]
    manifest["gaps"] = [
        "完整365个自然日日盘状态不是完整官方逐日TradingDay/ActionDay数据库；不得把每个自然日直接当TradingDay。",
        "六所公布的节假日日盘关闭区间一致；周末不因民用调休上班而开市，不供给民用工作日日历。",
        "SHFE/INE/CZCE/DCE仅记录公告明确的六个2026夜盘关闭边界；2025-12-31保留在旧样本。其他夜盘未知，不由日盘或周末推断。",
        "night_policy 仅用于确认 SHFE/INE/CZCE/DCE 的 open/closed 状态；正常夜盘的 ActionDay/TradingDay 仍未按交易所逐日展开。",
        "GFEX/CFFEX未在本快照填充夜盘；未核验官方逐日open映射，绝不推断。",
        "SHFE/INE/CFFEX 已固定官方正文 hash；DCE 的 18631023 页面是端午风险管理通知而非全年休市安排且直连受限，CZCE/GFEX 未取得可固定的休市正文。需使用者复核后用于生产。",
        "不覆盖临时停市、品种交易时段、公告后修订、2027下一交易日；快照年末next_trading_day超范围必须失败。"]
    (ROOT / "MANIFEST.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(sha)


if __name__ == "__main__":
    build()
