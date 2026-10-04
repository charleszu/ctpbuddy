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

namespace {

// For-quote subscription is a M2+ feature: answer locally with ErrorID 41 so
// the app never hangs. The callback is posted to the reader thread (vendor
// contract: SPI callbacks never run on the app's stack) and dropped when the
// app registered no SPI; -1 (网络连接失败) when not connected, like the vendor.
template <typename Fn>
int for_quote_stub(ApiCore& core, char* ppInstrumentID[], int nCount, Fn fire) {
    CThostFtdcRspInfoField rsp{};
    rsp.ErrorID = 41;
    set_text(rsp.ErrorMsg, sizeof(rsp.ErrorMsg), "CTPBuddy 暂不支持询价订阅");
    CThostFtdcSpecificInstrumentField f{};
    const bool have_id = nCount > 0 && ppInstrumentID && ppInstrumentID[0];
    if (have_id) set_cstr(f.InstrumentID, sizeof(f.InstrumentID), ppInstrumentID[0]);
    return core.post_callback([&core, f, have_id, rsp, fire]() mutable {
        auto* s = static_cast<CThostFtdcMdSpi*>(core.spi());
        if (!s) return;
        fire(s, have_id ? &f : nullptr, &rsp);
    });
}

}  // namespace

int MdApi::SubscribeForQuoteRsp(char* ppInstrumentID[], int nCount) {
    return for_quote_stub(*this, ppInstrumentID, nCount,
                          [](CThostFtdcMdSpi* s, CThostFtdcSpecificInstrumentField* f, CThostFtdcRspInfoField* rsp) {
                              s->OnRspSubForQuoteRsp(f, rsp, -1, true);
                          });
}

int MdApi::UnSubscribeForQuoteRsp(char* ppInstrumentID[], int nCount) {
    return for_quote_stub(*this, ppInstrumentID, nCount,
                          [](CThostFtdcMdSpi* s, CThostFtdcSpecificInstrumentField* f, CThostFtdcRspInfoField* rsp) {
                              s->OnRspUnSubForQuoteRsp(f, rsp, -1, true);
                          });
}

}  // namespace ctpbuddy
