// CTPBuddy CTP ABI shim -- shared core: wire transport, session plumbing and
// frame->SPI dispatch. The generated classes (TraderApi / MdApi) inherit this
// and only supply the typed SPI callbacks (dispatch_td.cpp / dispatch_md.cpp).
//
// Wire protocol (DESIGN.md §6.3, core/ctpbuddy-wire):
//   magic 'CB' + ver(u8) + type(u16 LE) + req_id(u32 LE) + len(u32 LE) + payload
// CTP structs use natural alignment on every side (gen_structs.py), so request
// and response payloads are raw memcpy -- no field-by-field conversion.
#pragma once

#include <atomic>
#include <cstdint>
#include <cstring>
#include <functional>
#include <mutex>
#include <string>
#include <thread>
#include <unordered_map>
#include <vector>

#include <winsock2.h>
#include <ws2tcpip.h>

#include "ThostFtdcUserApiStruct.h"

#pragma comment(lib, "ws2_32.lib")

namespace ctpbuddy {

// ---- wire message ids: mirror of core/ctpbuddy-wire/src/msgs.rs -------------
namespace msgs {
constexpr uint16_t PING = 0x0001;
constexpr uint16_t PONG = 0x0002;
constexpr uint16_t AUTH = 0x0101;
constexpr uint16_t AUTH_RSP = 0x0102;
constexpr uint16_t REQ_USER_LOGIN = 0x1001;
constexpr uint16_t RSP_USER_LOGIN = 0x1002;
constexpr uint16_t REQ_USER_LOGOUT = 0x1003;
constexpr uint16_t RSP_USER_LOGOUT = 0x1004;
constexpr uint16_t REQ_SETTLE_CONFIRM = 0x1005;
constexpr uint16_t RSP_SETTLE_CONFIRM = 0x1006;
constexpr uint16_t REQ_ORDER_INSERT = 0x1010;
constexpr uint16_t RSP_ORDER_INSERT = 0x1011;
constexpr uint16_t ERR_RTN_ORDER_INSERT = 0x1012;
constexpr uint16_t RTN_ORDER = 0x1013;
constexpr uint16_t RTN_TRADE = 0x1014;
constexpr uint16_t REQ_ORDER_ACTION = 0x1015;
constexpr uint16_t RSP_ORDER_ACTION = 0x1016;
constexpr uint16_t ERR_RTN_ORDER_ACTION = 0x1017;
constexpr uint16_t SUB_MD = 0x1020;
constexpr uint16_t RSP_SUB_MD = 0x1021;
constexpr uint16_t RTN_DEPTH_MD = 0x1022;
constexpr uint16_t UNSUB_MD = 0x1023;
constexpr uint16_t RSP_UNSUB_MD = 0x1024;
constexpr uint16_t RSP_ERROR = 0x1030;
constexpr uint16_t REQ_QRY_SETTLEMENT_INFO = 0x1031;
constexpr uint16_t RSP_QRY_SETTLEMENT_INFO = 0x1032;
constexpr uint16_t REQ_QRY_INSTRUMENT = 0x1040;
constexpr uint16_t RSP_QRY_INSTRUMENT = 0x1041;
constexpr uint16_t REQ_QRY_TRADING_ACCOUNT = 0x1042;
constexpr uint16_t RSP_QRY_TRADING_ACCOUNT = 0x1043;
constexpr uint16_t REQ_QRY_INVESTOR_POSITION = 0x1044;
constexpr uint16_t RSP_QRY_INVESTOR_POSITION = 0x1045;
constexpr uint16_t REQ_QRY_ORDER = 0x1046;
constexpr uint16_t RSP_QRY_ORDER = 0x1047;
constexpr uint16_t REQ_QRY_TRADE = 0x1048;
constexpr uint16_t RSP_QRY_TRADE = 0x1049;
constexpr uint16_t QRY_LAST = 0x1050;
// Reference-data queries (notes/04 G). Kept in lockstep with the core's
// msgs.rs and codegen/gen_shim.py — three places, same numbers.
constexpr uint16_t REQ_QRY_INSTRUMENT_MARGIN_RATE = 0x1051;
constexpr uint16_t RSP_QRY_INSTRUMENT_MARGIN_RATE = 0x1052;
constexpr uint16_t REQ_QRY_INSTRUMENT_COMMISSION_RATE = 0x1053;
constexpr uint16_t RSP_QRY_INSTRUMENT_COMMISSION_RATE = 0x1054;
constexpr uint16_t REQ_QRY_INSTRUMENT_ORDER_COMM_RATE = 0x1055;
constexpr uint16_t RSP_QRY_INSTRUMENT_ORDER_COMM_RATE = 0x1056;
constexpr uint16_t REQ_QRY_BROKER_TRADING_PARAMS = 0x1057;
constexpr uint16_t RSP_QRY_BROKER_TRADING_PARAMS = 0x1058;
constexpr uint16_t REQ_QRY_INVESTOR_POSITION_DETAIL = 0x1059;
constexpr uint16_t RSP_QRY_INVESTOR_POSITION_DETAIL = 0x105A;
constexpr uint16_t REQ_QRY_INVESTOR_PRODUCT_GROUP_MARGIN = 0x105B;
constexpr uint16_t RSP_QRY_INVESTOR_PRODUCT_GROUP_MARGIN = 0x105C;
constexpr uint16_t LOGOUT = 0x0103;
constexpr uint16_t LOGOUT_RSP = 0x0104;
constexpr uint16_t ADMIN_REQ = 0x0201;
constexpr uint16_t ADMIN_RSP = 0x0202;
}  // namespace msgs

class ApiCore;  // completed at the bottom of this header

constexpr size_t WIRE_HEADER_LEN = 13;
constexpr uint8_t WIRE_VERSION = 1;
constexpr size_t WIRE_MAX_PAYLOAD = 1 << 20;

struct Frame {
    uint16_t msg_type = 0;
    uint32_t req_id = 0;
    std::vector<uint8_t> payload;

    std::vector<uint8_t> encode() const {
        std::vector<uint8_t> v(WIRE_HEADER_LEN + payload.size());
        v[0] = 0x43;  // 'C'
        v[1] = 0x42;  // 'B'
        v[2] = WIRE_VERSION;
        v[3] = static_cast<uint8_t>(msg_type & 0xff);
        v[4] = static_cast<uint8_t>(msg_type >> 8);
        v[5] = static_cast<uint8_t>(req_id & 0xff);
        v[6] = static_cast<uint8_t>((req_id >> 8) & 0xff);
        v[7] = static_cast<uint8_t>((req_id >> 16) & 0xff);
        v[8] = static_cast<uint8_t>((req_id >> 24) & 0xff);
        uint32_t len = static_cast<uint32_t>(payload.size());
        v[9] = static_cast<uint8_t>(len & 0xff);
        v[10] = static_cast<uint8_t>((len >> 8) & 0xff);
        v[11] = static_cast<uint8_t>((len >> 16) & 0xff);
        v[12] = static_cast<uint8_t>((len >> 24) & 0xff);
        if (!payload.empty()) memcpy(v.data() + WIRE_HEADER_LEN, payload.data(), payload.size());
        return v;
    }
};

/// One in-flight request. `req_msg == 0` means "not found".
struct Pending {
    uint32_t req_id = 0;
    uint16_t req_msg = 0;
    int n_request_id = -1;
    // cached request payloads for the ErrRtn paths (CTP passes the client's
    // own input struct back on failure)
    CThostFtdcInputOrderField input_order{};
    CThostFtdcInputOrderActionField input_action{};
    CThostFtdcRspAuthenticateField rsp_authenticate{};
    // subscribe responses: the core answers once per instrument; bIsLast on
    // the final one, counted against what the shim sent.
    uint32_t expected_responses = 0;
    uint32_t responses = 0;
};

/// wire msg -> SPI callback row (generated: dispatch_td.cpp / dispatch_md.cpp).
struct DispatchRow {
    uint16_t msg;      // response/push frame this row serves
    void (*fn)(ApiCore&, const Frame&);
    void (*qry_last)(ApiCore&, int n_request_id);  // non-null for query rows
    void (*err)(ApiCore&, const CThostFtdcRspInfoField&, int n_request_id, const Pending&);
    // the REQUEST frame whose pending entry this row completes (0 for pushes):
    // RSP_ERROR / QRY_LAST carry only the original req_id, so the pending
    // lookup must go request-type -> row, not response-type -> row.
    uint16_t req_msg = 0;
};

/// Copy a frame payload into `storage` when sizes match; NULL otherwise
/// (CTP callbacks receive NULL for "no data" responses).
template <typename S>
const S* payload_as(const Frame& f, S& storage) {
    if (f.payload.size() == sizeof(S)) {
        memcpy(&storage, f.payload.data(), sizeof(S));
        return &storage;
    }
    return nullptr;
}

/// Fill a CThostFtdcOrderActionField from a cached InputOrderActionField
/// (ERR_RTN_ORDER_ACTION carries the exchange's view of the action).
inline void synth_order_action(CThostFtdcOrderActionField& a,
                               const CThostFtdcInputOrderActionField& in) {
    memset(&a, 0, sizeof(a));
    memcpy(a.BrokerID, in.BrokerID, sizeof(a.BrokerID));
    memcpy(a.InvestorID, in.InvestorID, sizeof(a.InvestorID));
    a.OrderActionRef = in.OrderActionRef;  // int, not an array
    memcpy(a.OrderRef, in.OrderRef, sizeof(a.OrderRef));
    a.RequestID = in.RequestID;
    a.FrontID = in.FrontID;
    a.SessionID = in.SessionID;
    memcpy(a.ExchangeID, in.ExchangeID, sizeof(a.ExchangeID));
    memcpy(a.OrderSysID, in.OrderSysID, sizeof(a.OrderSysID));
    a.ActionFlag = in.ActionFlag;  // char, not an array
    a.LimitPrice = in.LimitPrice;
    a.VolumeChange = in.VolumeChange;
    memcpy(a.UserID, in.UserID, sizeof(a.UserID));
    memcpy(a.InstrumentID, in.InstrumentID, sizeof(a.InstrumentID));
}

inline std::string cstr_of(const char* buf, size_t n) {
    size_t end = 0;
    while (end < n && buf[end] != 0) ++end;
    return std::string(buf, end);
}

inline void set_cstr(char* buf, size_t n, const std::string& s) {
    size_t k = s.size() < n ? s.size() : n;
    memcpy(buf, s.data(), k);
    memset(buf + k, 0, n - k);
}

/// Write a message authored in this source tree (UTF-8 literal, /utf-8) into a
/// CTP char field as GBK -- the encoding every CTP client decodes. Truncates
/// on a character boundary and always leaves a terminating NUL.
inline void set_text(char* buf, size_t n, const std::string& utf8) {
    std::string gbk;
    int wlen = MultiByteToWideChar(CP_UTF8, 0, utf8.data(), static_cast<int>(utf8.size()), nullptr, 0);
    if (wlen > 0) {
        std::wstring wide(static_cast<size_t>(wlen), L'\0');
        MultiByteToWideChar(CP_UTF8, 0, utf8.data(), static_cast<int>(utf8.size()), &wide[0], wlen);
        size_t used = 0;
        for (wchar_t ch : wide) {
            char tmp[4];
            int k = WideCharToMultiByte(936, 0, &ch, 1, tmp, sizeof(tmp), "?", nullptr);
            if (k <= 0 || used + static_cast<size_t>(k) + 1 > n) break;
            gbk.append(tmp, static_cast<size_t>(k));
            used += static_cast<size_t>(k);
        }
    }
    set_cstr(buf, n, gbk);
}

/// Everything shared by the two API shims. Not copyable; lifetime is managed
/// through Release() (which does `delete this`), matching the CTP contract.
class ApiCore {public:
    virtual ~ApiCore();

    // implemented by the generated classes
    virtual const DispatchRow* rows() const = 0;
    virtual void on_rsp_error_fallback(const CThostFtdcRspInfoField& rsp, int n_request_id) = 0;
    virtual void fire_front_connected() = 0;
    virtual void fire_front_disconnected(int reason) = 0;
    virtual void on_auth_failed(int n_request_id, const CThostFtdcRspInfoField& rsp) = 0;
    virtual void on_authenticate_rsp(const CThostFtdcRspAuthenticateField*, const CThostFtdcRspInfoField&, int) {}

    // ---- registration / lifecycle (generated core forwards call these) ----
    void core_register_front(const char* addr);
    void core_register_name_server(const char* addr);
    void core_register_fens(CThostFtdcFensUserInfoField* info);
    void core_init();
    int core_join();
    void core_release();
    const char* core_trading_day();
    void core_front_info(CThostFtdcFrontInfoField* out);

    // ---- SPI ----
    void set_spi(void* spi) { spi_ = spi; }
    void* spi() const { return spi_; }

    // ---- request surface (generated Req* overrides call these) ----
    // Return codes follow the vendor API: 0 sent, -1 network / bad argument,
    // -2 too many pending requests, -3 per-second flow control.
    template <typename S>
    int send_req(uint16_t msg, const S* p, int n_request_id) {
        if (!p) return -1;  // NULL input struct: never memcpy from it
        return send_request(msg, p, sizeof(S), n_request_id);
    }
    int send_request(uint16_t msg, const void* payload, size_t len, int n_request_id);
    int send_ctp_auth(const CThostFtdcReqAuthenticateField* req, int n_request_id);
    /// Request the M1 core does not implement: answers OnRspError(-1) on the
    /// reader thread (see post_callback). Returns -1 when not connected.
    int unsupported(int n_request_id, const char* method);
    /// Hand a locally-produced SPI callback to the reader thread so apps
    /// never see a callback on their own stack (vendor contract: all SPI
    /// callbacks arrive on the API's internal thread). The reader drains the
    /// queue between frames (<=100ms) and during reconnect backoff. Returns
    /// -1 and drops the callback when no connection exists -- the same
    /// answer the vendor gives a request issued before OnFrontConnected.
    int post_callback(std::function<void()> fn);

    // ---- used by the generated dispatch rows ----
    std::mutex& mu() { return mu_; }
    // CTP SPI callbacks take non-const pointers; apps treat them as read-only
    // (same contract as the vendor API's internal buffers).
    CThostFtdcRspInfoField& zero_rsp_info() { return zero_rsp_; }
    // pending_ accessors: the CALLER must hold mu() (send_request inserts into
    // the same map from the app thread). Rows lock, copy the Pending out,
    // unlock, then invoke the SPI.
    Pending take_pending(uint32_t req_id);
    Pending* find_pending(uint32_t req_id);
    void erase_pending(uint32_t req_id);
    void set_trading_day_locked(const char* day);

    // frame-level routing (reader thread)
    void on_frame(const Frame& f);

protected:
    ApiCore();

private:
    void reader_main();
    bool connect_front();
    bool read_exact(uint8_t* buf, size_t n);
    void close_socket_locked();
    void write_frame_locked(const Frame& f);
    void send_auth_locked(const char* broker, const char* user);
    void drain_auth_errors();
    void drain_deferred();
    void on_auth_rsp(const Frame& f);
    void on_rsp_error(const Frame& f);
    void on_qry_last(const Frame& f);
    const DispatchRow* find_row(uint16_t msg) const;
    const DispatchRow* find_row_by_req(uint16_t req_msg) const;
    void sleep_chunks(double seconds);
    static void parse_address(const std::string& addr, std::string& host, int& port);

    /// True for the ReqQry* family (CTP "查询流控" applies to these; the
    /// ReqQuery* core-processed family and all non-query requests are exempt).
    static bool is_query_msg(uint16_t msg);

    std::mutex mu_;
    void* spi_ = nullptr;
    SOCKET sock_ = INVALID_SOCKET;
    std::thread reader_;
    std::atomic<bool> started_{false};
    std::atomic<bool> stopped_{false};
    std::atomic<bool> released_{false};
    // set when Release() runs on the reader thread itself (inside an SPI
    // callback): the reader deletes the object once its loop unwinds
    bool delete_on_exit_ = false;

    std::string front_addr_;
    std::string ns_addr_;
    std::string front_host_;
    int front_port_ = 0;
    bool authed_ = false;
    bool auth_in_flight_ = false;
    uint32_t auth_wire_req_id_ = 0;
    std::string auth_broker_;
    std::string auth_user_;
    std::string bound_auth_broker_;
    std::string bound_auth_user_;
    std::vector<std::pair<int, CThostFtdcRspInfoField>> login_errors_;
    bool has_stashed_login_ = false;
    Frame stashed_login_;
    std::vector<std::pair<int, CThostFtdcRspInfoField>> auth_errors_;
    // locally-answered callbacks waiting for the reader thread (post_callback)
    std::vector<std::function<void()>> deferred_;
    uint32_t next_req_id_ = 1;
    std::unordered_map<uint32_t, Pending> pending_;
    // CTP 查询流控 (docs: 报单流控、查询流控和会话数控制): the vendor API
    // allows exactly ONE in-flight ReqQry* per session; a second query
    // before the first stream completes fails locally with -2. The
    // per-second QryFreq limit lives on the front (the Rust core answers
    // OnRspError[90] "CTP：查询未就绪，请稍后重试" when it trips).
    bool qry_in_flight_ = false;
    char trading_day_[9] = {0};
    CThostFtdcRspInfoField zero_rsp_{};
};

}  // namespace ctpbuddy
