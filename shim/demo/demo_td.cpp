// CTPBuddy M1 downstream demo -- a REAL CTP 6.7.13 application.
//
// This file contains zero CTPBuddy-specific code: it includes the stock vendor
// headers and links the CTPBuddy shim DLLs (thosttraderapi_se.dll /
// thostmduserapi_se.dll, dropped in place of the vendor DLLs). If this app
// completes against the Rust core, the ABI shim is transparent.
//
// Flow (a typical CTP app startup plus a round trip through the book):
//   RegisterFront -> Init -> OnFrontConnected
//   ReqUserLogin -> ReqSettlementInfoConfirm
//   (md api) SubscribeMarketData        <- the e2e harness resumes the
//                                           paused scenario when it sees the
//                                           "subscribed" line on stdout
//   OnRtnDepthMarketData (the full 3-tick replay settles the mark price)
//   ReqOrderInsert buy  @3502  -> crosses the ask -> RTN_ORDER '0' + RTN_TRADE
//   ReqQryTradingAccount / InvestorPosition / Instrument
//   ReqOrderInsert buy  @3480  -> parks in the book (status '3')
//   ReqOrderAction              -> cancel -> RTN_ORDER '5'
//   ReqQryOrder / ReqQryTrade
//   ReqOrderInsert sell @3498   -> closes the position -> RTN_TRADE
//   ReqQryTradingAccount recheck -> CloseProfit
//   (one unsupported Req* to prove the shim answers OnRspError locally)
//   (back-to-back ReqQry* pair: the second must fail rc -2 in flight)
//   ReqUserLogout -> Release
//
// Queries transparently ride out the front's per-second budget: an
// over-budget ReqQry* answers OnRspError[90] NEED_RETRY and the demo
// re-issues after the window, exactly like a production CTP client.
//
// Every step has a timeout and the process always terminates: a shim that
// leaves a downstream app hanging is a build failure, and this demo proves
// the M1 contract.
//
// Usage:
//     demo_td.exe <front-address> [broker] [investor]
// Exit code 0 + "DEMO: PASS" means the full round trip verified.
#include <chrono>
#include <condition_variable>
#include <cstdio>
#include <cstring>
#include <mutex>
#include <stdexcept>
#include <string>
#include <thread>
#include <vector>

#include "ThostFtdcTraderApi.h"
#include "ThostFtdcMdApi.h"

namespace {

constexpr int STEP_TIMEOUT_SEC = 20;

// Scenario constants -- keep in sync with tests/e2e/m1_shim_e2e.py.
constexpr const char* INSTRUMENT = "rb2601";
constexpr const char* EXCHANGE = "SHFE";
constexpr double BUY_PRICE = 3502.0;   // = ask1: crosses on insert
constexpr double REST_PRICE = 3480.0;  // deep below the book: parks
constexpr double SELL_PRICE = 3498.0;  // = bid1: crosses on insert
constexpr int LOTS = 1;
// The bundled ref-data snapshot ships no commission table (see
// `Catalog::bundled` in the core), so fills are free. A desk that supplies
// `commission_rates.jsonl` gets the full 开仓 / 平昨 / 平今 split instead.
constexpr double COMMISSION = 0.0;
constexpr double LAST_TICK_PRICE = 3501.0;  // scenario's final tick last price
constexpr int VOLUME_MULTIPLE = 10;      // rb2601 contract multiplier
// 保证金 = (ByVolume + ByMoney x Price x Mult) x Volume, priced at 昨结算价
// because the bundled MarginPriceType is '1' (notes/04 C2). The fill price
// (3502) is deliberately not the basis.
constexpr double PRE_SETTLEMENT = 3500.0;
constexpr double MARGIN_RATE = 0.16;     // refdata/margin_rates.jsonl 公司费率
constexpr int SCENARIO_TICKS = 3;        // ticks.csv rows (m1_shim_e2e.paused scenario)

struct DemoFail : std::runtime_error {
    explicit DemoFail(const std::string& what) : std::runtime_error(what) {}
};

std::string cstr(const char* p) { return p ? std::string(p) : std::string(); }

void put_cstr(char* dst, size_t n, const char* src) {
    size_t k = std::strlen(src);
    if (k >= n) k = n - 1;
    std::memcpy(dst, src, k);
    std::memset(dst + k, 0, n - k);
}

bool close_double(double a, double b, double tol = 1e-4) { return a - b <= tol && b - a <= tol; }

// One mutex + cv for the whole demo: every callback takes it, mutates, and
// notifies; the main thread waits on predicates. No lock is ever held while
// the main thread calls into the API.
struct Sync {
    std::mutex mu;
    std::condition_variable cv;

    template <class Pred>
    void wait(Pred pred, const char* what, int secs = STEP_TIMEOUT_SEC) {
        std::unique_lock<std::mutex> lk(mu);
        if (!cv.wait_for(lk, std::chrono::seconds(secs), pred)) {
            throw DemoFail(std::string("timeout waiting for ") + what);
        }
    }
    void notify() { cv.notify_all(); }
};

// ---- trader SPI: records everything the core pushes -------------------------
// NOTE: callbacks run on the shim's reader thread and must never throw across
// the DLL boundary -- failures are recorded here and raised on the main thread.
struct TdSpi : public CThostFtdcTraderSpi {
    Sync sync;

    bool front_connected = false;
    bool front_disconnected = false;
    int front_reason = 0;

    bool login_done = false;
    bool login_failed = false;
    int front_id = 0, session_id = 0;

    bool settle_done = false;
    bool insert_rsp = false;
    bool insert_err = false;
    bool action_rsp = false;
    bool logout_done = false;

    std::vector<CThostFtdcOrderField> orders;
    std::vector<CThostFtdcTradeField> trades;

    std::vector<CThostFtdcTradingAccountField> qry_account;
    bool qry_account_last = false;
    std::vector<CThostFtdcInvestorProductGroupMarginField> qry_product_margin;
    bool qry_product_margin_last = false;
    std::vector<CThostFtdcInvestorPositionField> qry_position;
    bool qry_position_last = false;
    std::vector<CThostFtdcInstrumentField> qry_instrument;
    bool qry_instrument_last = false;
    std::vector<CThostFtdcOrderField> qry_orders;
    bool qry_order_last = false;
    std::vector<CThostFtdcTradeField> qry_trades;
    bool qry_trade_last = false;

    // error channel: M1 answers unsupported requests locally with ErrorID -1
    bool expect_error = false;
    bool got_error = false;
    int error_id = 0;
    int error_req_id = -1;
    bool stray_error = false;
    std::string step_error;  // non-empty => a callback saw an error response
    // set when a query stream answered ErrorID 90 (NEED_RETRY): the front's
    // per-second QryFreq budget was spent; qry_with_retry re-issues.
    bool qry_throttled = false;

    void note(const char* line) {
        std::printf("[demo] %s\n", line);
        std::fflush(stdout);
    }

    void OnFrontConnected() override {
        std::lock_guard<std::mutex> g(sync.mu);
        front_connected = true;
        note("front connected (td)");
        sync.notify();
    }
    void OnFrontDisconnected(int nReason) override {
        std::lock_guard<std::mutex> g(sync.mu);
        front_disconnected = true;
        front_reason = nReason;
        note("front disconnected (td)");
        sync.notify();
    }

    void OnRspUserLogin(CThostFtdcRspUserLoginField* p, CThostFtdcRspInfoField* rsp, int, bool) override {
        std::lock_guard<std::mutex> g(sync.mu);
        if (rsp && rsp->ErrorID != 0) {
            login_failed = true;
            step_error = std::string("login rejected: ") + std::to_string(rsp->ErrorID) + " " + rsp->ErrorMsg;
        } else {
            login_done = true;
            front_id = p->FrontID;
            session_id = p->SessionID;
            char buf[128];
            std::snprintf(buf, sizeof(buf), "login ok: front=%d session=%d day=%s", p->FrontID, p->SessionID,
                          p->TradingDay);
            note(buf);
        }
        sync.notify();
    }
    void OnRspSettlementInfoConfirm(CThostFtdcSettlementInfoConfirmField*, CThostFtdcRspInfoField* rsp, int, bool) override {
        std::lock_guard<std::mutex> g(sync.mu);
        if (rsp && rsp->ErrorID != 0) {
            step_error = std::string("settle confirm error ") + std::to_string(rsp->ErrorID);
        } else {
            settle_done = true;
            note("settlement confirmed");
        }
        sync.notify();
    }

    void OnRspOrderInsert(CThostFtdcInputOrderField*, CThostFtdcRspInfoField* rsp, int nid, bool) override {
        std::lock_guard<std::mutex> g(sync.mu);
        if (rsp && rsp->ErrorID != 0) {
            insert_err = true;
            step_error = std::string("order insert rsp error ") + std::to_string(rsp->ErrorID);
        } else {
            insert_rsp = true;
            char buf[64];
            std::snprintf(buf, sizeof(buf), "order accepted (req %d)", nid);
            note(buf);
        }
        sync.notify();
    }
    void OnErrRtnOrderInsert(CThostFtdcInputOrderField* p, CThostFtdcRspInfoField* rsp) override {
        std::lock_guard<std::mutex> g(sync.mu);
        insert_err = true;
        step_error = std::string("ErrRtnOrderInsert ref=") + (p ? p->OrderRef : "?") + ": " +
                     std::to_string(rsp ? rsp->ErrorID : -1) + " " + (rsp ? rsp->ErrorMsg : "?");
        sync.notify();
    }
    void OnRspOrderAction(CThostFtdcInputOrderActionField*, CThostFtdcRspInfoField* rsp, int, bool) override {
        std::lock_guard<std::mutex> g(sync.mu);
        if (rsp && rsp->ErrorID != 0) {
            step_error = std::string("order action error ") + std::to_string(rsp->ErrorID);
        } else {
            action_rsp = true;
            note("order action accepted");
        }
        sync.notify();
    }

    void OnRtnOrder(CThostFtdcOrderField* p) override {
        std::lock_guard<std::mutex> g(sync.mu);
        orders.push_back(*p);
        char buf[192];
        std::snprintf(buf, sizeof(buf), "RtnOrder ref=%s sys=%s status=%c traded=%d total=%d price=%.0f", p->OrderRef,
                      p->OrderSysID, p->OrderStatus ? p->OrderStatus : '?', p->VolumeTraded, p->VolumeTotal,
                      p->LimitPrice);
        note(buf);
        sync.notify();
    }
    void OnRtnTrade(CThostFtdcTradeField* p) override {
        std::lock_guard<std::mutex> g(sync.mu);
        trades.push_back(*p);
        char buf[192];
        std::snprintf(buf, sizeof(buf), "RtnTrade %s %s %.0f x %d dir=%c offset=%c", p->TradeID, p->InstrumentID,
                      p->Price, p->Volume, p->Direction ? p->Direction : '?', p->OffsetFlag ? p->OffsetFlag : '?');
        note(buf);
        sync.notify();
    }

    void OnRspQryInvestorProductGroupMargin(CThostFtdcInvestorProductGroupMarginField* p,
        CThostFtdcRspInfoField* rsp, int, bool last) override {
        std::lock_guard<std::mutex> g(sync.mu);
        if (p) qry_product_margin.push_back(*p);
        if (last) qry_product_margin_last = true;
        if (rsp && rsp->ErrorID != 0) {
            if (last && rsp->ErrorID == 90) qry_throttled = true;
            else step_error = "品种保证金查询失败 " + std::to_string(rsp->ErrorID);
        }
        sync.notify();
    }

    void OnRspQryTradingAccount(CThostFtdcTradingAccountField* p, CThostFtdcRspInfoField* rsp, int, bool last) override {
        std::lock_guard<std::mutex> g(sync.mu);
        if (!last && p) qry_account.push_back(*p);
        if (last) qry_account_last = true;
        if (rsp && rsp->ErrorID != 0) {
            // ErrorID 90 (NEED_RETRY) is the front's per-second query
            // budget talking: the query did not run; the documented client
            // behavior is to wait past the window and re-issue.
            if (last && rsp->ErrorID == 90) {
                qry_throttled = true;
            } else {
                step_error = std::string("qry account error ") + std::to_string(rsp->ErrorID);
            }
        }
        sync.notify();
    }
    void OnRspQryInvestorPosition(CThostFtdcInvestorPositionField* p, CThostFtdcRspInfoField* rsp, int, bool last) override {
        std::lock_guard<std::mutex> g(sync.mu);
        if (!last && p) qry_position.push_back(*p);
        if (last) qry_position_last = true;
        if (rsp && rsp->ErrorID != 0) {
            if (last && rsp->ErrorID == 90) {
                qry_throttled = true;
            } else {
                step_error = std::string("qry position error ") + std::to_string(rsp->ErrorID);
            }
        }
        sync.notify();
    }
    void OnRspQryInstrument(CThostFtdcInstrumentField* p, CThostFtdcRspInfoField* rsp, int, bool last) override {
        std::lock_guard<std::mutex> g(sync.mu);
        if (!last && p) qry_instrument.push_back(*p);
        if (last) qry_instrument_last = true;
        if (rsp && rsp->ErrorID != 0) {
            if (last && rsp->ErrorID == 90) {
                qry_throttled = true;
            } else {
                step_error = std::string("qry instrument error ") + std::to_string(rsp->ErrorID);
            }
        }
        sync.notify();
    }
    void OnRspQryOrder(CThostFtdcOrderField* p, CThostFtdcRspInfoField* rsp, int, bool last) override {
        std::lock_guard<std::mutex> g(sync.mu);
        if (!last && p) qry_orders.push_back(*p);
        if (last) qry_order_last = true;
        if (rsp && rsp->ErrorID != 0) {
            if (last && rsp->ErrorID == 90) {
                qry_throttled = true;
            } else {
                step_error = std::string("qry order error ") + std::to_string(rsp->ErrorID);
            }
        }
        sync.notify();
    }
    void OnRspQryTrade(CThostFtdcTradeField* p, CThostFtdcRspInfoField* rsp, int, bool last) override {
        std::lock_guard<std::mutex> g(sync.mu);
        if (!last && p) qry_trades.push_back(*p);
        if (last) qry_trade_last = true;
        if (rsp && rsp->ErrorID != 0) {
            if (last && rsp->ErrorID == 90) {
                qry_throttled = true;
            } else {
                step_error = std::string("qry trade error ") + std::to_string(rsp->ErrorID);
            }
        }
        sync.notify();
    }

    void OnRspError(CThostFtdcRspInfoField* p, int nid, bool) override {
        std::lock_guard<std::mutex> g(sync.mu);
        got_error = true;
        error_id = p ? p->ErrorID : 0;
        error_req_id = nid;
        if (!expect_error) {
            stray_error = true;
            step_error = std::string("stray OnRspError ") + std::to_string(error_id) + " " + (p ? p->ErrorMsg : "");
        }
        char buf[192];
        std::snprintf(buf, sizeof(buf), "RspError id=%d req=%d msg=%s%s", p ? p->ErrorID : 0, nid, p ? p->ErrorMsg : "",
                      expect_error ? " (expected)" : " (STRAY)");
        note(buf);
        sync.notify();
    }
    void OnRspUserLogout(CThostFtdcUserLogoutField*, CThostFtdcRspInfoField* rsp, int, bool) override {
        std::lock_guard<std::mutex> g(sync.mu);
        if (rsp && rsp->ErrorID != 0) {
            step_error = std::string("logout error ") + std::to_string(rsp->ErrorID);
        } else {
            logout_done = true;
            note("logout ok");
        }
        sync.notify();
    }
};

// ---- market-data SPI ---------------------------------------------------------
struct MdSpi : public CThostFtdcMdSpi {
    Sync sync;

    bool front_connected = false;
    bool sub_done = false;
    bool sub_failed = false;
    int sub_count = 0;
    int tick_count = 0;
    CThostFtdcDepthMarketDataField last_tick{};

    void note(const char* line) {
        std::printf("[demo] %s\n", line);
        std::fflush(stdout);
    }

    void OnFrontConnected() override {
        std::lock_guard<std::mutex> g(sync.mu);
        front_connected = true;
        note("front connected (md)");
        sync.notify();
    }
    void OnFrontDisconnected(int nReason) override {
        std::lock_guard<std::mutex> g(sync.mu);
        char buf[64];
        std::snprintf(buf, sizeof(buf), "front disconnected (md), reason=%d", nReason);
        note(buf);
        sync.notify();
    }
    void OnRspSubMarketData(CThostFtdcSpecificInstrumentField* p, CThostFtdcRspInfoField* rsp, int, bool last) override {
        std::lock_guard<std::mutex> g(sync.mu);
        if (rsp && rsp->ErrorID != 0) {
            sub_failed = true;
            char buf[128];
            std::snprintf(buf, sizeof(buf), "subscribe rejected: %d %s", rsp->ErrorID, rsp->ErrorMsg);
            note(buf);
        } else {
            ++sub_count;
            if (last) sub_done = true;
            char buf[96];
            std::snprintf(buf, sizeof(buf), "subscribed %s (last=%d)", p ? p->InstrumentID : "?", last ? 1 : 0);
            note(buf);
        }
        sync.notify();
    }
    void OnRtnDepthMarketData(CThostFtdcDepthMarketDataField* p) override {
        std::lock_guard<std::mutex> g(sync.mu);
        ++tick_count;
        if (tick_count == 1) {
            char buf[160];
            std::snprintf(buf, sizeof(buf), "tick: %s bid1=%.0f ask1=%.0f last=%.0f vol=%d", p->InstrumentID,
                          p->BidPrice1, p->AskPrice1, p->LastPrice, p->Volume);
            note(buf);
        }
        last_tick = *p;
        sync.notify();
    }
};

// ---- order helpers -----------------------------------------------------------
int g_req_seq = 0;

CThostFtdcInputOrderField make_input(const char* broker, const char* investor, const char* ref, char direction,
                                      char offset, double price, int volume) {
    CThostFtdcInputOrderField f{};
    put_cstr(f.BrokerID, sizeof(f.BrokerID), broker);
    put_cstr(f.InvestorID, sizeof(f.InvestorID), investor);
    put_cstr(f.OrderRef, sizeof(f.OrderRef), ref);
    put_cstr(f.InstrumentID, sizeof(f.InstrumentID), INSTRUMENT);
    put_cstr(f.ExchangeID, sizeof(f.ExchangeID), EXCHANGE);
    f.OrderPriceType = '2';  // limit
    f.Direction = direction;  // '0' buy / '1' sell
    f.CombOffsetFlag[0] = offset;  // '0' open / '1' close today
    f.CombHedgeFlag[0] = '1';      // speculation
    f.LimitPrice = price;
    f.VolumeTotalOriginal = volume;
    f.TimeCondition = '3';       // GFD (TC_GFD; '1' = IOC)
    f.VolumeCondition = '1';     // any volume
    f.ContingentCondition = '1'; // immediate
    f.MinVolume = 1;
    f.RequestID = ++g_req_seq;
    return f;
}

void require_no_step_error(const TdSpi& spi) {
    if (!spi.step_error.empty()) throw DemoFail(spi.step_error);
}

// Well-behaved CTP client for the per-second query budget (CTP docs:
// 查询流控): an over-budget ReqQry* is answered by the front with
// OnRspError[90] "CTP：查询未就绪，请稍后重试" and bIsLast=true -- nothing
// ran. Production clients wait past the one-second window and re-issue;
// so does this demo, reusing the same nRequestID like real retry loops.
// `reset` / `done` run under the SPI lock (helper contract); `send` runs
// lock-free and only touches the API object.
template <class Reset, class Send, class Done>
void qry_with_retry(TdSpi& spi, const char* what, Reset reset, Send send, Done done) {
    constexpr int kMaxAttempts = 8;
    for (int attempt = 1; attempt <= kMaxAttempts; ++attempt) {
        int rid;
        {
            std::lock_guard<std::mutex> g(spi.sync.mu);
            reset();
            rid = ++g_req_seq;
        }
        send(rid);
        spi.sync.wait(done, what);
        bool throttled;
        {
            std::lock_guard<std::mutex> g(spi.sync.mu);
            throttled = spi.qry_throttled;
        }
        if (!throttled) {
            if (attempt > 1) {
                char buf[128];
                std::snprintf(buf, sizeof(buf), "%s: accepted after %d attempts (NEED_RETRY loop)", what, attempt);
                spi.note(buf);
            }
            return;
        }
        std::this_thread::sleep_for(std::chrono::milliseconds(1100));
    }
    throw DemoFail{std::string("query throttled for too long: ") + what};
}

void run(const char* front, const char* broker, const char* investor) {
    TdSpi td_spi;
    MdSpi md_spi;
    CThostFtdcTraderApi* td = CThostFtdcTraderApi::CreateFtdcTraderApi("");
    CThostFtdcMdApi* md = CThostFtdcMdApi::CreateFtdcMdApi("");

    struct ReleaseGuard {
        CThostFtdcTraderApi* td;
        CThostFtdcMdApi* md;
        ~ReleaseGuard() {
            if (md) md->Release();
            if (td) td->Release();
        }
    } guard{td, md};

    // -- connect + login ------------------------------------------------------
    td->RegisterFront(const_cast<char*>(front));
    td->RegisterSpi(&td_spi);
    td->Init();
    td_spi.sync.wait([&] { return td_spi.front_connected || td_spi.front_disconnected; }, "OnFrontConnected (td)");
    if (!td_spi.front_connected) throw DemoFail{"td front not connected"};

    CThostFtdcReqUserLoginField login{};
    put_cstr(login.BrokerID, sizeof(login.BrokerID), broker);
    put_cstr(login.UserID, sizeof(login.UserID), investor);
    td->ReqUserLogin(&login, ++g_req_seq);
    td_spi.sync.wait([&] { return td_spi.login_done || td_spi.login_failed; }, "OnRspUserLogin");
    require_no_step_error(td_spi);

    CThostFtdcSettlementInfoConfirmField confirm{};
    put_cstr(confirm.BrokerID, sizeof(confirm.BrokerID), broker);
    put_cstr(confirm.InvestorID, sizeof(confirm.InvestorID), investor);
    td->ReqSettlementInfoConfirm(&confirm, ++g_req_seq);
    td_spi.sync.wait([&] { return td_spi.settle_done; }, "OnRspSettlementInfoConfirm");
    require_no_step_error(td_spi);

    // -- market data: subscribe, then wait for the harness to resume playback --
    md->RegisterFront(const_cast<char*>(front));
    md->RegisterSpi(&md_spi);
    md->Init();
    md_spi.sync.wait([&] { return md_spi.front_connected; }, "OnFrontConnected (md)");
    char* instruments[1] = {const_cast<char*>(INSTRUMENT)};
    md->SubscribeMarketData(instruments, 1);
    md_spi.sync.wait([&] { return md_spi.sub_done || md_spi.sub_failed; }, "OnRspSubMarketData");
    if (!md_spi.sub_done) throw DemoFail{"market data subscription rejected"};
    std::printf("[demo] subscribed\n");  // harness resumes the scenario here
    std::fflush(stdout);

    // wait for the full replay (speed 0 fires all ticks at once): the
    // mark-to-market price must be settled before any account query
    md_spi.sync.wait([&] { return md_spi.tick_count >= SCENARIO_TICKS; }, "all scenario ticks");
    {
        std::lock_guard<std::mutex> g(md_spi.sync.mu);
        if (!close_double(md_spi.last_tick.BidPrice1, SELL_PRICE) ||
            !close_double(md_spi.last_tick.AskPrice1, BUY_PRICE)) {
            throw DemoFail{"unexpected book after first tick"};
        }
    }

    // -- crossing limit buy: immediate fill at 3502 ---------------------------
    {
        auto f = make_input(broker, investor, "100", '0', '0', BUY_PRICE, LOTS);
        td->ReqOrderInsert(&f, ++g_req_seq);
    }
    td_spi.sync.wait([&] { return (td_spi.insert_rsp || td_spi.insert_err) && td_spi.orders.size() >= 1 &&
                                  td_spi.trades.size() >= 1; },
                     "OnRtnOrder/OnRtnTrade for the crossing buy");
    require_no_step_error(td_spi);
    {
        std::lock_guard<std::mutex> g(td_spi.sync.mu);
        const auto& o = td_spi.orders.back();
        if (o.OrderStatus != '0' || o.VolumeTraded != LOTS) {
            throw DemoFail{std::string("crossing buy not fully traded (status=") +
                           (o.OrderStatus ? o.OrderStatus : '?') + ")"};
        }
        const auto& t = td_spi.trades.back();
        if (!close_double(t.Price, BUY_PRICE) || t.Volume != LOTS || t.Direction != '0' || t.OffsetFlag != '0') {
            throw DemoFail{"unexpected trade payload"};
        }
    }

    // -- query projections -----------------------------------------------------
    qry_with_retry(
        td_spi, "OnRspQryTradingAccount last",
        [&] {
            td_spi.qry_account.clear();
            td_spi.qry_account_last = false;
            td_spi.qry_throttled = false;
        },
        [&](int rid) {
            CThostFtdcQryTradingAccountField q{};
            put_cstr(q.BrokerID, sizeof(q.BrokerID), broker);
            put_cstr(q.InvestorID, sizeof(q.InvestorID), investor);
            td->ReqQryTradingAccount(&q, rid);
        },
        [&] { return td_spi.qry_account_last; });
    require_no_step_error(td_spi);
    {
        std::lock_guard<std::mutex> g(td_spi.sync.mu);
        if (td_spi.qry_account.size() != 1) throw DemoFail{"expected exactly one account row"};
        const auto& a = td_spi.qry_account[0];
        // CTP Balance is dynamic equity: the position is open and the last
        // tick (3501) sits one point under the 3502 entry, so Balance
        // carries the unrealized loss and Available follows it down.
        const double unrealized_open = (LAST_TICK_PRICE - BUY_PRICE) * LOTS * VOLUME_MULTIPLE;
        const double margin_open = PRE_SETTLEMENT * VOLUME_MULTIPLE * MARGIN_RATE;
        if (!close_double(a.Balance, 2000000.0 - COMMISSION + unrealized_open) ||
            !close_double(a.CurrMargin, margin_open) ||
            !close_double(a.Available, 2000000.0 - COMMISSION + unrealized_open - margin_open)) {
            throw DemoFail{std::string("account mismatch after open")};
        }
    }

    qry_with_retry(td_spi, "OnRspQryInvestorProductGroupMargin last",
        [&] {
            td_spi.qry_product_margin.clear();
            td_spi.qry_product_margin_last = false;
            td_spi.qry_throttled = false;
        },
        [&](int rid) {
            CThostFtdcQryInvestorProductGroupMarginField q{};
            put_cstr(q.BrokerID, sizeof(q.BrokerID), broker);
            put_cstr(q.InvestorID, sizeof(q.InvestorID), investor);
            put_cstr(q.ProductGroupID, sizeof(q.ProductGroupID), "rb");
            td->ReqQryInvestorProductGroupMargin(&q, rid);
        }, [&] { return td_spi.qry_product_margin_last; });
    require_no_step_error(td_spi);
    {
        std::lock_guard<std::mutex> g(td_spi.sync.mu);
        if (td_spi.qry_product_margin.size() != 1 ||
            !close_double(td_spi.qry_product_margin[0].UseMargin, td_spi.qry_account[0].CurrMargin))
            throw DemoFail{"品种保证金与 CurrMargin 不一致"};
        td_spi.note("品种保证金查询经真实 shim 回调与 CurrMargin 一致");
    }

    // Empty query: the real shim must deliver OnRspQry*(nullptr, ..., true),
    // not a zero-filled row. Filter to a product with no position.
    qry_with_retry(td_spi, "empty OnRspQryInvestorProductGroupMargin last",
        [&] {
            td_spi.qry_product_margin.clear();
            td_spi.qry_product_margin_last = false;
            td_spi.qry_throttled = false;
        },
        [&](int rid) {
            CThostFtdcQryInvestorProductGroupMarginField q{};
            put_cstr(q.BrokerID, sizeof(q.BrokerID), broker);
            put_cstr(q.InvestorID, sizeof(q.InvestorID), investor);
            put_cstr(q.ProductGroupID, sizeof(q.ProductGroupID), "no-such-product");
            td->ReqQryInvestorProductGroupMargin(&q, rid);
        }, [&] { return td_spi.qry_product_margin_last; });
    require_no_step_error(td_spi);
    {
        std::lock_guard<std::mutex> g(td_spi.sync.mu);
        if (!td_spi.qry_product_margin.empty())
            throw DemoFail{"empty product-margin query returned a zero-filled row"};
        td_spi.note("空查询经真实 shim 回调 nullptr + bIsLast=true");
    }

    qry_with_retry(
        td_spi, "OnRspQryInvestorPosition last",
        [&] {
            td_spi.qry_position.clear();
            td_spi.qry_position_last = false;
            td_spi.qry_throttled = false;
        },
        [&](int rid) {
            CThostFtdcQryInvestorPositionField q{};
            put_cstr(q.BrokerID, sizeof(q.BrokerID), broker);
            put_cstr(q.InvestorID, sizeof(q.InvestorID), investor);
            td->ReqQryInvestorPosition(&q, rid);
        },
        [&] { return td_spi.qry_position_last; });
    require_no_step_error(td_spi);
    {
        std::lock_guard<std::mutex> g(td_spi.sync.mu);
        if (td_spi.qry_position.size() != 1) throw DemoFail{"expected exactly one position row"};
        const auto& p = td_spi.qry_position[0];
        if (p.PosiDirection != '2' || p.Position != LOTS || p.TodayPosition != LOTS || p.YdPosition != 0) {
            throw DemoFail{"position mismatch after open"};
        }
    }

    qry_with_retry(
        td_spi, "OnRspQryInstrument last",
        [&] {
            td_spi.qry_instrument.clear();
            td_spi.qry_instrument_last = false;
            td_spi.qry_throttled = false;
        },
        [&](int rid) {
            CThostFtdcQryInstrumentField q{};
            put_cstr(q.InstrumentID, sizeof(q.InstrumentID), INSTRUMENT);
            td->ReqQryInstrument(&q, rid);
        },
        [&] { return td_spi.qry_instrument_last; });
    require_no_step_error(td_spi);
    {
        std::lock_guard<std::mutex> g(td_spi.sync.mu);
        if (td_spi.qry_instrument.size() != 1 || td_spi.qry_instrument[0].VolumeMultiple != VOLUME_MULTIPLE) {
            throw DemoFail{"instrument row mismatch (VolumeMultiple != 10)"};
        }
    }

    // -- a request the M1 core does not implement: the shim must answer
    //    OnRspError(-1) locally instead of leaving the app hanging ------------
    // ReqQryInvestor is the sample: still unimplemented (notes/04 lists
    // Investor query results as a v2 item). It used to be
    // ReqQryInstrumentCommissionRate, until the reference-data queries landed
    // and that one started answering for real -- which is exactly the point:
    // a "this is unsupported" probe must name a request that stays
    // unsupported, or the demo silently stops testing anything.
    {
        int rid;
        {
            std::lock_guard<std::mutex> g(td_spi.sync.mu);
            td_spi.expect_error = true;
            td_spi.got_error = false;
            rid = ++g_req_seq;
        }
        CThostFtdcQryTradingCodeField tq{};
        td->ReqQryTradingCode(&tq, rid);
        td_spi.sync.wait([&] { return td_spi.got_error; }, "OnRspError for an unsupported request");
        std::lock_guard<std::mutex> g(td_spi.sync.mu);
        td_spi.expect_error = false;
        if (td_spi.error_id != -1 || td_spi.error_req_id != rid) {
            throw DemoFail{"unsupported request did not answer OnRspError(-1) with the right nRequestID"};
        }
    }

    // -- resting GFD order + cancel -------------------------------------------
    {
        auto f = make_input(broker, investor, "777", '0', '0', REST_PRICE, LOTS);
        td->ReqOrderInsert(&f, ++g_req_seq);
    }
    // §8.9: a resting order reports 'a' (unknown) and then '3' (queued) as
    // two back-to-back pushes -- wait for the STATUS, never for "one more
    // order arrived", or the check races between the two pushes.
    td_spi.sync.wait([&] {
        for (const auto& o : td_spi.orders) {
            if (std::string(o.OrderRef) == "777" && o.OrderStatus == '3') return true;
        }
        return false;
    }, "RTN_ORDER '3' for the resting order");
    require_no_step_error(td_spi);
    {
        std::lock_guard<std::mutex> g(td_spi.sync.mu);
        const auto& o = td_spi.orders.back();
        if (std::string(o.OrderRef) != "777" || o.OrderStatus != '3') {
            throw DemoFail{"order did not park in the book (status expected '3')"};
        }
    }
    {
        CThostFtdcInputOrderActionField a{};
        put_cstr(a.BrokerID, sizeof(a.BrokerID), broker);
        put_cstr(a.InvestorID, sizeof(a.InvestorID), investor);
        put_cstr(a.OrderRef, sizeof(a.OrderRef), "777");
        put_cstr(a.InstrumentID, sizeof(a.InstrumentID), INSTRUMENT);
        put_cstr(a.ExchangeID, sizeof(a.ExchangeID), EXCHANGE);
        a.FrontID = td_spi.front_id;
        a.SessionID = td_spi.session_id;
        a.ActionFlag = '0';  // delete
        td->ReqOrderAction(&a, ++g_req_seq);
    }
    // the cancel pushes 前态('3') + 新态('5'): wait for the terminal status
    td_spi.sync.wait([&] {
        for (const auto& o : td_spi.orders) {
            if (std::string(o.OrderRef) == "777" && o.OrderStatus == '5') return true;
        }
        return false;
    }, "RTN_ORDER '5' for the cancel");
    require_no_step_error(td_spi);
    {
        std::lock_guard<std::mutex> g(td_spi.sync.mu);
        const auto& o = td_spi.orders.back();
        if (std::string(o.OrderRef) != "777" || o.OrderStatus != '5') {
            throw DemoFail{"order was not cancelled (status expected '5')"};
        }
    }

    // -- query order / trade streams -------------------------------------------
    qry_with_retry(
        td_spi, "OnRspQryOrder last",
        [&] {
            td_spi.qry_orders.clear();
            td_spi.qry_order_last = false;
            td_spi.qry_throttled = false;
        },
        [&](int rid) {
            CThostFtdcQryOrderField q{};
            put_cstr(q.BrokerID, sizeof(q.BrokerID), broker);
            put_cstr(q.InvestorID, sizeof(q.InvestorID), investor);
            put_cstr(q.InstrumentID, sizeof(q.InstrumentID), INSTRUMENT);
            td->ReqQryOrder(&q, rid);
        },
        [&] { return td_spi.qry_order_last; });
    require_no_step_error(td_spi);
    {
        std::lock_guard<std::mutex> g(td_spi.sync.mu);
        bool found_cancelled = false;
        for (const auto& o : td_spi.qry_orders) {
            if (std::string(o.OrderRef) == "777" && o.OrderStatus == '5') found_cancelled = true;
        }
        if (td_spi.qry_orders.size() < 2 || !found_cancelled) {
            throw DemoFail{"qry order stream missing the cancelled order"};
        }
    }
    qry_with_retry(
        td_spi, "OnRspQryTrade last",
        [&] {
            td_spi.qry_trades.clear();
            td_spi.qry_trade_last = false;
            td_spi.qry_throttled = false;
        },
        [&](int rid) {
            CThostFtdcQryTradeField q{};
            put_cstr(q.BrokerID, sizeof(q.BrokerID), broker);
            put_cstr(q.InvestorID, sizeof(q.InvestorID), investor);
            put_cstr(q.InstrumentID, sizeof(q.InstrumentID), INSTRUMENT);
            td->ReqQryTrade(&q, rid);
        },
        [&] { return td_spi.qry_trade_last; });
    require_no_step_error(td_spi);
    {
        std::lock_guard<std::mutex> g(td_spi.sync.mu);
        if (td_spi.qry_trades.size() < 1) throw DemoFail{"qry trade stream empty"};
    }

    // -- crossing limit sell: close the position at 3498 -----------------------
    size_t trades_before;
    {
        std::lock_guard<std::mutex> g(td_spi.sync.mu);
        trades_before = td_spi.trades.size();
    }
    {
        auto f = make_input(broker, investor, "778", '1', '1', SELL_PRICE, LOTS);
        td->ReqOrderInsert(&f, ++g_req_seq);
    }
    td_spi.sync.wait([&] { return td_spi.trades.size() > trades_before; }, "RTN_TRADE for the closing sell");
    require_no_step_error(td_spi);
    {
        std::lock_guard<std::mutex> g(td_spi.sync.mu);
        const auto& t = td_spi.trades.back();
        if (!close_double(t.Price, SELL_PRICE) || t.Volume != LOTS || t.Direction != '1' || t.OffsetFlag != '1') {
            throw DemoFail{"unexpected closing trade payload"};
        }
    }

    // -- account after the round trip ------------------------------------------
    qry_with_retry(
        td_spi, "second OnRspQryTradingAccount last",
        [&] {
            td_spi.qry_account.clear();
            td_spi.qry_account_last = false;
            td_spi.qry_throttled = false;
        },
        [&](int rid) {
            CThostFtdcQryTradingAccountField q{};
            put_cstr(q.BrokerID, sizeof(q.BrokerID), broker);
            put_cstr(q.InvestorID, sizeof(q.InvestorID), investor);
            td->ReqQryTradingAccount(&q, rid);
        },
        [&] { return td_spi.qry_account_last; });
    require_no_step_error(td_spi);
    {
        std::lock_guard<std::mutex> g(td_spi.sync.mu);
        if (td_spi.qry_account.empty()) throw DemoFail{"account row missing after close"};
        const auto& a = td_spi.qry_account[0];
        char buf[256];
        std::snprintf(buf, sizeof(buf),
                      "after close: balance=%.4f close_profit=%.2f margin=%.2f available=%.2f", a.Balance,
                      a.CloseProfit, a.CurrMargin, a.Available);
        td_spi.note(buf);
        const double expected = 2000000.0 - 2 * COMMISSION + (SELL_PRICE - BUY_PRICE) * VOLUME_MULTIPLE;
        if (!close_double(a.Balance, expected) || !close_double(a.CurrMargin, 0.0)) {
            throw DemoFail{std::string("account mismatch after close: balance=") + std::to_string(a.Balance) +
                           " expected=" + std::to_string(expected) + " margin=" + std::to_string(a.CurrMargin)};
        }
        if (!close_double(a.CloseProfit, (SELL_PRICE - BUY_PRICE) * VOLUME_MULTIPLE, 1e-6)) {
            throw DemoFail{"CloseProfit mismatch"};
        }
    }

    if (td_spi.stray_error) throw DemoFail{"stray OnRspError on the trader channel"};

    // -- CTP in-flight query gate (docs: 查询流控): while one query stream is
    //    outstanding the vendor API rejects the next ReqQry* locally with -2
    //    (未处理请求超过许可数) -- it never reaches the wire. Sleep past the
    //    per-second window first so the front itself is willing, isolating
    //    the client-side gate from the front-side budget.
    std::this_thread::sleep_for(std::chrono::milliseconds(1200));
    {
        int rid_a;
        {
            std::lock_guard<std::mutex> g(td_spi.sync.mu);
            td_spi.qry_orders.clear();
            td_spi.qry_order_last = false;
            td_spi.qry_throttled = false;
            rid_a = ++g_req_seq;
        }
        CThostFtdcQryOrderField qa{};
        put_cstr(qa.BrokerID, sizeof(qa.BrokerID), broker);
        put_cstr(qa.InvestorID, sizeof(qa.InvestorID), investor);
        td->ReqQryOrder(&qa, rid_a);  // accepted: this query is now in flight
        CThostFtdcQryInstrumentField qb{};
        put_cstr(qb.InstrumentID, sizeof(qb.InstrumentID), INSTRUMENT);
        int rc = td->ReqQryInstrument(&qb, rid_a + 1);
        if (rc != -2) {
            throw DemoFail{std::string("concurrent query not rejected with -2, rc=") + std::to_string(rc)};
        }
        td_spi.note("second concurrent query rejected rc=-2 (in-flight limit)");
        td_spi.sync.wait([&] { return td_spi.qry_order_last && !td_spi.qry_throttled; },
                         "OnRspQryOrder last after the -2 probe");
        require_no_step_error(td_spi);
        std::lock_guard<std::mutex> g(td_spi.sync.mu);
        if (td_spi.qry_orders.size() < 2) throw DemoFail{"qry order stream missing rows after the probe"};
    }

    // -- logout -----------------------------------------------------------------
    {
        CThostFtdcUserLogoutField lo{};
        put_cstr(lo.BrokerID, sizeof(lo.BrokerID), broker);
        put_cstr(lo.UserID, sizeof(lo.UserID), investor);
        td->ReqUserLogout(&lo, ++g_req_seq);
    }
    td_spi.sync.wait([&] { return td_spi.logout_done; }, "OnRspUserLogout");
    require_no_step_error(td_spi);

    char buf[160];
    std::snprintf(buf, sizeof(buf), "flow complete: %zu rtn orders, %zu rtn trades, %d md ticks", td_spi.orders.size(),
                  td_spi.trades.size(), md_spi.tick_count);
    td_spi.note(buf);
}

}  // namespace

int main(int argc, char** argv) {
    const char* front = argc > 1 ? argv[1] : "tcp://127.0.0.1:8001";
    const char* broker = argc > 2 ? argv[2] : "8888";
    const char* investor = argc > 3 ? argv[3] : "smoke001";
    std::printf("[demo] front=%s broker=%s investor=%s\n", front, broker, investor);
    try {
        run(front, broker, investor);
    } catch (const DemoFail& e) {
        std::printf("DEMO: FAIL: %s\n", e.what());
        return 1;
    } catch (const std::exception& e) {
        std::printf("DEMO: FAIL: unexpected exception: %s\n", e.what());
        return 1;
    }
    std::printf("DEMO: PASS\n");
    return 0;
}
