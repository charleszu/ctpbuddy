# -*- coding: utf-8 -*-
"""Regenerate the CTPBuddy status column and the status summary in
docs/错误码全集.md.

Every row is classified against the *actual* code surface (core constants +
the handlers' push-surface split), not against wishful intent:

  已实现  the core really emits this code, on the stated push surface
  可落地  the semantics sit inside CTPBuddy's trading/risk scope but no code
          path emits them yet -- a registered gap, not a feature request
  暂不可达  the code belongs to a business domain CTPBuddy does not implement
          (银期转账 / 换汇 / 期权执行 / 组合 / 报价 / 监控中心 / 短信 ...), so a
          client can never observe it here

The run is a **full recompute**: every table row's status cell is rewritten
(whatever it said before), and the whole 「CTPBuddy 状态列说明」 section (counts
table, 已实现 push-surface lists, 可落地 gap table, 暂不可达 domain
distribution) is regenerated from the same data. The three tallies therefore
cannot drift from the rows again.

Run:    python tools/fill_errorcode_status.py            # rewrite the doc
Check:  python tools/fill_errorcode_status.py --check    # compare only; exit 1 on drift
"""
from __future__ import annotations

import argparse
import difflib
import os
import re
import sys
from collections import Counter, OrderedDict

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
# The gitignored SDK copy wins when present; the CHM-derived copy shipped under
# docs/api-doc-html/files/ is byte-identical in content (299 entries) and is
# what a fresh clone has.
ERROR_XML_CANDIDATES = (
    os.path.join(ROOT, "ctpsdk", "6.7.13_20260225", "td", "win64", "error.xml"),
    os.path.join(ROOT, "docs", "api-doc-html", "files", "error.xml"),
)
DOC = os.path.join(ROOT, "docs", "错误码全集.md")
EXPECTED_ENTRIES = 299

# ---------------------------------------------------------------------------
# 已实现: code -> (state, note), cross-checked against core/ctpbuddy-server
# ---------------------------------------------------------------------------
# insert CTP-layer refusals -> RSP_ERROR only (handlers::reject_insert, first arm)
# insert exchange-layer refusals -> RSP_ORDER_INSERT{0} then ERR_RTN_ORDER_INSERT
# cancel refusals -> RSP_ERROR (OnRspOrderAction) then ERR_RTN_ORDER_ACTION
IMPLEMENTED = OrderedDict([
    (0,   "成功响应恒 0（非错误）"),
    (3,   "登录 UserID 为空（`INVALID_LOGIN`，ErrorMsg `CTP:不合法的登录：UserID 为空`）；报单 BrokerID/InvestorID 与登录会话不一致同码；会话面 `OnRspUserLogin` / CTP 层 `OnRspOrderInsert`"),
    (15,  "报单字段有误 / 价格≤0 / PriceType 非法；CTP 层 · 仅 `OnRspOrderInsert`"),
    (16,  "合约不在目录；CTP 层 · 仅 `OnRspOrderInsert`"),
    (17,  "合约 `IsTrading=0`（RefData `is_trading` 字段）；CTP 层 · 仅 `OnRspOrderInsert`"),
    (22,  "同 (front, session, OrderRef) 重复报单——活动单与已终态单（`terminal_refs` 记忆）均判重；CTP 层 · 仅 `OnRspOrderInsert`"),
    (25,  "撤单找不到活动单；**双面** `OnRspOrderAction` → `OnErrRtnOrderAction`"),
    (26,  "撤单命中终态（`terminal_refs` 记忆）；**双面**"),
    (27,  "不支持的功能：GTD/GTC、非法 TimeCondition/VolumeCondition、条件单（ContingentCondition≠立即）；CTP 层 · 仅 `OnRspOrderInsert`（原误用 41）"),
    (30,  "平仓量超过今+昨持仓；CTP 层 · 仅 `OnRspOrderInsert`"),
    (31,  "可用资金 < 冻结估算额；CTP 层 · 仅 `OnRspOrderInsert`"),
    (42,  "`settlement_required=true`（默认）时当前交易日未确认结算单的报单前置拒绝；CTP 层 · 仅 `OnRspOrderInsert`"),
    (50,  "CloseToday 可用今仓不足；CTP 层 · 仅 `OnRspOrderInsert`"),
    (51,  "CloseYesterday 可用昨仓不足；CTP 层 · 仅 `OnRspOrderInsert`"),
    (60,  "`max_user_sessions` 按 BrokerID+UserID 限制在线已登录连接（默认 0 关闭）；会话面 `OnRspUserLogin`"),
    (63,  "登录 BrokerID 与核心 `--broker-id` 不一致（`AUTH_FAILED`，ErrorMsg `CTP:客户端认证失败：BrokerID 'x' 与本核心服务的不一致（8888）`）；会话面 `OnRspUserLogin`"),
    (64,  "未认证即登录（先 AUTH）——核心产出，非 shim 本地码；会话面 `OnRspUserLogin`"),
    (90,  "`ReqQry*` 每会话超预算（`qry_gate`，默认 2/s）；查询面 `OnRspQry*`"),
    (116, "每秒预算（`order_gate`，`--order-freq` 默认 20），**报单与撤单是两条独立预算流**，互不挤占；insert 仅响应面 / cancel **双面**"),
    (148, "请求 ExchangeID 与合约目录不符；CTP 层 · 仅 `OnRspOrderInsert`"),
    (163, "限价超出最新 tick 涨跌停板；**交易所层** · `OnRspOrderInsert{0}` → `OnErrRtnOrderInsert`"),
    (164, "手数 ≤0 或越所/品种 min-max；**交易所层**"),
    (165, "限价非最小变动价位整数倍；**交易所层**"),
])

# Push-surface grouping of the implemented codes (rendered as the
# 「已实现 N 条与推送面」 list). A code may sit in two groups (116: insert +
# cancel); the union must equal IMPLEMENTED, which `main` asserts.
SURFACES = [
    ("CTP 层拒绝（insert，仅 `OnRspOrderInsert`，无后续状态回报）",
     [3, 15, 16, 17, 22, 27, 148, 30, 31, 42, 50, 51, 116]),
    ("交易所层拒绝（insert，`OnRspOrderInsert{0}` → `OnErrRtnOrderInsert`）", [163, 164, 165]),
    ("撤单拒绝（双面，`OnRspOrderAction` → `OnErrRtnOrderAction`）", [25, 26, 116]),
    ("查询流控（`OnRspQry*`）", [90]),
    ("会话/登录（`OnRspUserLogin`）", [3, 60, 63, 64]),
    ("非错误", [0]),
]

# ---------------------------------------------------------------------------
# 可落地: registered gaps inside our own scope (no code path emits them yet)
# ---------------------------------------------------------------------------
ACTIONABLE = {
    2:   "会话信息不一致（撤单携带的 FrontID/SessionID 与本会话不符——当前按 OrderRef 定位，未校验）",
    4:   "用户不活跃（无用户停用态）",
    5:   "同一 UserID 重复登录（当前按会话累加 SessionID，不拒绝）",
    6:   "未登录即请求——当前统一用 API 本地 -3，未用此码",
    7:   "未 Init 完即请求——shim 侧保证，核心不判",
    8:   "前置不活跃（无前置状态机；重连即换 FrontID）",
    9:   "权限：报单/查询权限位（规则表尚未建模）",
    11:  "找不到用户：登录 UserID 未开户（当前 SimNow 式自动开户）",
    12:  "找不到经纪公司（`--broker-id` 单一，无多 Broker 表；BrokerID 不符当前回 63）",
    13:  "找不到投资者：登录 InvestorID 不存在（同上）",
    19:  "投资者不活跃（无停用/权限态）",
    20:  "投资者未在交易所开户（无开户状态字段）",
    23:  "撤单 ActionFlag 非法（当前只支持 Delete，其余归一）",
    24:  "撤单已报送未回报不允许重复撤（无在途撤单去重）",
    28:  "无报单交易权限（权限位未建模）",
    29:  "只可平仓（规则表「开仓受限」档位未注入）",
    37:  "现货卖空（无现货标的）",
    38:  "不合法的结算引用（无银行存支票）",
    43:  "没有对应的入金记录（无出入金）",
    48:  "投资者/口令无效（无口令校验与错次计数）",
    49:  "登录 IP 不合法（无 IP 白名单）",
    52:  "经纪公司可用条件单数量不足（条件单未实现，报入条件单当前回 27）",
    55:  "重发未知单经纪/投资者不匹配（无 ResendOrder 流程）",
    65:  "合约不支持互换类型报单（互换单未实现）",
    75:  "连续登录失败超限（无错次锁定）",
    77:  "无此功能：`ReqOrderAction` 挂起/激活/修改、组合/询价/报价等（FAQ #42/#13）；GTC/GTD/条件单报单当前走 27",
    78:  "发送报单失败（无交易所链路，恒不触发）",
    79:  "发送报单操作失败（同上）",
    80:  "交易所不支持的价格类型（任意价/市价归一后不判）",
    82:  "无效的组合合约（组合合约未实现）",
    89:  "打开文件失败（无流文件落盘）",
    91:  "交易所返回的错误：**常量已留**，交易所侧拒单转发尚未接线（#43 相邻）",
    94:  "不允许撤销 OTC 衍生报单（OTC 未实现）",
    95:  "不支持的价格（限价≤0 走 15）",
    100: "RULE 品种内对锁仓折扣参数（策略单未实现）",
    101: "本系统无报单权限（权限位未建模）",
    102: "无 DR 号（无灾备路由）",
    103: "不支持批量操作（无批量报单/批量撤单）",
    106: "投资者限仓（持仓限额规则未建模）",
    139: "当前时间禁止行权（期权未实现）",
    154: "查询核心忙（查询流控已用 90，无独立忙态）",
    157: "该交易所不支持撤销未知单（撤单走 25，未按所分流）",
    158: "超过信息量限制（无 FTD 报文流控）",
    162: "产品不在可交易阶段（无交易时段门禁，场景时钟只驱动行情）",
    4041: "重复提交（提交/心跳层）",
    5051: "序号字段重复（无 MaxOrderRef 重发校验）",
    6000: "订阅合约数超限（`--subscribe` 无上限）",
}

# Thematic grouping of the gaps (rendered as the 「可落地 N 条（缺口清单）」
# table). Every ACTIONABLE code must appear in exactly one theme; `main`
# asserts it so a code added above cannot silently fall out of the table.
GAP_THEMES = [
    ("会话一致性", [2], "撤单携带的 FrontID/SessionID 未与本会话校验"),
    ("登录风控", [4, 5, 8, 11, 12, 13, 19, 20, 48, 49, 75],
     "用户/经纪公司/前置状态表、重复登录拒绝、口令校验与错次锁定、IP 白名单"),
    ("未登录/未初始化", [6, 7], "当前以 API 本地 -3 / shim 侧保证代替官方码"),
    ("权限位", [9, 28, 101, 102], "报单/查询权限与灾备路由"),
    ("条件单额度", [52], "条件单数量额度（功能本身未实现，报入条件单回 27）"),
    ("撤单校验", [23, 24, 157], "ActionFlag 合法性、在途撤单去重、按所的「撤销未知单」支持矩阵"),
    ("规则表未建模档位", [29, 37, 106, 162, 95, 80],
     "只可平仓 / 现货卖空 / 限仓 / 交易时段门禁 / 价格类型支持矩阵"),
    ("未实现的报单类型", [65, 77, 82, 89, 94, 100, 103],
     "互换单、组合合约、策略单、OTC、批量操作、流文件落盘"),
    ("出入金/结算引用", [38, 43], "无银行存支票与出入金记录"),
    ("期权", [139], "期权行权时段（期权未实现）"),
    ("查询/序号/订阅", [154, 158, 5051, 6000],
     "查询核心忙独立态、FTD 报文流控、MaxOrderRef 重发校验、订阅合约数上限"),
    ("提交层", [4041], "重复提交（提交/心跳层）"),
    ("交易所链路", [78, 79, 91, 55],
     "`91 EXCHANGE_RTNERROR` 常量已留，交易所侧拒单转发未接线（#43 相邻工作）；无 ResendOrder 流程"),
]

# ---------------------------------------------------------------------------
# 暂不可达: business domains CTPBuddy deliberately does not implement
# ---------------------------------------------------------------------------
UNREACHABLE_RULES = [
    ("银期转账", r"银期|转账|冲正|SECAGENT|OFFER|预约开户|币种|汇率|签到|MAC|PIN|密钥|CURRENCYID|IdentifiedCardNo|ACCOUNT_NOT_FUND|ACCOUNT_NOT_ACTIVE|AMOUNT_OUTOFTHEWAY|CHECKD_FILE|DUP_BANK_SERIAL|BANKACCOUNT|ACCOUNTID_|BANK_SERVER_ERROR|BANK_SYSTEM_INTERNAL_ERROR|DUPLATION_BANK_SERIAL|BANK_INTERNAL_ERR|银行"),
    ("银期换汇", r"^FBE_|换汇"),
    ("建行银期业务/技术错误码", r"^(PW_|AL_|AC_|DC_|CE_|DO_|TM_|RC_|BL_|NA_|HW_|IO_|DB_|NC_|SS_|CM_|FC_|TL_|AT_)"),
    ("期权执行/行权", r"期权|EXEC|EXECUTE|EXECORDER|行权|权利金|COMBOPTIONS"),
    ("组合合约/拆分组合", r"组合|COMB_|DEL_COMB|COMBACTION|RCAMS|INSTRUMENTMAP"),
    ("套利/套保/做市", r"套利|套保|做市|SPBM|SPD_|HEDGE_|ARBITRAGE|RULE_?IN?STRPARAM"),
    ("报价/询价", r"报价|询价|QUOTE"),
    ("预埋单/条件单", r"预埋|PARKED|CONDORDER|CONDITIONAL|条件单"),
    ("监控中心/短信/手机", r"监控|CFMMC|短信|SMS|MOBILE|SEC_TRANSFER|验证码|手机号"),
    ("动态令牌/认证授权/API Key", r"动态令牌|OTP|认证|AUTH|API_|SHAKE|PASSWORD|口令|密码|IP_"),
    ("对冲设置/自对冲", r"对冲|OFFSET_"),
    ("质押/备兑", r"质押|备兑|MORTGAGE"),
    ("违规/限仓/交易时段等柜台策略", r"限仓|违规|异常|禁止|NOT_TRADING|OUT_OF_"),
    ("请求字段完整性（柜台级）", r"IS_MISSING|IS_WRONG|ACCOUNT_ID|INVESTOR_ID|EXCHANGE_ID_IS_MISSING"),
    ("主键/同步类", r"DUPLICATE_PK|CANNOT_FIND_PK|BROKER_SYNCH|INACTIVE_BROKER|DATA_SYNC|TK_BUSY"),
    ("交易所连接类", r"EXCHANG_TRADING|交易所网络|交易所未处理|交易所每秒|VALID_TRADER|PARTICIPANT|EXCHANGE_CLIENT"),
]
UNCLASSIFIED = "未归类（需人工判定）"

STATE_IMPL, STATE_GAP, STATE_UNREACH = "已实现", "可落地", "暂不可达"


def classify(eid: str, value: int, prompt: str):
    """-> (state, label/note)."""
    if value in IMPLEMENTED:
        return STATE_IMPL, IMPLEMENTED[value]
    if value in ACTIONABLE:
        return STATE_GAP, ACTIONABLE[value]
    key = "%s %s" % (eid, prompt)
    for label, pat in UNREACHABLE_RULES:
        if re.search(pat, key, re.I):
            return STATE_UNREACH, label
    return STATE_UNREACH, UNCLASSIFIED


# Any table row of the error.xml table: value | id | prompt | <whatever status>
ROW = re.compile(r"^\|\s*(-?\d+)\s*\|\s*([A-Za-z0-9_]+)\s*\|(.*?)\|[^|]*\|\s*$")
SUMMARY_HEAD = "## CTPBuddy 状态列说明"
SUMMARY_END = "## API 本地返回码"


def load_entries():
    for path in ERROR_XML_CANDIDATES:
        if os.path.exists(path):
            with open(path, "rb") as f:
                raw = f.read().decode("gb18030", errors="replace")
            entries = re.findall(
                r'<error\s+id="([^"]+)"\s+value="(-?\d+)"\s+prompt="([^"]*)"\s*/>', raw)
            if len(entries) != EXPECTED_ENTRIES:
                raise SystemExit("%s: entries = %d, expected %d" % (path, len(entries), EXPECTED_ENTRIES))
            return path, entries
    raise SystemExit("error.xml not found in any of: %s" % ", ".join(ERROR_XML_CANDIDATES))


def codes_str(codes):
    return " ".join("`%d`" % c for c in codes)


def render_summary(states: dict, labels: dict) -> list:
    """The whole 「CTPBuddy 状态列说明」 section, from data only."""
    n = Counter(states.values())
    impl = sorted(c for c, s in states.items() if s == STATE_IMPL)
    gaps = sorted(c for c, s in states.items() if s == STATE_GAP)
    unreach = sorted(c for c, s in states.items() if s == STATE_UNREACH)

    out = [
        SUMMARY_HEAD,
        "",
        "「CTPBuddy」列按**实际代码面**（`core/` 常量 + `handlers.rs` 的推送面分流）标注，不是意图声明。三态：",
        "",
        "| 状态 | 含义 | 条数 |",
        "|---|---|---|",
        "| **已实现** | 核心真的会发出此码，且推送面与官方口径一致 | %d |" % n[STATE_IMPL],
        "| **可落地** | 语义落在 CTPBuddy 的交易/风控范围内，但**当前无代码路径发出**——已登记的缺口，不是需求 | %d |" % n[STATE_GAP],
        "| **暂不可达** | 属于 CTPBuddy 不实现的业务域（银期转账、期权执行、组合、报价、监控中心、短信认证…），客户端在本仿真环境永远观察不到 | %d |" % n[STATE_UNREACH],
        "",
        "状态列与本节全部由 `tools/fill_errorcode_status.py` 按上述规则**全量重算**生成，改代码面后重跑即可，不要手改表格；"
        "`python tools/fill_errorcode_status.py --check` 用于 CI/评审时比对文档是否与脚本一致。",
        "",
        "### 已实现 %d 条与推送面" % n[STATE_IMPL],
        "",
    ]
    for label, codes in SURFACES:
        out.append("**%s**：%s  " % (label, codes_str(codes)))
    out[-1] = out[-1].rstrip()
    out += [
        "",
        "`116` 同时出现在报单与撤单两行：两条流各自计数、互不挤占。`3` 同时出现在登录与报单两行：登录 UserID 为空与报单 BrokerID/InvestorID 与会话不一致同码；登录 BrokerID 不符回 `63`。",
        "",
        "推送面分层的依据与逐条论证见 [`docs/notes/09-错单推送面与错误码对账.md`](notes/09-错单推送面与错误码对账.md)，设计口径见 [DESIGN §8.12](../DESIGN.md)。",
        "",
        "### 可落地 %d 条（缺口清单，按主题归并）" % n[STATE_GAP],
        "",
        "| 主题 | 错误码 | 缺什么 |",
        "|---|---|---|",
    ]
    for theme, codes, what in GAP_THEMES:
        out.append("| %s | %s | %s |" % (theme, codes_str(codes), what))
    out += [
        "",
        "### 暂不可达 %d 条的业务域分布" % n[STATE_UNREACH],
        "",
    ]
    dist = Counter(labels[c] for c in unreach)
    # keep the rule order (= domain importance), then the unclassified bucket
    order = [lab for lab, _ in UNREACHABLE_RULES] + [UNCLASSIFIED]
    parts = ["%s %d" % (lab, dist[lab]) for lab in order if dist.get(lab)]
    out.append(" · ".join(parts))
    out.append("")
    # sanity trailer the reader can eyeball: the three tallies sum to the full set
    out.append("合计 %d + %d + %d = %d 条（error.xml 全集）。" % (len(impl), len(gaps), len(unreach), len(states)))
    out.append("")
    return out


def regenerate(doc_text: str, entries) -> str:
    # keep the file's own line ending (the doc is CRLF on Windows checkouts)
    nl = "\r\n" if "\r\n" in doc_text else "\n"
    lines = doc_text.split(nl)
    keys = {(int(v), e): (e, p) for e, v, p in entries}
    states, labels, seen = {}, {}, set()

    # 1. rows: rewrite the status cell of every error.xml row
    for i, line in enumerate(lines):
        m = ROW.match(line)
        if not m:
            continue
        value = int(m.group(1))
        eid, prompt = m.group(2), m.group(3).rstrip()
        if (value, eid) not in keys:
            continue
        state, note = classify(eid, value, prompt)
        states[value], labels[value] = state, note
        lines[i] = "| %d | %s |%s| %s · %s |" % (value, eid, prompt, state, note)
        seen.add((value, eid))
    missing = sorted(set(keys) - seen)
    if missing:
        raise SystemExit("unannotated rows (not in doc table): %s" % missing[:20])

    # 2. summary section: replace everything between the two headings
    start = next((i for i, l in enumerate(lines) if l.rstrip() == SUMMARY_HEAD), None)
    end = next((i for i, l in enumerate(lines) if l.startswith(SUMMARY_END)), None)
    if start is None or end is None or end <= start:
        raise SystemExit("summary markers %r / %r not found in %s" % (SUMMARY_HEAD, SUMMARY_END, DOC))
    lines[start:end] = render_summary(states, labels)
    return nl.join(lines)


def self_check():
    impl_keys = set(IMPLEMENTED)
    surf_keys = set(c for _, codes in SURFACES for c in codes)
    if surf_keys != impl_keys:
        raise SystemExit("SURFACES != IMPLEMENTED: only-in-surfaces=%s only-in-implemented=%s"
                         % (sorted(surf_keys - impl_keys), sorted(impl_keys - surf_keys)))
    theme_counts = Counter(c for _, codes, _ in GAP_THEMES for c in codes)
    dup = sorted(c for c, k in theme_counts.items() if k > 1)
    if dup:
        raise SystemExit("codes in more than one GAP_THEMES row: %s" % dup)
    if set(theme_counts) != set(ACTIONABLE):
        raise SystemExit("GAP_THEMES != ACTIONABLE: only-in-themes=%s only-in-actionable=%s"
                         % (sorted(set(theme_counts) - set(ACTIONABLE)), sorted(set(ACTIONABLE) - set(theme_counts))))
    overlap = impl_keys & set(ACTIONABLE)
    if overlap:
        raise SystemExit("codes both IMPLEMENTED and ACTIONABLE: %s" % sorted(overlap))


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--check", action="store_true",
                    help="只比对不写入；文档与脚本重算结果不一致时以 1 退出")
    args = ap.parse_args()

    self_check()
    xml_path, entries = load_entries()
    with open(DOC, encoding="utf-8", newline="") as f:
        current = f.read()
    new = regenerate(current, entries)

    n = Counter(classify(e, int(v), p)[0] for e, v, p in entries)
    tally = "%s %d / %s %d / %s %d" % (STATE_IMPL, n[STATE_IMPL], STATE_GAP, n[STATE_GAP], STATE_UNREACH, n[STATE_UNREACH])

    if args.check:
        if new == current:
            print("OK: %s is up to date (%s; source %s)" % (os.path.relpath(DOC, ROOT), tally, os.path.relpath(xml_path, ROOT)))
            return 0
        diff = difflib.unified_diff(current.splitlines(), new.splitlines(),
                                    "doc (current)", "doc (recomputed)", lineterm="", n=0)
        shown = 0
        for line in diff:
            print(line)
            shown += 1
            if shown >= 60:
                print("... (diff truncated)")
                break
        print("DRIFT: %s differs from recompute (%s) — run without --check to rewrite"
              % (os.path.relpath(DOC, ROOT), tally), file=sys.stderr)
        return 1

    if new != current:
        with open(DOC, "w", encoding="utf-8", newline="") as f:
            f.write(new)
        print("rewrote %s (%s; source %s)" % (os.path.relpath(DOC, ROOT), tally, os.path.relpath(xml_path, ROOT)))
    else:
        print("unchanged %s (%s)" % (os.path.relpath(DOC, ROOT), tally))
    return 0


if __name__ == "__main__":
    sys.exit(main())
