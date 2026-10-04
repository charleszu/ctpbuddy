// CTPBuddy CTP ABI shim -- market-data API: the subscription surface needs
// custom bodies (packing InstrumentID arrays into the wire frame); everything
// else is generated (api_md_reqs.cpp / dispatch_md.cpp).
#include "api_md.hpp"

#include <vector>

namespace ctpbuddy {

int MdApi::SubscribeMarketData(char* ppInstrumentID[], int nCount) {
    if (nCount <= 0 || !ppInstrumentID) return 0;  // nothing to subscribe
    std::vector<CThostFtdcSpecificInstrumentField> fields(static_cast<size_t>(nCount));
    for (int i = 0; i < nCount; ++i) {
        memset(&fields[i], 0, sizeof(fields[i]));
        const char* id = ppInstrumentID[i];
        if (id) set_cstr(fields[i].InstrumentID, sizeof(fields[i].InstrumentID), id);
    }
    return send_request(msgs::SUB_MD, fields.data(), fields.size() * sizeof(fields[0]), -1);
}

int MdApi::UnSubscribeMarketData(char* ppInstrumentID[], int nCount) {
    if (nCount <= 0 || !ppInstrumentID) return 0;
    std::vector<CThostFtdcSpecificInstrumentField> fields(static_cast<size_t>(nCount));
    for (int i = 0; i < nCount; ++i) {
        memset(&fields[i], 0, sizeof(fields[i]));
        const char* id = ppInstrumentID[i];
        if (id) set_cstr(fields[i].InstrumentID, sizeof(fields[i].InstrumentID), id);
    }
    return send_request(msgs::UNSUB_MD, fields.data(), fields.size() * sizeof(fields[0]), -1);
}

int MdApi::SubscribeForQuoteRsp(char* ppInstrumentID[], int nCount) {
    // for-quote is a M2+ feature: answer locally, never leave the app hanging.
    CThostFtdcRspInfoField rsp{};
    rsp.ErrorID = 41;
    set_text(rsp.ErrorMsg, sizeof(rsp.ErrorMsg), "CTPBuddy 暂不支持询价订阅");
    if (nCount > 0 && ppInstrumentID && ppInstrumentID[0]) {
        CThostFtdcSpecificInstrumentField f{};
        set_cstr(f.InstrumentID, sizeof(f.InstrumentID), ppInstrumentID[0]);
        static_cast<CThostFtdcMdSpi*>(spi())->OnRspSubForQuoteRsp(&f, &rsp, -1, true);
    } else {
        static_cast<CThostFtdcMdSpi*>(spi())->OnRspSubForQuoteRsp(nullptr, &rsp, -1, true);
    }
    return 0;
}

int MdApi::UnSubscribeForQuoteRsp(char* ppInstrumentID[], int nCount) {
    CThostFtdcRspInfoField rsp{};
    rsp.ErrorID = 41;
    set_text(rsp.ErrorMsg, sizeof(rsp.ErrorMsg), "CTPBuddy 暂不支持询价订阅");
    if (nCount > 0 && ppInstrumentID && ppInstrumentID[0]) {
        CThostFtdcSpecificInstrumentField f{};
        set_cstr(f.InstrumentID, sizeof(f.InstrumentID), ppInstrumentID[0]);
        static_cast<CThostFtdcMdSpi*>(spi())->OnRspUnSubForQuoteRsp(&f, &rsp, -1, true);
    } else {
        static_cast<CThostFtdcMdSpi*>(spi())->OnRspUnSubForQuoteRsp(nullptr, &rsp, -1, true);
    }
    return 0;
}

}  // namespace ctpbuddy
