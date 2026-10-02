# -*- coding: utf-8 -*-
"""Regenerate the CTPBuddy status column in docs/错误码全集.md.

Every row is classified against the *actual* code surface (core constants +
the handlers' push-surface split), not against wishful intent:

  已实现  the core really emits this code, on the stated push surface
  可落地  the semantics sit inside CTPBuddy's trading/risk scope but no code
          path emits them yet -- a registered gap, not a feature request
  暂不可达  the code belongs to a business domain CTPBuddy does not implement
          (银期转账 / 换汇 / 期权执行 / 组合 / 报价 / 监控中心 / 短信 ...), so a
          client can never observe it here

Run:  python tools/fill_errorcode_status.py
"""
from __future__ import annotations

import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ERROR_XML = os.path.join(ROOT, "ctpsdk", "6.7.13_20260225", "td", "win64", "error.xml")
DOC = os.path.join(ROOT, "docs", "错误码全集.md")

# ---------------------------------------------------------------------------
# 已实现: code -> push surface, cross-checked against core/ctpbuddy-server
# ---------------------------------------------------------------------------
# insert CTP-layer refusals -> RSP_ERROR only (handlers::reject_insert, first arm)
# insert exchange-layer refusals -> RSP_ORDER_INSERT{0} then ERR_RTN_ORDER_INSERT
# cancel refusals -> RSP_ERROR (OnRspOrderAction) then ERR_RTN_ORDER_ACTION
IMPLEMENTED = {
    0:   ("已实现", "成功响应恒 0（非错误）"),
    3:   ("已实现", "登录 UserID 为空（`INVALID_LOGIN`，本地化文案）"),
    15:  ("已实现", "报单字段有误 / 价格≤0 / PriceType 非法；CTP 层 · 仅 `OnRspOrderInsert`"),
    16:  ("已实现", "合约不在目录；CTP 层 · 仅 `OnRspOrderInsert`"),
    17:  ("已实现", "合约 `IsTrading=0`；CTP 层 · 仅 `OnRspOrderInsert`"),
    22:  ("已实现", "同 (front, session, OrderRef) 活动单重复；CTP 层 · 仅 `OnRspOrderInsert`"),
    25:  ("已实现", "撤单找不到活动单；**双面** `OnRspOrderAction` → `OnErrRtnOrderAction`"),
    26:  ("已实现", "撤单命中终态（`terminal_refs` 记忆）；**双面**"),
    30:  ("已实现", "平仓量超过今+昨持仓；CTP 层 · 仅 `OnRspOrderInsert`"),
    31:  ("已实现", "可用资金 < 冻结估算额；CTP 层 · 仅 `OnRspOrderInsert`"),
    50:  ("已实现", "CloseToday 可用今仓不足；CTP 层 · 仅 `OnRspOrderInsert`"),
    51:  ("已实现", "CloseYesterday 可用昨仓不足；CTP 层 · 仅 `OnRspOrderInsert`"),
    63:  ("已实现", "登录 BrokerID 与核心不一致（复用 `AUTH_FAILED` 槽位，语义偏差见 notes/09）"),
    90:  ("已实现", "`ReqQry*` 每会话超预算（`qry_gate`，默认 2/s）；查询面 `OnRspQry*`"),
    116: ("已实现", "报单/撤单共享的每秒预算（`order_gate`，`--order-freq` 默认 20）；insert 仅响应面 / cancel **双面**"),
    148: ("已实现", "请求 ExchangeID 与合约目录不符；CTP 层 · 仅 `OnRspOrderInsert`"),
    163: ("已实现", "限价超出最新 tick 涨跌停板；**交易所层** · `OnRspOrderInsert{0}` → `OnErrRtnOrderInsert`"),
    164: ("已实现", "手数 ≤0 或越所/品种 min-max；**交易所层**"),
    165: ("已实现", "限价非最小变动价位整数倍；**交易所层**"),
}

# API 本地返回码（error.xml 不含，照 API 文档录，见表格末段）
API_LOCAL = [
    (-3, "未登录 / 会话失效：报单、撤单、查询、订阅各面均先判会话"),
    (-2, "未认证（先 AUTH）/ 报单·撤单·登录·登出·确认·订阅字段长度错误 / shim 在途请求超许可"),
    (-1, "报文类型不支持"),
]

# ---------------------------------------------------------------------------
# 可落地: registered gaps inside our own scope (no code path emits them yet)
# ---------------------------------------------------------------------------
ACTIONABLE = {
    5:   "同一 UserID 重复登录（当前按会话累加 SessionID，不拒绝）",
    6:   "未登录即请求——当前统一用 API 本地 -3，未用此码",
    7:   "未 Init 完即请求——shim 侧保证，核心不判",
    9:   "权限：报单/查询权限位（规则表尚未建模）",
    11:  "找不到用户：登录 UserID 未开户（当前 SimNow 式自动开户）",
    13:  "找不到投资者：登录 InvestorID 不存在（同上）",
    19:  "投资者不活跃（无停用/权限态）",
    20:  "投资者未在交易所开户（无开户状态字段）",
    23:  "撤单 ActionFlag 非法（当前只支持 Delete，其余归一）",
    24:  "撤单已报送未回报不允许重复撤（无在途撤单去重）",
    28:  "无报单交易权限（权限位未建模）",
    29:  "只可平仓（规则表「开仓受限」档位未注入）",
    37:  "现货卖空（无现货标的）",
    42:  "结算结果未确认：确认报文已收，但**尚未做报单前置门禁**",
    48:  "投资者/口令无效（无口令校验与错次计数）",
    49:  "登录 IP 不合法（无 IP 白名单）",
    52:  "经纪公司可用条件单数量不足（条件单未实现）",
    60:  "用户在线会话超上限（无上限配置）",
    64:  "客户端未认证（shim 侧返回，核心不产）",
    75:  "连续登录失败超限（无错次锁定）",
    77:  "无此功能：`ReqOrderAction` 挂起/激活/修改、GTC 报单、组合/询价/报价等（FAQ #42/#13）",
    78:  "发送报单失败（无交易所链路，恒不触发）",
    79:  "发送报单操作失败（同上）",
    80:  "交易所不支持的价格类型（任意价/市价归一后不判）",
    82:  "无效的组合合约（组合合约未实现）",
    91:  "交易所返回的错误：**常量已留**，交易所侧拒单转发尚未接线（#43 相邻）",
    95:  "不支持的价格（限价≤0 走 15）",
    106: "投资者限仓（持仓限额规则未建模）",
    139: "当前时间禁止行权（期权未实现）",
    157: "该交易所不支持撤销未知单（撤单走 25，未按所分流）",
    162: "产品不在可交易阶段（无交易时段门禁，场景时钟只驱动行情）",
    2:   "会话信息不一致（撤单携带的 FrontID/SessionID 与本会话不符——当前按 OrderRef 定位，未校验）",
    4:   "用户不活跃（无用户停用态）",
    8:   "前置不活跃（无前置状态机；重连即换 FrontID）",
    12:  "找不到经纪公司（`--broker-id` 单一，无多 Broker 表）",
    27:  "不支持的功能：无此功能（与 77 同族，见上）",
    38:  "不合法的结算引用（无银行存支票）",
    43:  "没有对应的入金记录（无出入金）",
    55:  "重发未知单经纪/投资者不匹配（无 ResendOrder 流程）",
    65:  "合约不支持互换类型报单（互换单未实现）",
    89:  "打开文件失败（无流文件落盘）",
    94:  "不允许撤销 OTC 衍生报单（OTC 未实现）",
    100: "RULE 品种内对锁仓折扣参数（策略单未实现）",
    101: "本系统无报单权限（权限位未建模）",
    102: "无 DR 号（无灾备路由）",
    103: "不支持批量操作（无批量报单/批量撤单）",
    154: "查询核心忙（查询流控已用 90，无独立忙态）",
    158: "超过信息量限制（无 FTD 报文流控）",
    4041: "重复提交（提交/心跳层）",
    5051: "序号字段重复（无 MaxOrderRef 重发校验）",
    6000: "订阅合约数超限（`--subscribe` 无上限）",
}

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


def classify(eid: str, value: int, prompt: str) -> str:
    if value in IMPLEMENTED:
        state, note = IMPLEMENTED[value]
        return "%s · %s" % (state, note)
    if value in ACTIONABLE:
        return "可落地 · %s" % ACTIONABLE[value]
    key = "%s %s" % (eid, prompt)
    for label, pat in UNREACHABLE_RULES:
        if re.search(pat, key, re.I):
            return "暂不可达 · %s" % label
    return "暂不可达 · 未归类（需人工判定）"


ROW = re.compile(r"^\|\s*(-?\d+)\s*\|\s*([A-Za-z0-9_]+)\s*\|(.*?)\|\s*\|\s*$")


def main() -> int:
    with open(ERROR_XML, "rb") as f:
        raw = f.read().decode("gb18030", errors="replace")
    entries = re.findall(
        r'<error\s+id="([^"]+)"\s+value="(-?\d+)"\s+prompt="([^"]*)"\s*/>', raw)
    if len(entries) != 299:
        print("error.xml entries = %d, expected 299" % len(entries), file=sys.stderr)
        return 1

    with open(DOC, encoding="utf-8") as f:
        lines = f.read().split("\n")

    # key the doc rows by (value, id): unique across error.xml, and immune to
    # whitespace differences in the prompt cell
    keys = {(int(v), e) for e, v, _p in entries}
    seen = set()
    for i, line in enumerate(lines):
        m = ROW.match(line)
        if not m:
            continue
        value = int(m.group(1))
        eid, prompt = m.group(2), m.group(3).rstrip()
        if (value, eid) not in keys:
            continue
        lines[i] = "| %d | %s |%s| %s |" % (value, eid, prompt,
                                            classify(eid, value, prompt))
        seen.add((value, eid))
    missing = sorted(keys - seen)
    if missing:
        print("unannotated: %s" % missing[:20], file=sys.stderr)
        return 1

    with open(DOC, "w", encoding="utf-8") as f:
        f.write("\n".join(lines))
    print("annotated %d rows" % len(seen))
    return 0


if __name__ == "__main__":
    sys.exit(main())