// CTPBuddy CTP ABI shim -- shared core implementation. See api_core.hpp for
// the protocol notes. Threading model (mirrors the vendor API):
//   * the app thread calls Req*/Register*/Init/Release;
//   * one reader thread owns the socket: connect/retry, frame reads and ALL
//     SPI callbacks (so callbacks never run on the app's thread);
//   * `mu_` guards shared state; dispatch rows lock only to extract data and
//     invoke SPI callbacks with the lock RELEASED (apps routinely call back
//     into the API from a callback -- e.g. ReqQryOrder inside OnRspUserLogin).
#include "api_core.hpp"

// compile-time layout contract: every CTP struct the wire protocol carries
// must match the generated mirrors bit for bit (519 static_asserts).
#include "generated/registry.hpp"

#include <chrono>
#include <cstdio>
#include <cstdlib>

namespace ctpbuddy {

namespace {

// One WSAStartup/WSACleanup pair per DLL; both shim DLLs link this file.
struct WinsockInit {
    WinsockInit() {
        WSADATA d;
        WSAStartup(MAKEWORD(2, 2), &d);
    }
    ~WinsockInit() { WSACleanup(); }
};
WinsockInit g_winsock;

// Front flow-control values reported through GetFrontInfo (CTP 6.7.13 API
// docs, 报单流控、查询流控和会话数控制: QryFreq is configured on the front
// component; the Rust core enforces the same default, overridable with
// --qry-freq / CTPBUDDY_QRY_FREQ). Override the reported value with the
// CTPBUDDY_QRY_FREQ environment variable.
int qry_freq_env() {
    const char* v = std::getenv("CTPBUDDY_QRY_FREQ");
    if (v && *v) {
        int n = std::atoi(v);
        if (n > 0) return n;
    }
    return 2;
}
const int kQryFreq = qry_freq_env();
const int kFtdPkgFreq = 6;

}  // namespace

ApiCore::ApiCore() = default;

ApiCore::~ApiCore() {
    if (!released_.load()) {
        stopped_ = true;
        {
            std::lock_guard<std::mutex> g(mu_);
            close_socket_locked();
        }
        if (reader_.joinable()) reader_.join();
    }
    pending_.clear();
}

// ---- registration -----------------------------------------------------------

void ApiCore::parse_address(const std::string& addr, std::string& host, int& port) {
    std::string s = addr;
    const std::string tcp = "tcp://";
    if (s.rfind(tcp, 0) == 0) s = s.substr(tcp.size());
    auto colon = s.rfind(':');
    if (colon == std::string::npos || s.empty()) {
        host.clear();
        port = 0;
        return;
    }
    host = s.substr(0, colon);
    port = atoi(s.c_str() + colon + 1);
    if (port <= 0 || port > 65535) port = 0;
}

void ApiCore::core_register_front(const char* addr) {
    std::lock_guard<std::mutex> g(mu_);
    front_addr_ = addr ? addr : "";
    if (front_addr_.empty()) {
        front_host_.clear();
        front_port_ = 0;
        return;
    }
    parse_address(front_addr_, front_host_, front_port_);
}

void ApiCore::core_register_name_server(const char* addr) {
    std::lock_guard<std::mutex> g(mu_);
    ns_addr_ = addr ? addr : "";
    // No name-server resolution in M1: if the app only registered a name
    // server, treat it as the front (best effort).
    if (front_addr_.empty() && !ns_addr_.empty()) {
        parse_address(ns_addr_, front_host_, front_port_);
    }
}

void ApiCore::core_register_fens(CThostFtdcFensUserInfoField* info) {
    if (!info) return;
    std::lock_guard<std::mutex> g(mu_);
    // Fens identity is the pre-login AUTH identity when the app provides one.
    std::string broker = cstr_of(info->BrokerID, sizeof(info->BrokerID));
    std::string user = cstr_of(info->UserID, sizeof(info->UserID));
    if (!broker.empty() && !user.empty()) {
        auth_broker_ = broker;
        auth_user_ = user;
    }
}

// ---- lifecycle --------------------------------------------------------------

void ApiCore::core_init() {
    bool expected = false;
    if (!started_.compare_exchange_strong(expected, true)) return;  // Init once
    stopped_ = false;
    reader_ = std::thread([this] { reader_main(); });
}

int ApiCore::core_join() {
    if (reader_.joinable()) reader_.join();
    return 0;
}

void ApiCore::core_release() {
    bool expected = false;
    if (!released_.compare_exchange_strong(expected, true)) return;  // once
    stopped_ = true;
    {
        std::lock_guard<std::mutex> g(mu_);
        close_socket_locked();
    }
    if (reader_.joinable()) reader_.join();
    delete this;  // CTP contract: Release deletes the API object
}

const char* ApiCore::core_trading_day() {
    // stable pointer; only the reader thread (under mu_) writes it
    std::lock_guard<std::mutex> g(mu_);
    return trading_day_;
}

void ApiCore::core_front_info(CThostFtdcFrontInfoField* out) {
    if (!out) return;
    std::lock_guard<std::mutex> g(mu_);
    memset(out, 0, sizeof(*out));
    set_cstr(out->FrontAddr, sizeof(out->FrontAddr), front_addr_);
    out->QryFreq = kQryFreq;     // per-second query budget this shim reports
    out->FTDPkgFreq = kFtdPkgFreq;
}

// ---- socket plumbing --------------------------------------------------------

bool ApiCore::connect_front() {
    std::string host;
    int port = 0;
    {
        std::lock_guard<std::mutex> g(mu_);
        host = front_host_;
        port = front_port_;
    }
    if (host.empty() || port == 0) return false;
    SOCKET s = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
    if (s == INVALID_SOCKET) return false;
    sockaddr_in sa{};
    sa.sin_family = AF_INET;
    sa.sin_port = htons(static_cast<u_short>(port));
    if (inet_pton(AF_INET, host.c_str(), &sa.sin_addr) != 1) {
        closesocket(s);
        return false;
    }
    if (connect(s, reinterpret_cast<sockaddr*>(&sa), sizeof(sa)) != 0) {
        closesocket(s);
        return false;
    }
    {
        std::lock_guard<std::mutex> g(mu_);
        if (sock_ != INVALID_SOCKET) closesocket(sock_);
        sock_ = s;
    }
    return true;
}

bool ApiCore::read_exact(uint8_t* buf, size_t n) {
    size_t got = 0;
    while (got < n) {
        SOCKET s;
        {
            std::lock_guard<std::mutex> g(mu_);
            s = sock_;
        }
        if (s == INVALID_SOCKET) return false;
        int r = recv(s, reinterpret_cast<char*>(buf + got), static_cast<int>(n - got), 0);
        if (r > 0) {
            got += static_cast<size_t>(r);
            continue;
        }
        if (r == 0) return false;  // orderly shutdown
        int err = WSAGetLastError();
        if (err == WSAEINTR) continue;
        return false;
    }
    return true;
}

void ApiCore::close_socket_locked() {
    // caller holds mu_ OR we are single-threaded (dtor path)
    if (sock_ != INVALID_SOCKET) {
        closesocket(sock_);
        sock_ = INVALID_SOCKET;
    }
}

void ApiCore::write_frame_locked(const Frame& f) {
    // caller holds mu_; sock_ may be invalid while connecting -- drop.
    if (sock_ == INVALID_SOCKET) return;
    std::vector<uint8_t> bytes = f.encode();
    size_t sent = 0;
    while (sent < bytes.size()) {
        int r = send(sock_, reinterpret_cast<const char*>(bytes.data() + sent),
                     static_cast<int>(bytes.size() - sent), 0);
        if (r > 0) {
            sent += static_cast<size_t>(r);
            continue;
        }
        int err = WSAGetLastError();
        if (err == WSAEINTR) continue;
        break;  // reader thread will notice the dead socket
    }
}

void ApiCore::send_auth_locked(const char* broker, const char* user) {
    // caller holds mu_
    auth_in_flight_ = true;
    Frame f;
    f.msg_type = msgs::AUTH;
    f.req_id = next_req_id_++;
    std::string body = "{\"broker_id\":\"" + std::string(broker) +
                       "\",\"user_id\":\"" + std::string(user) +
                       "\",\"app_id\":\"ctpbuddy-shim\"}";
    f.payload.assign(body.begin(), body.end());
    write_frame_locked(f);
}

// ---- reader thread ----------------------------------------------------------

void ApiCore::reader_main() {
    while (!stopped_.load()) {
        if (!connect_front()) {
            sleep_chunks(1.0);
            continue;
        }
        {
            std::lock_guard<std::mutex> g(mu_);
            authed_ = false;
            if (!auth_user_.empty()) send_auth_locked(auth_broker_.c_str(), auth_user_.c_str());
        }
        fire_front_connected();

        uint8_t header[WIRE_HEADER_LEN];
        bool alive = true;
        while (!stopped_.load()) {
            if (!read_exact(header, WIRE_HEADER_LEN)) {
                alive = false;
                break;
            }
            if (header[0] != 0x43 || header[1] != 0x42 || header[2] != WIRE_VERSION) {
                alive = false;
                break;
            }
            Frame f;
            f.msg_type = static_cast<uint16_t>(header[3] | (header[4] << 8));
            f.req_id = static_cast<uint32_t>(header[5]) | (static_cast<uint32_t>(header[6]) << 8) |
                       (static_cast<uint32_t>(header[7]) << 16) | (static_cast<uint32_t>(header[8]) << 24);
            uint32_t len = static_cast<uint32_t>(header[9]) | (static_cast<uint32_t>(header[10]) << 8) |
                           (static_cast<uint32_t>(header[11]) << 16) | (static_cast<uint32_t>(header[12]) << 24);
            if (len > WIRE_MAX_PAYLOAD) {
                alive = false;
                break;
            }
            f.payload.resize(len);
            if (len > 0 && !read_exact(f.payload.data(), len)) {
                alive = false;
                break;
            }
            on_frame(f);
        }
        {
            std::lock_guard<std::mutex> g(mu_);
            close_socket_locked();
            authed_ = false;
            // an in-flight query can no longer complete on this socket
            qry_in_flight_ = false;
        }
        if (alive && !stopped_.load()) fire_front_disconnected(0x1001);
        sleep_chunks(1.0);  // reconnect backoff
    }
}

void ApiCore::sleep_chunks(double seconds) {
    const auto step = std::chrono::milliseconds(100);
    auto deadline = std::chrono::steady_clock::now() + std::chrono::duration_cast<std::chrono::milliseconds>(
                                                               std::chrono::duration<double>(seconds));
    while (!stopped_.load()) {
        if (std::chrono::steady_clock::now() >= deadline) return;
        std::this_thread::sleep_for(step);
    }
}

// ---- frame routing ----------------------------------------------------------

void ApiCore::on_frame(const Frame& f) {
    switch (f.msg_type) {
        case msgs::AUTH_RSP:
            on_auth_rsp(f);
            return;
        case msgs::RSP_ERROR:
            on_rsp_error(f);
            return;
        case msgs::QRY_LAST:
            on_qry_last(f);
            return;
        default:
            break;
    }
    const DispatchRow* row = find_row(f.msg_type);
    if (row && row->fn) row->fn(*this, f);
    // otherwise: protocol noise from a future core version -- ignore.
}

const DispatchRow* ApiCore::find_row(uint16_t msg) const {
    const DispatchRow* rows = this->rows();
    if (!rows) return nullptr;
    for (const DispatchRow* r = rows; r->msg != 0; ++r) {
        if (r->msg == msg) return r;
    }
    return nullptr;
}

const DispatchRow* ApiCore::find_row_by_req(uint16_t req_msg) const {
    const DispatchRow* rows = this->rows();
    if (!rows) return nullptr;
    for (const DispatchRow* r = rows; r->msg != 0; ++r) {
        if (r->req_msg == req_msg) return r;
    }
    return nullptr;
}

bool ApiCore::is_query_msg(uint16_t msg) {
    switch (msg) {
        case msgs::REQ_QRY_INSTRUMENT:
        case msgs::REQ_QRY_TRADING_ACCOUNT:
        case msgs::REQ_QRY_INVESTOR_POSITION:
        case msgs::REQ_QRY_INVESTOR_POSITION_DETAIL:
        case msgs::REQ_QRY_ORDER:
        case msgs::REQ_QRY_TRADE:
        case msgs::REQ_QRY_INSTRUMENT_MARGIN_RATE:
        case msgs::REQ_QRY_INSTRUMENT_COMMISSION_RATE:
        case msgs::REQ_QRY_INSTRUMENT_ORDER_COMM_RATE:
        case msgs::REQ_QRY_BROKER_TRADING_PARAMS:
            return true;
        default:
            return false;
    }
}

void ApiCore::on_auth_rsp(const Frame& f) {
    std::string body(reinterpret_cast<const char*>(f.payload.data()), f.payload.size());
    bool ok = body.find("\"ok\":true") != std::string::npos ||
              body.find("\"ok\": true") != std::string::npos;
    // best-effort error extraction for diagnostics
    std::string err;
    {
        const std::string key = "\"error\":";
        auto p = body.find(key);
        if (p != std::string::npos) {
            p += key.size();
            if (p < body.size() && body[p] == '"') {
                ++p;
                auto q = body.find('"', p);
                if (q != std::string::npos) err = body.substr(p, q - p);
            }
        }
    }

    Frame stashed;
    int nrid = -1;
    bool have_stash = false;
    {
        std::lock_guard<std::mutex> g(mu_);
        auth_in_flight_ = false;
        // a genuine AUTH rejection leaves authed_ false; a late "already
        // authenticated" reply to a duplicate AUTH must not downgrade us.
        if (ok) authed_ = true;
        if (has_stashed_login_) {
            stashed = stashed_login_;
            has_stashed_login_ = false;
            have_stash = true;
            if (!ok) nrid = take_pending(stashed.req_id).n_request_id;
        }
    }
    if (ok) {
        if (have_stash) {
            std::lock_guard<std::mutex> g(mu_);
            write_frame_locked(stashed);
        }
        return;
    }
    CThostFtdcRspInfoField rsp{};
    rsp.ErrorID = 63;  // CTTrading: 校验失败
    if (err.empty()) err = "CTPBuddy AUTH 失败";
    set_cstr(rsp.ErrorMsg, sizeof(rsp.ErrorMsg), err);
    if (have_stash && nrid >= 0) {
        on_auth_failed(nrid, rsp);
    }
}

void ApiCore::on_rsp_error(const Frame& f) {
    CThostFtdcRspInfoField rsp{};
    payload_as(f, rsp);
    Pending pd;
    const DispatchRow* row = nullptr;
    {
        std::lock_guard<std::mutex> g(mu_);
        pd = take_pending(f.req_id);
        if (pd.req_msg != 0) row = find_row_by_req(pd.req_msg);
        if (pd.req_msg != 0 && is_query_msg(pd.req_msg)) qry_in_flight_ = false;
    }
    if (pd.req_msg != 0) {
        if (row && row->err) row->err(*this, rsp, pd.n_request_id, pd);
        else on_rsp_error_fallback(rsp, pd.n_request_id);
    } else {
        on_rsp_error_fallback(rsp, -1);
    }
}

void ApiCore::on_qry_last(const Frame& f) {
    Pending pd;
    const DispatchRow* row = nullptr;
    {
        std::lock_guard<std::mutex> g(mu_);
        pd = take_pending(f.req_id);
        if (pd.req_msg != 0) row = find_row_by_req(pd.req_msg);
        if (pd.req_msg != 0 && is_query_msg(pd.req_msg)) qry_in_flight_ = false;
    }
    if (pd.req_msg != 0 && row && row->qry_last) {
        row->qry_last(*this, pd.n_request_id);
    }
}

// ---- request surface --------------------------------------------------------

int ApiCore::send_request(uint16_t msg, const void* payload, size_t len, int n_request_id) {
    Frame f;
    f.msg_type = msg;
    f.payload.assign(static_cast<const uint8_t*>(payload),
                     static_cast<const uint8_t*>(payload) + len);

    int retired_nrid = -1;  // an older stashed login retired by this one
    CThostFtdcRspInfoField retired_rsp{};
    {
        std::lock_guard<std::mutex> g(mu_);
        if (stopped_.load()) return -1;
        // CTP 查询流控 (docs: 报单流控、查询流控和会话数控制): the vendor API
        // allows exactly one in-flight ReqQry* per session; a second query
        // before the first stream completes fails locally with -2
        // (未处理请求超过许可数) and is never put on the wire.
        if (is_query_msg(msg)) {
            if (qry_in_flight_) return -2;
            qry_in_flight_ = true;
        }
        f.req_id = next_req_id_++;
        Pending pd;
        pd.req_id = f.req_id;
        pd.req_msg = msg;
        pd.n_request_id = n_request_id;
        if (msg == msgs::REQ_ORDER_INSERT && len == sizeof(CThostFtdcInputOrderField)) {
            memcpy(&pd.input_order, payload, sizeof(pd.input_order));
        } else if (msg == msgs::REQ_ORDER_ACTION && len == sizeof(CThostFtdcInputOrderActionField)) {
            memcpy(&pd.input_action, payload, sizeof(pd.input_action));
        } else if ((msg == msgs::SUB_MD || msg == msgs::UNSUB_MD) &&
                   len >= sizeof(CThostFtdcSpecificInstrumentField)) {
            pd.expected_responses = static_cast<uint32_t>(len / sizeof(CThostFtdcSpecificInstrumentField));
        }
        pending_[f.req_id] = pd;

        if (msg == msgs::REQ_USER_LOGIN && !authed_) {
            if (len == sizeof(CThostFtdcReqUserLoginField)) {
                const auto* req = static_cast<const CThostFtdcReqUserLoginField*>(payload);
                auth_broker_ = cstr_of(req->BrokerID, sizeof(req->BrokerID));
                auth_user_ = cstr_of(req->UserID, sizeof(req->UserID));
            }
            if (has_stashed_login_) {
                // an earlier login is still waiting for AUTH: retire it now
                retired_nrid = take_pending(stashed_login_.req_id).n_request_id;
                retired_rsp.ErrorID = -3;
                set_cstr(retired_rsp.ErrorMsg, sizeof(retired_rsp.ErrorMsg), "重复的登录请求");
            }
            stashed_login_ = f;
            has_stashed_login_ = true;
            if (!auth_in_flight_) {
                auth_in_flight_ = true;
                send_auth_locked(auth_broker_.c_str(), auth_user_.c_str());
            }
        } else {
            write_frame_locked(f);
        }
    }
    if (retired_nrid >= 0) on_auth_failed(retired_nrid, retired_rsp);
    return 0;
}

void ApiCore::unsupported(int n_request_id, const char* method) {
    CThostFtdcRspInfoField rsp{};
    rsp.ErrorID = -1;
    set_cstr(rsp.ErrorMsg, sizeof(rsp.ErrorMsg), std::string("CTPBuddy 尚未实现 ") + method);
    on_rsp_error_fallback(rsp, n_request_id);
}

// ---- pending helpers --------------------------------------------------------

Pending ApiCore::take_pending(uint32_t req_id) {
    auto it = pending_.find(req_id);
    if (it == pending_.end()) return Pending{};
    Pending pd = it->second;
    pending_.erase(it);
    return pd;
}

Pending* ApiCore::find_pending(uint32_t req_id) {
    auto it = pending_.find(req_id);
    return it == pending_.end() ? nullptr : &it->second;
}

void ApiCore::erase_pending(uint32_t req_id) { pending_.erase(req_id); }

void ApiCore::set_trading_day_locked(const char* day) {
    set_cstr(trading_day_, sizeof(trading_day_), cstr_of(day, 9));
}

}  // namespace ctpbuddy
