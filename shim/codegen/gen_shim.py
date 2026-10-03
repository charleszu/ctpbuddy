#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""CTPBuddy Shim codegen: CTP 6.7.13 API headers -> Shim C++ sources.

Usage:
    python gen_shim.py [--sdk DIR] [--out DIR] [--src DIR]

Reads (user-supplied SDK, never committed):
    <sdk>/td/win64/ThostFtdcTraderApi.h
    <sdk>/md/win64/ThostFtdcMdApi.h

Writes (generated, checked in -- same policy as registry.hpp):
    <src>/generated/api_td.hpp        TraderApi class (all vtable methods)
    <src>/generated/api_td_reqs.cpp   request trampolines + core forwards
    <src>/generated/api_md.hpp        MdApi class
    <src>/generated/api_md_reqs.cpp   request trampolines + core forwards
    <src>/generated/dispatch_td.cpp   wire frame -> CThostFtdcTraderSpi table
    <src>/generated/dispatch_md.cpp   wire frame -> CThostFtdcMdSpi table

Design notes:
- The CTP ABI is the contract: every pure virtual of CThostFtdcTraderApi /
  CThostFtdcMdApi must be overridden with the exact same signature so the
  vtable layout is binary-compatible with the vendor DLL (same-name drop-in).
- Requests whose wire ids exist in the M1 protocol are trampolines into
  ApiCore::send_req (raw struct memcpy -- layouts match, see gen_structs.py).
  Everything else fails fast with a local OnRspError ("not implemented") so an
  app can never hang waiting for a callback that will never come.
- Wire ids are hand-assigned in core/ctpbuddy-wire/src/msgs.rs; the maps below
  name the ones the M1 core implements. Extending = add a map entry.
"""
import argparse
import os
import re
import sys

# --- wire message ids (mirror of core/ctpbuddy-wire/src/msgs.rs) --------------
MSG = {
    "PING": 0x0001, "PONG": 0x0002,
    "AUTH": 0x0101, "AUTH_RSP": 0x0102,
    "REQ_USER_LOGIN": 0x1001, "RSP_USER_LOGIN": 0x1002,
    "REQ_USER_LOGOUT": 0x1003, "RSP_USER_LOGOUT": 0x1004,
    "REQ_SETTLE_CONFIRM": 0x1005, "RSP_SETTLE_CONFIRM": 0x1006,
    "REQ_ORDER_INSERT": 0x1010, "RSP_ORDER_INSERT": 0x1011,
    "ERR_RTN_ORDER_INSERT": 0x1012, "RTN_ORDER": 0x1013, "RTN_TRADE": 0x1014,
    "REQ_ORDER_ACTION": 0x1015, "RSP_ORDER_ACTION": 0x1016,
    "ERR_RTN_ORDER_ACTION": 0x1017,
    "SUB_MD": 0x1020, "RSP_SUB_MD": 0x1021, "RTN_DEPTH_MD": 0x1022,
    "UNSUB_MD": 0x1023, "RSP_UNSUB_MD": 0x1024,
    "RSP_ERROR": 0x1030,
    "REQ_QRY_INSTRUMENT": 0x1040, "RSP_QRY_INSTRUMENT": 0x1041,
    "REQ_QRY_TRADING_ACCOUNT": 0x1042, "RSP_QRY_TRADING_ACCOUNT": 0x1043,
    "REQ_QRY_INVESTOR_POSITION": 0x1044, "RSP_QRY_INVESTOR_POSITION": 0x1045,
    "REQ_QRY_ORDER": 0x1046, "RSP_QRY_ORDER": 0x1047,
    "REQ_QRY_TRADE": 0x1048, "RSP_QRY_TRADE": 0x1049,
    # Reference-data queries. These answer from the *same* tables the ledger
    # computes from, so a client that cross-checks ReqQryInstrumentMarginRate
    # against ReqQryTradingAccount.CurrMargin sees consistent numbers
    # (notes/04 G).
    "REQ_QRY_INSTRUMENT_MARGIN_RATE": 0x1051, "RSP_QRY_INSTRUMENT_MARGIN_RATE": 0x1052,
    "REQ_QRY_INSTRUMENT_COMMISSION_RATE": 0x1053, "RSP_QRY_INSTRUMENT_COMMISSION_RATE": 0x1054,
    "REQ_QRY_INSTRUMENT_ORDER_COMM_RATE": 0x1055, "RSP_QRY_INSTRUMENT_ORDER_COMM_RATE": 0x1056,
    "REQ_QRY_BROKER_TRADING_PARAMS": 0x1057, "RSP_QRY_BROKER_TRADING_PARAMS": 0x1058,
    "REQ_QRY_INVESTOR_POSITION_DETAIL": 0x1059, "RSP_QRY_INVESTOR_POSITION_DETAIL": 0x105A,
    "REQ_QRY_INVESTOR_PRODUCT_GROUP_MARGIN": 0x105B, "RSP_QRY_INVESTOR_PRODUCT_GROUP_MARGIN": 0x105C,
    "QRY_LAST": 0x1050,
}

# CTP request method -> (wire req msg, request struct). Requests absent here
# get the local OnRspError trampoline.
TD_REQUESTS = {
    "ReqUserLogin": ("REQ_USER_LOGIN", "CThostFtdcReqUserLoginField"),
    "ReqUserLogout": ("REQ_USER_LOGOUT", "CThostFtdcUserLogoutField"),
    "ReqSettlementInfoConfirm": ("REQ_SETTLE_CONFIRM", "CThostFtdcSettlementInfoConfirmField"),
    "ReqOrderInsert": ("REQ_ORDER_INSERT", "CThostFtdcInputOrderField"),
    "ReqOrderAction": ("REQ_ORDER_ACTION", "CThostFtdcInputOrderActionField"),
    "ReqQryInstrument": ("REQ_QRY_INSTRUMENT", "CThostFtdcQryInstrumentField"),
    "ReqQryTradingAccount": ("REQ_QRY_TRADING_ACCOUNT", "CThostFtdcQryTradingAccountField"),
    "ReqQryInvestorPosition": ("REQ_QRY_INVESTOR_POSITION", "CThostFtdcQryInvestorPositionField"),
    "ReqQryInvestorPositionDetail": ("REQ_QRY_INVESTOR_POSITION_DETAIL", "CThostFtdcQryInvestorPositionDetailField"),
    "ReqQryInvestorProductGroupMargin": ("REQ_QRY_INVESTOR_PRODUCT_GROUP_MARGIN", "CThostFtdcQryInvestorProductGroupMarginField"),
    "ReqQryOrder": ("REQ_QRY_ORDER", "CThostFtdcQryOrderField"),
    "ReqQryTrade": ("REQ_QRY_TRADE", "CThostFtdcQryTradeField"),
    "ReqQryInstrumentMarginRate": ("REQ_QRY_INSTRUMENT_MARGIN_RATE", "CThostFtdcQryInstrumentMarginRateField"),
    "ReqQryInstrumentCommissionRate": ("REQ_QRY_INSTRUMENT_COMMISSION_RATE", "CThostFtdcQryInstrumentCommissionRateField"),
    "ReqQryInstrumentOrderCommRate": ("REQ_QRY_INSTRUMENT_ORDER_COMM_RATE", "CThostFtdcQryInstrumentOrderCommRateField"),
    "ReqQryBrokerTradingParams": ("REQ_QRY_BROKER_TRADING_PARAMS", "CThostFtdcQryBrokerTradingParamsField"),
}
MD_REQUESTS = {
    "ReqUserLogin": ("REQ_USER_LOGIN", "CThostFtdcReqUserLoginField"),
    "ReqUserLogout": ("REQ_USER_LOGOUT", "CThostFtdcUserLogoutField"),
}

# response frame msg -> the REQUEST frame whose pending entry its row
# completes. RSP_ERROR and QRY_LAST frames carry only the original req_id,
# so ApiCore must map request-type -> dispatch row through this key
# (api_core.cpp: find_row_by_req). Pushes and errrtn rows have no entry (0).
REQ_OF_RSP = {
    "RSP_USER_LOGIN": "REQ_USER_LOGIN",
    "RSP_USER_LOGOUT": "REQ_USER_LOGOUT",
    "RSP_SETTLE_CONFIRM": "REQ_SETTLE_CONFIRM",
    "RSP_ORDER_INSERT": "REQ_ORDER_INSERT",
    "RSP_ORDER_ACTION": "REQ_ORDER_ACTION",
    "RSP_QRY_ORDER": "REQ_QRY_ORDER",
    "RSP_QRY_TRADE": "REQ_QRY_TRADE",
    "RSP_QRY_INVESTOR_POSITION": "REQ_QRY_INVESTOR_POSITION",
    "RSP_QRY_INVESTOR_POSITION_DETAIL": "REQ_QRY_INVESTOR_POSITION_DETAIL",
    "RSP_QRY_INVESTOR_PRODUCT_GROUP_MARGIN": "REQ_QRY_INVESTOR_PRODUCT_GROUP_MARGIN",
    "RSP_QRY_TRADING_ACCOUNT": "REQ_QRY_TRADING_ACCOUNT",
    "RSP_QRY_INSTRUMENT": "REQ_QRY_INSTRUMENT",
    "RSP_QRY_INSTRUMENT_MARGIN_RATE": "REQ_QRY_INSTRUMENT_MARGIN_RATE",
    "RSP_QRY_INSTRUMENT_COMMISSION_RATE": "REQ_QRY_INSTRUMENT_COMMISSION_RATE",
    "RSP_QRY_INSTRUMENT_ORDER_COMM_RATE": "REQ_QRY_INSTRUMENT_ORDER_COMM_RATE",
    "RSP_QRY_BROKER_TRADING_PARAMS": "REQ_QRY_BROKER_TRADING_PARAMS",
    "RSP_SUB_MD": "SUB_MD",
    "RSP_UNSUB_MD": "UNSUB_MD",
}

# Dispatch tables: wire frame -> SPI callback. Row kinds:
#   rsp     success response; payload = response struct (or empty -> NULL);
#           completes the pending request (bIsLast = true).
#   qry     query stream row (bIsLast = false); QRY_LAST completes the stream.
#   cached  success response with an EMPTY payload: the pending request's
#           cached input struct is passed (RSP_ORDER_INSERT / RSP_ORDER_ACTION).
#   cached+err_rsp: a RSP_ERROR completing this request answers on the
#           response surface itself -- OnRsp*(NULL, pRspInfo, bIsLast=true),
#           the front-office half of a rejection (DESIGN §8.12).
#   errrtn  ERR_RTN_* failure: payload = the client's own input struct ++
#           RspInfoField ("payload_input") -- the exchange half of a rejection,
#           sent after the success response already consumed the pending.
#   sub     subscribe response, one per instrument; bIsLast on the last one.
#   push    server push (req_id = 0).
TD_DISPATCH = [
    ("rsp", "RSP_USER_LOGIN", "CThostFtdcRspUserLoginField", "OnRspUserLogin", {"trading_day": True}),
    ("rsp", "RSP_USER_LOGOUT", "CThostFtdcUserLogoutField", "OnRspUserLogout", {}),
    ("rsp", "RSP_SETTLE_CONFIRM", "CThostFtdcSettlementInfoConfirmField", "OnRspSettlementInfoConfirm", {}),
    ("cached", "RSP_ORDER_INSERT", "CThostFtdcInputOrderField", "OnRspOrderInsert",
     {"cache": "input_order", "err_rsp": True}),
    ("errrtn", "ERR_RTN_ORDER_INSERT", "CThostFtdcInputOrderField", "OnErrRtnOrderInsert",
     {"payload_input": True}),
    ("cached", "RSP_ORDER_ACTION", "CThostFtdcInputOrderActionField", "OnRspOrderAction",
     {"cache": "input_action", "err_rsp": True}),
    ("errrtn", "ERR_RTN_ORDER_ACTION", "CThostFtdcInputOrderActionField", "OnErrRtnOrderAction",
     {"payload_input": True, "synth_action": True}),
    ("qry", "RSP_QRY_ORDER", "CThostFtdcOrderField", "OnRspQryOrder", {}),
    ("qry", "RSP_QRY_TRADE", "CThostFtdcTradeField", "OnRspQryTrade", {}),
    ("qry", "RSP_QRY_INVESTOR_POSITION", "CThostFtdcInvestorPositionField", "OnRspQryInvestorPosition", {}),
    ("qry", "RSP_QRY_INVESTOR_POSITION_DETAIL", "CThostFtdcInvestorPositionDetailField", "OnRspQryInvestorPositionDetail", {}),
    ("qry", "RSP_QRY_INVESTOR_PRODUCT_GROUP_MARGIN", "CThostFtdcInvestorProductGroupMarginField", "OnRspQryInvestorProductGroupMargin", {}),
    ("qry", "RSP_QRY_TRADING_ACCOUNT", "CThostFtdcTradingAccountField", "OnRspQryTradingAccount", {}),
    ("qry", "RSP_QRY_INSTRUMENT", "CThostFtdcInstrumentField", "OnRspQryInstrument", {}),
    ("qry", "RSP_QRY_INSTRUMENT_MARGIN_RATE", "CThostFtdcInstrumentMarginRateField", "OnRspQryInstrumentMarginRate", {}),
    ("qry", "RSP_QRY_INSTRUMENT_COMMISSION_RATE", "CThostFtdcInstrumentCommissionRateField", "OnRspQryInstrumentCommissionRate", {}),
    ("qry", "RSP_QRY_INSTRUMENT_ORDER_COMM_RATE", "CThostFtdcInstrumentOrderCommRateField", "OnRspQryInstrumentOrderCommRate", {}),
    ("qry", "RSP_QRY_BROKER_TRADING_PARAMS", "CThostFtdcBrokerTradingParamsField", "OnRspQryBrokerTradingParams", {}),
    ("push", "RTN_ORDER", "CThostFtdcOrderField", "OnRtnOrder", {}),
    ("push", "RTN_TRADE", "CThostFtdcTradeField", "OnRtnTrade", {}),
]
MD_DISPATCH = [
    ("rsp", "RSP_USER_LOGIN", "CThostFtdcRspUserLoginField", "OnRspUserLogin", {"trading_day": True}),
    ("rsp", "RSP_USER_LOGOUT", "CThostFtdcUserLogoutField", "OnRspUserLogout", {}),
    ("sub", "RSP_SUB_MD", "CThostFtdcSpecificInstrumentField", "OnRspSubMarketData", {}),
    ("sub", "RSP_UNSUB_MD", "CThostFtdcSpecificInstrumentField", "OnRspUnSubMarketData", {}),
    ("push", "RTN_DEPTH_MD", "CThostFtdcDepthMarketDataField", "OnRtnDepthMarketData", {}),
]

# Core (non-request) virtuals, forwarded to ApiCore. Verbatim signatures from
# the vendor headers; Subscribe* for MD is hand-written (api_md.cpp).
TD_CORE = [
    ("void", "Release", "()"),
    ("void", "Init", "()"),
    ("int", "Join", "()"),
    ("const char *", "GetTradingDay", "()"),
    ("void", "GetFrontInfo", "(CThostFtdcFrontInfoField* pFrontInfo)"),
    ("void", "RegisterFront", "(char *pszFrontAddress)"),
    ("void", "RegisterNameServer", "(char *pszNsAddress)"),
    ("void", "RegisterFensUserInfo", "(CThostFtdcFensUserInfoField * pFensUserInfo)"),
    ("void", "RegisterSpi", "(CThostFtdcTraderSpi *pSpi)"),
    ("void", "SubscribePrivateTopic", "(THOST_TE_RESUME_TYPE nResumeType, int nSeqNo=1)"),
    ("void", "SubscribePublicTopic", "(THOST_TE_RESUME_TYPE nResumeType)"),
]
MD_CORE = [
    ("void", "Release", "()"),
    ("void", "Init", "()"),
    ("int", "Join", "()"),
    ("const char *", "GetTradingDay", "()"),
    ("void", "RegisterFront", "(char *pszFrontAddress)"),
    ("void", "RegisterNameServer", "(char *pszNsAddress)"),
    ("void", "RegisterFensUserInfo", "(CThostFtdcFensUserInfoField * pFensUserInfo)"),
    ("void", "RegisterSpi", "(CThostFtdcMdSpi *pSpi)"),
]
# MD subscription surface: declared here, implemented in api_md.cpp.
MD_HANDWRITTEN = [
    ("int", "SubscribeMarketData", "(char *ppInstrumentID[], int nCount)"),
    ("int", "UnSubscribeMarketData", "(char *ppInstrumentID[], int nCount)"),
    ("int", "SubscribeForQuoteRsp", "(char *ppInstrumentID[], int nCount)"),
    ("int", "UnSubscribeForQuoteRsp", "(char *ppInstrumentID[], int nCount)"),
]

CORE_FORWARD = {
    "Release": "core_release();",
    "Init": "core_init();",
    "Join": "return core_join();",
    "GetTradingDay": "return core_trading_day();",
    "GetFrontInfo": "core_front_info(pFrontInfo);",
    "RegisterFront": "core_register_front(pszFrontAddress);",
    "RegisterNameServer": "core_register_name_server(pszNsAddress);",
    "RegisterFensUserInfo": "core_register_fens(pFensUserInfo);",
    "RegisterSpi": "set_spi(pSpi);",
    "SubscribePrivateTopic": "(void)nResumeType; (void)nSeqNo;",
    "SubscribePublicTopic": "(void)nResumeType;",
}

API_CLASS_RE = {
    "td": re.compile(r"class\s+TRADER_API_EXPORT\s+CThostFtdcTraderApi\s*\{(.*?)\nprotected:", re.S),
    "md": re.compile(r"class\s+MD_API_EXPORT\s+CThostFtdcMdApi\s*\{(.*?)\nprotected:", re.S),
}
METHOD_RE = re.compile(r"^\s*virtual\s+((?:const\s+char\s*\*|int|void))\s+(\w+)\s*\(([^;]*?)\)\s*(?:=\s*0)?\s*;\s*$")
PARAM_RE = re.compile(r"^\s*([\w:]+)\s*\*\s*(\w+)\s*$")


def read_text(path):
    with open(path, "r", encoding="gb18030", errors="replace") as f:
        return f.read()


def parse_api_methods(body):
    """-> [(ret, name, params)] for every pure virtual in the class body."""
    out = []
    for line in body.splitlines():
        m = METHOD_RE.match(line)
        if not m:
            continue
        ret, name, params = m.group(1), m.group(2), m.group(3).strip()
        if name in ("~CThostFtdcTraderApi", "~CThostFtdcMdApi"):
            continue
        out.append((" ".join(ret.split()), name, params))
    return out


def cstr(buf, size):
    """std::string from a fixed-size NUL-padded char array field."""
    return 'cstr_of(%s, %d)' % (buf, size)


def gen_class(api, cls_name, base_api, requests, core, handwritten, methods):
    """Returns the class declaration: core forwards + hand-written + all Req*."""
    lines_h = []
    spi = "CThostFtdcTraderSpi" if api == "td" else "CThostFtdcMdSpi"
    table = "kTdDispatch" if api == "td" else "kMdDispatch"

    lines_h.append("class %s : public %s, private ApiCore {" % (cls_name, base_api))
    lines_h.append("public:")
    lines_h.append("    %s();" % cls_name)
    lines_h.append("    ~%s();" % cls_name)
    lines_h.append("")
    lines_h.append("    // dispatch plumbing (see dispatch_%s.cpp)" % api)
    lines_h.append("    const DispatchRow* rows() const override;")
    lines_h.append("    void on_rsp_error_fallback(const CThostFtdcRspInfoField& rsp, int n_request_id) override;")
    lines_h.append("    void on_auth_failed(int n_request_id, const CThostFtdcRspInfoField& rsp) override;")
    lines_h.append("    void fire_front_connected() override;")
    lines_h.append("    void fire_front_disconnected(int reason) override;")
    lines_h.append("")
    lines_h.append("    // ---- core lifecycle / registration (forwarded to ApiCore) ----")
    for ret, name, params in core:
        lines_h.append("    %s %s%s override;" % (ret, name, params))
    lines_h.append("")
    lines_h.append("    // ---- hand-written request surface ----")
    for ret, name, params in handwritten:
        lines_h.append("    %s %s%s override;" % (ret, name, params))
    lines_h.append("")
    lines_h.append("    // ---- request trampolines (api_%s_reqs.cpp) ----" % api)
    core_names = {name for _, name, _ in core} | {name for _, name, _ in handwritten}
    for ret, name, params in methods:
        if name not in core_names:
            lines_h.append("    %s %s(%s) override;" % (ret, name, params))
    lines_h.append("")
    lines_h.append("protected:")
    lines_h.append("};")
    return "\n".join(lines_h)


def strip_defaults(params):
    """Default arguments belong in the class declaration only."""
    return re.sub(r"(\w+)\s*=\s*[^,)]+", r"\1", params)


def gen_reqs(api, cls_name, methods, requests, handwritten):
    """Definitions for every API method: ctor/dtor + core forwards + trampolines."""
    out = []
    out.append("%s::%s() {}\n" % (cls_name, cls_name))
    out.append("%s::~%s() {}\n" % (cls_name, cls_name))
    # core forwards
    for ret, name, params in (TD_CORE if api == "td" else MD_CORE):
        body = CORE_FORWARD.get(name)
        if body is None:
            continue
        out.append("%s %s::%s%s {\n    %s\n}\n" % (ret, cls_name, name, strip_defaults(params), body))
    # request trampolines (every pure virtual not covered by the core list)
    mapped = []
    handwritten_names = {name for _, name, _ in handwritten}
    for ret, name, params in methods:
        if name in handwritten_names:
            continue  # hand-written surface (api_md.cpp)
        if name in {n for _, n, _ in (TD_CORE if api == "td" else MD_CORE)}:
            continue
        entry = requests.get(name)
        if entry is None:
            # not implemented by the M1 core. With nRequestID: answer locally so
            # the app never waits for a callback that will not arrive.
            # Without (fire-and-forget registration): reject the call itself.
            if "nRequestID" in params:
                out.append(
                    "%s %s::%s(%s) {\n"
                    "    // not implemented by the M1 core: answer locally so the app\n"
                    "    // never waits for a callback that will not arrive.\n"
                    "    unsupported(nRequestID, \"%s\");\n"
                    "    return 0;\n"
                    "}\n" % (ret, cls_name, name, params, name))
            else:
                out.append(
                    "%s %s::%s(%s) {\n"
                    "    // not implemented by the M1 core: reject the call itself.\n"
                    "    (void)0;\n"
                    "    return -1;\n"
                    "}\n" % (ret, cls_name, name, params))
            continue
        msg_name, _struct = entry
        # first parameter: <Type> *<name>
        first = params.split(",")[0].strip()
        m = PARAM_RE.match(first)
        if not m:
            raise SystemExit("cannot parse parameter of %s: %r" % (name, first))
        ptype, pname = m.group(1), m.group(2)
        mapped.append(name)
        out.append(
            "%s %s::%s(%s) {\n"
            "    return send_req<%s>(msgs::%s, %s, nRequestID);\n"
            "}\n" % (ret, cls_name, name, params, ptype, msg_name, pname))
    return "\n".join(out), mapped


def gen_dispatch(api, rows, spi_type):
    """Frame -> SPI dispatch table."""
    out = []
    out.append("// @generated by shim/codegen/gen_shim.py. DO NOT EDIT.")
    out.append('#include "api_%s.hpp"' % api)
    out.append("")
    out.append("namespace ctpbuddy {")
    out.append("namespace {")
    out.append("")
    out.append("using ctpbuddy::ApiCore;")
    out.append("using ctpbuddy::Frame;")
    out.append("using ctpbuddy::Pending;")
    out.append("")
    for kind, msg, struct, fn, opt in rows:
        msg_id = MSG[msg]
        var = "row_%s" % fn
        if kind in ("rsp", "qry"):
            td = opt.get("trading_day", False)
            out.append("static void %s(ApiCore& c, const Frame& f) {" % var)
            out.append("    %s fld{};" % struct)
            out.append("    const %s* p = nullptr;" % struct)
            out.append("    int nrid = 0;")
            out.append("    {")
            out.append("        std::lock_guard<std::mutex> g(c.mu());")
            if kind == "qry":
                out.append("        Pending* pd = c.find_pending(f.req_id);")
                out.append("        if (!pd) return;")
                out.append("        nrid = pd->n_request_id;")
            else:
                out.append("        Pending pd = c.take_pending(f.req_id);")
                out.append("        nrid = pd.n_request_id;")
            out.append("        p = payload_as(f, fld);")
            if td:
                out.append("        if (p) c.set_trading_day_locked(p->TradingDay);")
            out.append("    }")
            out.append("    static_cast<%s*>(c.spi())->%s(const_cast<%s*>(p), &c.zero_rsp_info(), nrid, %s);"
                       % (spi_type, fn, struct, "false" if kind == "qry" else "true"))
            out.append("}")
            if kind == "qry":
                out.append("")
                out.append("static void %s_last(ApiCore& c, int nrid) {" % var)
                out.append("    static_cast<%s*>(c.spi())->%s(nullptr, &c.zero_rsp_info(), nrid, true);"
                           % (spi_type, fn))
                out.append("}")
            out.append("")
        elif kind == "cached":
            cache = opt["cache"]
            out.append("static void %s(ApiCore& c, const Frame& f) {" % var)
            out.append("    Pending pd = c.take_pending(f.req_id);")
            out.append("    if (pd.req_msg == 0) return;")
            out.append("    static_cast<%s*>(c.spi())->%s(&pd.%s, &c.zero_rsp_info(), pd.n_request_id, true);"
                       % (spi_type, fn, cache))
            out.append("}")
            out.append("")
        elif kind == "errrtn":
            if opt.get("payload_input"):
                # Payload = the client's own input struct ++ RspInfoField: the
                # success response has already consumed the pending entry, so the
                # input must ride along (DESIGN §8.12, #42).
                out.append("static void %s(ApiCore& c, const Frame& f) {" % var)
                out.append("    %s in{};" % struct)
                out.append("    CThostFtdcRspInfoField rsp{};")
                out.append("    const size_t n = sizeof(in);")
                out.append("    if (f.payload.size() >= n) memcpy(&in, f.payload.data(), n);")
                out.append("    if (f.payload.size() >= n + sizeof(rsp)) memcpy(&rsp, f.payload.data() + n, sizeof(rsp));")
                if opt.get("synth_action"):
                    out.append("    CThostFtdcOrderActionField act{};")
                    out.append("    synth_order_action(act, in);")
                    out.append("    static_cast<%s*>(c.spi())->%s(&act, const_cast<CThostFtdcRspInfoField*>(&rsp));" % (spi_type, fn))
                else:
                    out.append("    static_cast<%s*>(c.spi())->%s(&in, const_cast<CThostFtdcRspInfoField*>(&rsp));" % (spi_type, fn))
                out.append("}")
                out.append("")
            elif opt.get("synth_action"):
                out.append("static void %s(ApiCore& c, const Frame& f) {" % var)
                out.append("    CThostFtdcRspInfoField rsp{};")
                out.append("    payload_as(f, rsp);")
                out.append("    Pending pd = c.take_pending(f.req_id);")
                out.append("    if (pd.req_msg == 0) return;")
                out.append("    CThostFtdcOrderActionField act{};")
                out.append("    synth_order_action(act, pd.input_action);")
                out.append("    static_cast<%s*>(c.spi())->%s(&act, const_cast<CThostFtdcRspInfoField*>(&rsp));" % (spi_type, fn))
                out.append("}")
            else:
                out.append("static void %s(ApiCore& c, const Frame& f) {" % var)
                out.append("    CThostFtdcRspInfoField rsp{};")
                out.append("    payload_as(f, rsp);")
                out.append("    Pending pd = c.take_pending(f.req_id);")
                out.append("    if (pd.req_msg == 0) return;")
                out.append("    static_cast<%s*>(c.spi())->%s(&pd.%s, &rsp);"
                           % (spi_type, fn, opt["cache"]))
                out.append("}")
            out.append("")
        elif kind == "sub":
            out.append("static void %s(ApiCore& c, const Frame& f) {" % var)
            out.append("    %s fld{};" % struct)
            out.append("    const %s* p = nullptr;" % struct)
            out.append("    int nrid = 0;")
            out.append("    bool last = false;")
            out.append("    {")
            out.append("        std::lock_guard<std::mutex> g(c.mu());")
            out.append("        Pending* pd = c.find_pending(f.req_id);")
            out.append("        if (!pd) return;")
            out.append("        nrid = pd->n_request_id;")
            out.append("        p = payload_as(f, fld);")
            out.append("        pd->responses += 1;")
            out.append("        last = pd->expected_responses != 0 && pd->responses >= pd->expected_responses;")
            out.append("        if (last) c.erase_pending(f.req_id);")
            out.append("    }")
            out.append("    static_cast<%s*>(c.spi())->%s(const_cast<%s*>(p), &c.zero_rsp_info(), nrid, last);"
                       % (spi_type, fn, struct))
            out.append("}")
            out.append("")
        elif kind == "push":
            out.append("static void %s(ApiCore& c, const Frame& f) {" % var)
            out.append("    %s fld{};" % struct)
            out.append("    const %s* p = payload_as(f, fld);" % struct)
            out.append("    if (p) static_cast<%s*>(c.spi())->%s(const_cast<%s*>(p));" % (spi_type, fn, struct))
            out.append("}")
            out.append("")
        else:
            raise SystemExit("unknown row kind %r" % kind)

    # error completions for RSP_ERROR (one per rsp/qry/cached row).
    # A query stream can complete as an error too: the front answers an
    # over-budget ReqQry* with RSP_ERROR[90] NEED_RETRY, which must land in
    # the matching OnRspQry* (bIsLast=true), never the OnRspError fallback.
    for kind, msg, struct, fn, opt in rows:
        if kind in ("rsp", "qry"):
            out.append("static void %s_err(ApiCore& c, const CThostFtdcRspInfoField& rsp, int nrid, const Pending&) {"
                       % ("row_%s" % fn))
            out.append("    static_cast<%s*>(c.spi())->%s(nullptr, const_cast<CThostFtdcRspInfoField*>(&rsp), nrid, true);" % (spi_type, fn))
            out.append("}")
            out.append("")
        elif kind == "cached" and opt.get("err_rsp"):
            # the request failed at the front office (front-office half of a
            # rejection): the callback answers NULL input + pRspInfo
            # (DESIGN §8.12); the 错单回报 half never follows these codes.
            out.append("static void %s_err(ApiCore& c, const CThostFtdcRspInfoField& rsp, int nrid, const Pending&) {"
                       % ("row_%s" % fn))
            out.append("    static_cast<%s*>(c.spi())->%s(nullptr, const_cast<CThostFtdcRspInfoField*>(&rsp), nrid, true);" % (spi_type, fn))
            out.append("}")
            out.append("")

    out.append("}  // namespace")
    out.append("")
    table = "kTdDispatch" if api == "td" else "kMdDispatch"
    out.append("const DispatchRow %s[] = {" % table)
    for kind, msg, struct, fn, opt in rows:
        var = "row_%s" % fn
        req = REQ_OF_RSP.get(msg)
        req_init = "msgs::%s" % req if req else "0"
        if kind == "qry":
            # err!=nullptr: RSP_ERROR (e.g. NEED_RETRY throttle) completes
            # the query through its own OnRspQry* with bIsLast=true.
            out.append("    {msgs::%s, &%s, &%s_last, &%s_err, %s}," % (msg, var, var, var, req_init))
        elif kind == "rsp":
            out.append("    {msgs::%s, &%s, nullptr, &%s_err, %s}," % (msg, var, var, req_init))
        elif kind == "cached" and opt.get("err_rsp"):
            out.append("    {msgs::%s, &%s, nullptr, &%s_err, %s}," % (msg, var, var, req_init))
        else:
            out.append("    {msgs::%s, &%s, nullptr, nullptr, %s}," % (msg, var, req_init))
    out.append("    {0, nullptr, nullptr, nullptr, 0},")
    out.append("};")
    out.append("")
    out.append("const DispatchRow* %s::rows() const { return %s; }" % (
        "TraderApi" if api == "td" else "MdApi", table))
    out.append("")
    out.append("void %s::on_rsp_error_fallback(const CThostFtdcRspInfoField& rsp, int n_request_id) {" % (
        "TraderApi" if api == "td" else "MdApi"))
    out.append("    static_cast<%s*>(spi())->OnRspError(const_cast<CThostFtdcRspInfoField*>(&rsp), n_request_id, true);" % spi_type)
    out.append("}")
    out.append("")
    out.append("void %s::on_auth_failed(int n_request_id, const CThostFtdcRspInfoField& rsp) {" % (
        "TraderApi" if api == "td" else "MdApi"))
    out.append("    static_cast<%s*>(spi())->OnRspUserLogin(nullptr, const_cast<CThostFtdcRspInfoField*>(&rsp), n_request_id, true);" % spi_type)
    out.append("}")
    out.append("")
    out.append("void %s::fire_front_connected() {" % ("TraderApi" if api == "td" else "MdApi"))
    out.append("    static_cast<%s*>(spi())->OnFrontConnected();" % spi_type)
    out.append("}")
    out.append("")
    out.append("void %s::fire_front_disconnected(int reason) {" % ("TraderApi" if api == "td" else "MdApi"))
    out.append("    static_cast<%s*>(spi())->OnFrontDisconnected(reason);" % spi_type)
    out.append("}")
    out.append("")
    out.append("}  // namespace ctpbuddy")
    out.append("")
    return "\n".join(out)


HEADER_TMPL = """// @generated by shim/codegen/gen_shim.py from CTP 6.7.13 headers. DO NOT EDIT.
// The class overrides EVERY pure virtual of %(base_api)s so the vtable is
// binary-compatible with the vendor DLL (same-name drop-in replacement).
#pragma once

#include "ThostFtdc%(include)s"
#include "api_core.hpp"

namespace ctpbuddy {

%(class_src)s

}  // namespace ctpbuddy
"""


def main():
    ap = argparse.ArgumentParser()
    here = os.path.dirname(os.path.abspath(__file__))
    ap.add_argument("--sdk", default=os.path.join(here, "..", "..", "ctpsdk", "6.7.13_20260225"))
    ap.add_argument("--src", default=os.path.join(here, "..", "src"))
    args = ap.parse_args()
    sdk = os.path.abspath(args.sdk)
    src = os.path.abspath(args.src)
    gen = os.path.join(src, "generated")
    os.makedirs(gen, exist_ok=True)

    td_path = os.path.join(sdk, "td", "win64", "ThostFtdcTraderApi.h")
    md_path = os.path.join(sdk, "md", "win64", "ThostFtdcMdApi.h")
    if not os.path.exists(td_path):
        td_path = os.path.join(sdk, "ThostFtdcTraderApi.h")
        md_path = os.path.join(sdk, "ThostFtdcMdApi.h")
    td_h = read_text(td_path)
    md_h = read_text(md_path)

    # 新查询编号由本生成器同步，禁止手改生成的常量。
    root = os.path.abspath(os.path.join(here, "..", ".."))
    for rel, pattern, template in [
        ("core/ctpbuddy-wire/src/msgs.rs", r"pub const (\w+): u16 = 0x([0-9A-Fa-f]+);", "pub const %s: u16 = 0x%04X;"),
        ("shim/src/api_core.hpp", r"constexpr uint16_t (\w+) = 0x([0-9A-Fa-f]+);", "constexpr uint16_t %s = 0x%04X;"),
        ("py/ctpbuddy/wire.py", r"^(\w+) = 0x([0-9A-Fa-f]+)", "%s = 0x%04X"),
    ]:
        path = os.path.join(root, rel)
        with open(path, encoding="utf-8") as f:
            text = f.read()
        matches = list(re.finditer(pattern, text, re.M))
        known = {m.group(1): int(m.group(2), 16) for m in matches}
        if len(known) != len(matches):
            raise SystemExit("消息常量重复: " + path)
        for name, value in MSG.items():
            if name in known and known[name] != value:
                raise SystemExit("消息编号不一致: %s %s" % (path, name))
        additions = [template % (name, value) for name, value in MSG.items() if name not in known]
        if additions:
            end = matches[-1].end()
            text = text[:end] + "\n" + "\n".join(additions) + text[end:]
            with open(path, "w", encoding="utf-8", newline="\n") as f:
                f.write(text)

    td_body = API_CLASS_RE["td"].search(td_h)
    md_body = API_CLASS_RE["md"].search(md_h)
    if not td_body or not md_body:
        raise SystemExit("API class not found in headers")
    td_methods = parse_api_methods(td_body.group(1))
    md_methods = parse_api_methods(md_body.group(1))

    # --- headers ---
    td_cls = gen_class("td", "TraderApi", "CThostFtdcTraderApi", TD_REQUESTS, TD_CORE, [], td_methods)
    with open(os.path.join(gen, "api_td.hpp"), "w", encoding="utf-8") as f:
        f.write(HEADER_TMPL % {
            "base_api": "CThostFtdcTraderApi",
            "include": "TraderApi.h",
            "class_src": td_cls,
        })
    md_cls = gen_class("md", "MdApi", "CThostFtdcMdApi", MD_REQUESTS, MD_CORE, MD_HANDWRITTEN, md_methods)
    with open(os.path.join(gen, "api_md.hpp"), "w", encoding="utf-8") as f:
        f.write(HEADER_TMPL % {
            "base_api": "CThostFtdcMdApi",
            "include": "MdApi.h",
            "class_src": md_cls,
        })

    # --- request trampolines ---
    REQS_TMPL = """// @generated by shim/codegen/gen_shim.py. DO NOT EDIT.
#include "api_%(api)s.hpp"

namespace ctpbuddy {

%(src)s

}  // namespace ctpbuddy
"""
    td_src, td_mapped = gen_reqs("td", "TraderApi", td_methods, TD_REQUESTS, [])
    with open(os.path.join(gen, "api_td_reqs.cpp"), "w", encoding="utf-8") as f:
        f.write(REQS_TMPL % {"api": "td", "src": td_src})
    md_src, md_mapped = gen_reqs("md", "MdApi", md_methods, MD_REQUESTS, MD_HANDWRITTEN)
    with open(os.path.join(gen, "api_md_reqs.cpp"), "w", encoding="utf-8") as f:
        f.write(REQS_TMPL % {"api": "md", "src": md_src})

    # --- dispatch tables ---
    with open(os.path.join(gen, "dispatch_td.cpp"), "w", encoding="utf-8") as f:
        f.write(gen_dispatch("td", TD_DISPATCH, "CThostFtdcTraderSpi"))
    with open(os.path.join(gen, "dispatch_md.cpp"), "w", encoding="utf-8") as f:
        f.write(gen_dispatch("md", MD_DISPATCH, "CThostFtdcMdSpi"))

    print("td api methods : %d (mapped: %s)" % (len(td_methods), ", ".join(td_mapped)))
    print("md api methods : %d (mapped: %s)" % (len(md_methods), ", ".join(md_mapped)))
    print("wrote          : %s" % gen)


if __name__ == "__main__":
    main()
