// CTPBuddy CTP ABI shim -- shared core implementation. See api_core.hpp for
// the protocol notes. Threading model (mirrors the vendor API):
//   * the app thread calls Req*/Register*/Init/Release;
//   * one reader thread owns the socket: connect/retry, frame reads and ALL
//     SPI callbacks (so callbacks never run on the app's thread);
//   * `mu_` guards shared state; dispatch rows lock only to extract data and
//     invoke SPI callbacks with the lock RELEASED (apps routinely call back
//     into the API from a callback -- e.g. ReqQryOrder inside OnRspUserLogin);
//   * callbacks the shim answers LOCALLY (unsupported Req*, duplicate login,
//     bad ReqAuthenticate, MD for-quote stubs) are never invoked on the app's
//     stack either: they go through post_callback() / auth_errors_ /
//     login_errors_ and the reader thread drains them between frames.
#include "api_core.hpp"

// compile-time layout contract: every CTP struct the wire protocol carries
// must match the generated mirrors bit for bit (519 static_asserts).
#include "generated/registry.hpp"

#include <chrono>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <fstream>
#include <string_view>

#ifdef _WIN32
#include <windows.h>
#endif
#ifndef _WIN32
#include <dlfcn.h>
#include <iconv.h>
#endif

namespace ctpbuddy {

namespace {

// libzmq is intentionally loaded at runtime. The shim remains drop-in
// deployable on hosts that only use TCP, while a host opting into zmq:// only
// needs libzmq at runtime. All socket calls stay on the reader thread.
constexpr int ZMQ_SUB = 2;
constexpr int ZMQ_DEALER = 5;
constexpr int ZMQ_DONTWAIT = 1;
constexpr int ZMQ_SNDHWM = 23;
constexpr int ZMQ_RCVHWM = 24;
constexpr int ZMQ_LINGER = 17;
constexpr int ZMQ_SUBSCRIBE = 6;
constexpr int ZMQ_CURVE_SERVERKEY = 50;
constexpr int ZMQ_CURVE_PUBLICKEY = 48;
constexpr int ZMQ_CURVE_SECRETKEY = 49;
constexpr int ZMQ_RCVMORE = 13;
constexpr short ZMQ_POLLIN = 1;

struct ZmqPollItem {
    void* socket;
#ifdef _WIN32
    SOCKET fd;
#else
    int fd;
#endif
    short events;
    short revents;
};

struct ZmqLibrary {
    using CtxNew = void* (*)();
    using CtxTerm = int (*)(void*);
    using Socket = void* (*)(void*, int);
    using Close = int (*)(void*);
    using Connect = int (*)(void*, const char*);
    using Send = int (*)(void*, const void*, size_t, int);
    using Recv = int (*)(void*, void*, size_t, int);
    using SetSockOpt = int (*)(void*, int, const void*, size_t);
    using GetSockOpt = int (*)(void*, int, void*, size_t*);
    using Poll = int (*)(ZmqPollItem*, int, long);

    void* handle = nullptr;
    CtxNew ctx_new = nullptr;
    CtxTerm ctx_term = nullptr;
    Socket socket = nullptr;
    Close close = nullptr;
    Connect connect = nullptr;
    Send send = nullptr;
    Recv recv = nullptr;
    SetSockOpt setsockopt = nullptr;
    GetSockOpt getsockopt = nullptr;
    Poll poll = nullptr;

    bool load() {
        if (handle) return true;
#ifdef _WIN32
        const char* configured = std::getenv("CTPBUDDY_ZMQ_LIBRARY");
        const char* names[] = {configured && *configured ? configured : "libzmq.dll", "zmq.dll", nullptr};
        for (const char** p = names; *p && !handle; ++p) {
            handle = static_cast<void*>(LoadLibraryA(*p));
        }
        auto symbol = [this](const char* name) -> void* {
            return handle ? reinterpret_cast<void*>(GetProcAddress(static_cast<HMODULE>(handle), name)) : nullptr;
        };
#else
        const char* configured = std::getenv("CTPBUDDY_ZMQ_LIBRARY");
        const char* names[] = {configured && *configured ? configured : "libzmq.so.5", "libzmq.so", nullptr};
        for (const char** p = names; *p && !handle; ++p) handle = dlopen(*p, RTLD_NOW | RTLD_LOCAL);
        auto symbol = [this](const char* name) -> void* { return handle ? dlsym(handle, name) : nullptr; };
#endif
        if (!handle) return false;
#define ZMQ_LOAD(name) name = reinterpret_cast<decltype(name)>(symbol("zmq_" #name))
        ZMQ_LOAD(ctx_new);
        ZMQ_LOAD(ctx_term);
        ZMQ_LOAD(socket);
        ZMQ_LOAD(close);
        ZMQ_LOAD(connect);
        ZMQ_LOAD(send);
        ZMQ_LOAD(recv);
        ZMQ_LOAD(setsockopt);
        ZMQ_LOAD(getsockopt);
        ZMQ_LOAD(poll);
#undef ZMQ_LOAD
        return ctx_new && ctx_term && socket && close && connect && send && recv && setsockopt && getsockopt && poll;
    }
};

ZmqLibrary& zmq_library() {
    static ZmqLibrary library;
    return library;
}

bool starts_with(const std::string& value, std::string_view prefix) {
    return value.size() >= prefix.size() && value.compare(0, prefix.size(), prefix.data(), prefix.size()) == 0;
}

bool apply_curve_options(ZmqLibrary& z, void* socket) {
    const char* server = std::getenv("CTPBUDDY_ZMQ_CURVE_SERVER_KEY");
    const char* pub = std::getenv("CTPBUDDY_ZMQ_CURVE_PUBLIC_KEY");
    const char* secret = std::getenv("CTPBUDDY_ZMQ_CURVE_SECRET_KEY");
    const bool any = (server && *server) || (pub && *pub) || (secret && *secret);
    if (!any) return true;
    if (!server || !pub || !secret || std::strlen(server) != 40 || std::strlen(pub) != 40 ||
        std::strlen(secret) != 40) {
        return false;
    }
    return z.setsockopt(socket, ZMQ_CURVE_SERVERKEY, server, 40) == 0 &&
           z.setsockopt(socket, ZMQ_CURVE_PUBLICKEY, pub, 40) == 0 &&
           z.setsockopt(socket, ZMQ_CURVE_SECRETKEY, secret, 40) == 0;
}

std::string trim_ascii(std::string value) {
    const auto first = value.find_first_not_of(" \t\r\n");
    if (first == std::string::npos) return {};
    const auto last = value.find_last_not_of(" \t\r\n");
    return value.substr(first, last - first + 1);
}

std::string resolve_front_alias(const std::string& addr) {
    const char* configured = std::getenv("CTPBUDDY_INI");
    const std::string path = configured && *configured ? configured : "ctpbuddy.ini";
    std::ifstream input(path);
    if (!input) return addr;
    bool in_fronts = false;
    std::string line;
    while (std::getline(input, line)) {
        line = trim_ascii(line);
        if (line.empty() || line[0] == ';' || line[0] == '#') continue;
        if (line.front() == '[' && line.back() == ']') {
            in_fronts = trim_ascii(line.substr(1, line.size() - 2)) == "fronts";
            continue;
        }
        if (!in_fronts) continue;
        const auto equals = line.find('=');
        if (equals == std::string::npos) continue;
        const auto comment = line.find_first_of(";#", equals + 1);
        const std::string key = trim_ascii(line.substr(0, equals));
        const std::string value = trim_ascii(line.substr(equals + 1, comment == std::string::npos
                                                               ? std::string::npos
                                                               : comment - equals - 1));
        if (!key.empty() && key == addr && !value.empty()) return value;
    }
    return addr;
}

#ifdef _WIN32
// One WSAStartup/WSACleanup pair per DLL; both shim DLLs link this file, so a
// process loading td+md runs two pairs. That is correct: Winsock keeps a
// per-process reference count, every WSAStartup must be matched by exactly one
// WSACleanup, and the library is torn down only when the count hits zero.
// The static object below guarantees the pairing per DLL image.
struct WinsockInit {
    WinsockInit() {
        WSADATA d;
        WSAStartup(MAKEWORD(2, 2), &d);
    }
    ~WinsockInit() { WSACleanup(); }
};
WinsockInit g_winsock;

int last_socket_error() { return WSAGetLastError(); }
bool error_is_interrupt(int e) { return e == WSAEINTR; }
#else
int last_socket_error() { return errno; }
bool error_is_interrupt(int e) { return e == EINTR; }
#endif

/// Wait up to `ms` for `s` to become readable: 1 readable (or hangup/error:
/// the following recv reports it), 0 timeout, -1 failure.
int wait_readable(SOCKET s, int ms) {
#ifdef _WIN32
    fd_set readable;
    FD_ZERO(&readable);
    FD_SET(s, &readable);
    timeval timeout{0, ms * 1000};
    int ready = select(0, &readable, nullptr, nullptr, &timeout);
    if (ready == SOCKET_ERROR) return -1;
    return ready > 0 ? 1 : 0;
#else
    // poll(), not select(): select() is undefined for fds >= FD_SETSIZE, which
    // a host process with many open files can easily hand us.
    pollfd p{};
    p.fd = s;
    p.events = POLLIN;
    int ready = poll(&p, 1, ms);
    if (ready < 0) return error_is_interrupt(errno) ? 0 : -1;
    return ready > 0 ? 1 : 0;
#endif
}

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
int ftd_freq_env() {
    const char* v = std::getenv("CTPBUDDY_FTD_MAX_COMM_FLUX");
    return v && *v ? (std::max)(0, std::atoi(v)) : 0;
}
const int kFtdPkgFreq = ftd_freq_env();

}  // namespace

std::string utf8_to_gbk(const std::string& utf8, size_t max_bytes) {
    std::string gbk;
#ifdef _WIN32
    int wlen = MultiByteToWideChar(CP_UTF8, 0, utf8.data(), static_cast<int>(utf8.size()), nullptr, 0);
    if (wlen <= 0) return gbk;
    std::wstring wide(static_cast<size_t>(wlen), L'\0');
    MultiByteToWideChar(CP_UTF8, 0, utf8.data(), static_cast<int>(utf8.size()), &wide[0], wlen);
    for (wchar_t ch : wide) {
        char tmp[4];
        int k = WideCharToMultiByte(936, 0, &ch, 1, tmp, sizeof(tmp), "?", nullptr);
        if (k <= 0 || gbk.size() + static_cast<size_t>(k) > max_bytes) break;
        gbk.append(tmp, static_cast<size_t>(k));
    }
#else
    // glibc names the Windows code page 936 superset "GBK"; one descriptor
    // per call keeps this safe from any thread (error paths only).
    iconv_t cd = iconv_open("GBK", "UTF-8");
    if (cd == reinterpret_cast<iconv_t>(-1)) return gbk;
    size_t i = 0;
    while (i < utf8.size()) {
        const unsigned char lead = static_cast<unsigned char>(utf8[i]);
        size_t clen = lead < 0x80 ? 1 : (lead >> 5) == 0x6 ? 2 : (lead >> 4) == 0xE ? 3 : (lead >> 3) == 0x1E ? 4 : 1;
        if (i + clen > utf8.size()) clen = utf8.size() - i;
        char in[4];
        memcpy(in, utf8.data() + i, clen);
        i += clen;
        char out[8];
        char* ip = in;
        char* op = out;
        size_t il = clen, ol = sizeof(out);
        iconv(cd, nullptr, nullptr, nullptr, nullptr);  // reset shift state
        size_t rc = iconv(cd, &ip, &il, &op, &ol);
        size_t k = sizeof(out) - ol;
        if (rc == static_cast<size_t>(-1) || il != 0 || k == 0) {
            out[0] = '?';  // same fallback as WideCharToMultiByte's default char
            k = 1;
        }
        if (gbk.size() + k > max_bytes) break;
        gbk.append(out, k);
    }
    iconv_close(cd);
#endif
    return gbk;
}

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

void ApiCore::parse_address(const std::string& addr, TransportKind& kind,
                            std::string& host, int& port, std::string& endpoint) {
    std::string s = addr;
    endpoint.clear();
    if (starts_with(s, "zmq://") || starts_with(s, "ipc://")) {
        kind = TransportKind::Zmq;
        endpoint = starts_with(s, "zmq://") ? s.substr(6) : s;
        if (endpoint.empty()) {
            host.clear();
            port = 0;
            return;
        }
        if (!starts_with(endpoint, "tcp://") && !starts_with(endpoint, "ipc://")) {
            endpoint = "tcp://" + endpoint;
        }
        host.clear();
        port = 0;
        return;
    }
    kind = TransportKind::Tcp;
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
        zmq_endpoint_.clear();
        transport_kind_ = TransportKind::Tcp;
        return;
    }
    parse_address(resolve_front_alias(front_addr_), transport_kind_, front_host_, front_port_, zmq_endpoint_);
}

void ApiCore::core_register_name_server(const char* addr) {
    std::lock_guard<std::mutex> g(mu_);
    ns_addr_ = addr ? addr : "";
    // No name-server resolution in M1: if the app only registered a name
    // server, treat it as the front (best effort).
    if (front_addr_.empty() && !ns_addr_.empty()) {
        parse_address(resolve_front_alias(ns_addr_), transport_kind_, front_host_, front_port_, zmq_endpoint_);
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
    if (reader_.joinable()) {
        // CTP forbids Release inside an SPI callback; an app that does it
        // anyway must not deadlock joining its own thread. Detach instead:
        // the reader sees stopped_ and returns without touching `this` again
        // only after this call stack unwinds, so deletion is deferred to it.
        if (reader_.get_id() == std::this_thread::get_id()) {
            reader_.detach();
            delete_on_exit_ = true;
            return;
        }
        reader_.join();
    }
    delete this;  // CTP contract: Release deletes the API object
}

const char* ApiCore::core_trading_day() {
    // trading_day_ is written by the reader thread (set_trading_day_locked)
    // on every login response; handing its address out would let the app
    // read it while the reader rewrites it. Copy under mu_ into a
    // thread_local buffer instead: the pointer stays valid for the calling
    // thread until its next GetTradingDay() call -- the same "copy it if you
    // keep it" contract the vendor API documents. Trade-off vs. a write-once
    // buffer: a re-login on a new trading day is reflected immediately.
    thread_local char buf[sizeof(trading_day_)];
    std::lock_guard<std::mutex> g(mu_);
    memcpy(buf, trading_day_, sizeof(buf));
    return buf;
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
    TransportKind kind = TransportKind::Tcp;
    std::string endpoint;
    {
        std::lock_guard<std::mutex> g(mu_);
        kind = transport_kind_;
        host = front_host_;
        port = front_port_;
        endpoint = zmq_endpoint_;
    }
    // CTPBUDDY_ADDR redirects every front to a CTPBuddy core without touching
    // the downstream's own (production) front configuration -- DESIGN §5.3.
    if (const char* over = std::getenv("CTPBUDDY_ADDR")) {
        if (*over) parse_address(over, kind, host, port, endpoint);
    }
    if (kind == TransportKind::Zmq) {
        auto& z = zmq_library();
        if (!z.load() || endpoint.empty()) return false;
        void* ctx = z.ctx_new();
        if (!ctx) return false;
        void* dealer = z.socket(ctx, ZMQ_DEALER);
        if (!dealer) {
            z.ctx_term(ctx);
            return false;
        }
        const int zero = 0;
        const int hwm = 1000;
        if (!apply_curve_options(z, dealer)) {
            z.close(dealer);
            z.ctx_term(ctx);
            return false;
        }
        z.setsockopt(dealer, ZMQ_LINGER, &zero, sizeof(zero));
        z.setsockopt(dealer, ZMQ_SNDHWM, &hwm, sizeof(hwm));
        z.setsockopt(dealer, ZMQ_RCVHWM, &hwm, sizeof(hwm));
        if (z.connect(dealer, endpoint.c_str()) != 0) {
            z.close(dealer);
            z.ctx_term(ctx);
            return false;
        }

        void* sub = nullptr;
        if (const char* pub = std::getenv("CTPBUDDY_ZMQ_PUB_ADDR")) {
            std::string pub_endpoint = pub;
            if (!pub_endpoint.empty()) {
                if (starts_with(pub_endpoint, "zmq://")) pub_endpoint = pub_endpoint.substr(6);
                if (!starts_with(pub_endpoint, "tcp://") && !starts_with(pub_endpoint, "ipc://")) {
                    pub_endpoint = "tcp://" + pub_endpoint;
                }
                sub = z.socket(ctx, ZMQ_SUB);
                if (sub) {
                    if (!apply_curve_options(z, sub)) {
                        z.close(sub);
                        z.close(dealer);
                        z.ctx_term(ctx);
                        return false;
                    }
                }
                if (sub) {
                    z.setsockopt(sub, ZMQ_LINGER, &zero, sizeof(zero));
                    z.setsockopt(sub, ZMQ_RCVHWM, &hwm, sizeof(hwm));
                    const char md[] = "md/";
                    const char td[] = "td/";
                    const char sys[] = "sys/";
                    z.setsockopt(sub, ZMQ_SUBSCRIBE, md, sizeof(md) - 1);
                    z.setsockopt(sub, ZMQ_SUBSCRIBE, td, sizeof(td) - 1);
                    z.setsockopt(sub, ZMQ_SUBSCRIBE, sys, sizeof(sys) - 1);
                    if (z.connect(sub, pub_endpoint.c_str()) != 0) {
                        z.close(sub);
                        sub = nullptr;
                    }
                }
            }
        }
        {
            std::lock_guard<std::mutex> g(mu_);
            close_socket_locked();
            zmq_context_ = ctx;
            zmq_dealer_ = dealer;
            zmq_sub_ = sub;
            transport_connected_ = true;
            transport_kind_ = TransportKind::Zmq;
        }
        return true;
    }
    if (host.empty() || port == 0) return false;
    // getaddrinfo: accepts IPv4/IPv6 literals and host names ("localhost",
    // a WSL/docker service name), not just dotted-quad literals.
    addrinfo hints{};
    hints.ai_family = AF_UNSPEC;
    hints.ai_socktype = SOCK_STREAM;
    hints.ai_protocol = IPPROTO_TCP;
    addrinfo* res = nullptr;
    std::string host_only = host;
    if (host_only.size() > 2 && host_only.front() == '[' && host_only.back() == ']') {
        host_only = host_only.substr(1, host_only.size() - 2);
    }
    if (getaddrinfo(host_only.c_str(), std::to_string(port).c_str(), &hints, &res) != 0 || !res) {
        return false;
    }
    SOCKET s = INVALID_SOCKET;
    for (addrinfo* ai = res; ai; ai = ai->ai_next) {
        SOCKET c = socket(ai->ai_family, ai->ai_socktype, ai->ai_protocol);
        if (c == INVALID_SOCKET) continue;
        if (connect(c, ai->ai_addr, static_cast<int>(ai->ai_addrlen)) == 0) {
            s = c;
            break;
        }
        closesocket(c);
    }
    freeaddrinfo(res);
    if (s == INVALID_SOCKET) return false;
    const int one = 1;  // small request frames: never wait on Nagle
    setsockopt(s, IPPROTO_TCP, TCP_NODELAY, reinterpret_cast<const char*>(&one), sizeof(one));
    {
        std::lock_guard<std::mutex> g(mu_);
        if (sock_ != INVALID_SOCKET) closesocket(sock_);
        transport_kind_ = TransportKind::Tcp;
        transport_connected_ = false;
        sock_ = s;
    }
    return true;
}

bool ApiCore::connected_locked() const {
    return transport_kind_ == TransportKind::Zmq ? transport_connected_ && zmq_dealer_ != nullptr
                                                  : sock_ != INVALID_SOCKET;
}

void ApiCore::flush_zmq_outbound() {
    auto& z = zmq_library();
    std::deque<std::vector<uint8_t>> pending;
    void* dealer = nullptr;
    {
        std::lock_guard<std::mutex> g(mu_);
        if (transport_kind_ != TransportKind::Zmq || !transport_connected_ || !zmq_dealer_) return;
        pending.swap(zmq_outbound_);
        dealer = zmq_dealer_;
    }
    while (!pending.empty()) {
        std::vector<uint8_t> bytes = std::move(pending.front());
        pending.pop_front();
        int rc = z.send(dealer, bytes.data(), bytes.size(), ZMQ_DONTWAIT);
        if (rc < 0) {
            // EAGAIN means the peer's HWM is full. Preserve ordering and try
            // again on the next poll tick; a hard error ends the connection.
            std::lock_guard<std::mutex> g(mu_);
            if (transport_connected_) {
                while (!pending.empty()) {
                    zmq_outbound_.push_front(std::move(pending.back()));
                    pending.pop_back();
                }
                zmq_outbound_.push_front(std::move(bytes));
            }
            return;
        }
    }
}

namespace {

bool decode_frame_bytes(const uint8_t* bytes, size_t len, Frame& out) {
    if (len < WIRE_HEADER_LEN || bytes[0] != 0x43 || bytes[1] != 0x42 || bytes[2] != WIRE_VERSION) return false;
    uint32_t payload_len = static_cast<uint32_t>(bytes[9]) | (static_cast<uint32_t>(bytes[10]) << 8) |
                           (static_cast<uint32_t>(bytes[11]) << 16) | (static_cast<uint32_t>(bytes[12]) << 24);
    if (payload_len > WIRE_MAX_PAYLOAD || payload_len + WIRE_HEADER_LEN != len) return false;
    out.msg_type = static_cast<uint16_t>(bytes[3] | (bytes[4] << 8));
    out.req_id = static_cast<uint32_t>(bytes[5]) | (static_cast<uint32_t>(bytes[6]) << 8) |
                 (static_cast<uint32_t>(bytes[7]) << 16) | (static_cast<uint32_t>(bytes[8]) << 24);
    out.payload.assign(bytes + WIRE_HEADER_LEN, bytes + len);
    return true;
}

bool zmq_more(ZmqLibrary& z, void* socket) {
    int more = 0;
    size_t size = sizeof(more);
    return z.getsockopt(socket, ZMQ_RCVMORE, &more, &size) == 0 && more != 0;
}

bool zmq_recv_part(ZmqLibrary& z, void* socket, std::vector<uint8_t>& out) {
    out.resize(WIRE_MAX_PAYLOAD + WIRE_HEADER_LEN + 256);
    int rc = z.recv(socket, out.data(), out.size(), 0);
    if (rc < 0) return false;
    out.resize(static_cast<size_t>(rc));
    return true;
}

}  // namespace

bool ApiCore::read_zmq_frame(Frame& out) {
    auto& z = zmq_library();
    while (!stopped_.load()) {
        flush_zmq_outbound();
        drain_auth_errors();
        drain_deferred();
        void* dealer = nullptr;
        void* sub = nullptr;
        {
            std::lock_guard<std::mutex> g(mu_);
            if (!transport_connected_ || !zmq_dealer_) return false;
            dealer = zmq_dealer_;
            sub = zmq_sub_;
        }
        ZmqPollItem items[2]{};
        items[0].socket = dealer;
        items[0].events = ZMQ_POLLIN;
        int count = 1;
        if (sub) {
            items[1].socket = sub;
            items[1].events = ZMQ_POLLIN;
            count = 2;
        }
        int ready = z.poll(items, count, 100);
        if (ready < 0) return false;
        if (ready == 0) continue;
        void* selected = nullptr;
        bool is_sub = false;
        if ((items[0].revents & ZMQ_POLLIN) != 0) selected = dealer;
        else if (sub && (items[1].revents & ZMQ_POLLIN) != 0) {
            selected = sub;
            is_sub = true;
        }
        if (!selected) continue;
        std::vector<uint8_t> first;
        if (!zmq_recv_part(z, selected, first)) return false;
        if (is_sub) {
            if (!zmq_more(z, selected)) continue;
            std::vector<uint8_t> frame;
            if (!zmq_recv_part(z, selected, frame)) return false;
            while (zmq_more(z, selected)) {
                std::vector<uint8_t> ignored;
                if (!zmq_recv_part(z, selected, ignored)) return false;
            }
            if (decode_frame_bytes(frame.data(), frame.size(), out)) {
                // The core already delivers subscribed MD and private CTP
                // callbacks through this session's DEALER. PUB is a shared
                // observer stream; replaying it into SPI duplicates callbacks
                // and bypasses this session's subscription/account scope.
                if (out.msg_type == msgs::RTN_DEPTH_MD || out.msg_type == msgs::RTN_ORDER ||
                    out.msg_type == msgs::RTN_TRADE) continue;
                return true;
            }
        } else {
            while (zmq_more(z, selected)) {
                std::vector<uint8_t> ignored;
                if (!zmq_recv_part(z, selected, ignored)) return false;
            }
            if (decode_frame_bytes(first.data(), first.size(), out)) return true;
        }
    }
    return false;
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
        drain_auth_errors();
        int ready = wait_readable(s, 100);
        if (ready == 0) continue;
        if (ready < 0) return false;
        int r = static_cast<int>(recv(s, reinterpret_cast<char*>(buf + got), static_cast<int>(n - got), 0));
        if (r > 0) {
            got += static_cast<size_t>(r);
            continue;
        }
        if (r == 0) return false;  // orderly shutdown
        if (error_is_interrupt(last_socket_error())) continue;
        return false;
    }
    return true;
}

void ApiCore::close_socket_locked() {
    // caller holds mu_ OR we are single-threaded (dtor path)
    if (transport_kind_ == TransportKind::Zmq) {
        auto& z = zmq_library();
        // libzmq sockets are thread-affine. Release() may call this method on
        // the app thread; the reader observes stopped_ and performs the close
        // on its own thread before it joins.
        if (reader_.joinable() && reader_.get_id() != std::this_thread::get_id()) {
            transport_connected_ = false;
            return;
        }
        if (zmq_sub_) z.close(zmq_sub_);
        if (zmq_dealer_) z.close(zmq_dealer_);
        if (zmq_context_) z.ctx_term(zmq_context_);
        zmq_sub_ = nullptr;
        zmq_dealer_ = nullptr;
        zmq_context_ = nullptr;
        transport_connected_ = false;
        zmq_outbound_.clear();
        return;
    }
    if (sock_ != INVALID_SOCKET) {
        closesocket(sock_);
        sock_ = INVALID_SOCKET;
    }
}

void ApiCore::write_frame_locked(const Frame& f) {
    // caller holds mu_; the ZMQ socket is owned by the reader thread, so app
    // threads enqueue bytes and the reader flushes them before polling.
    if (transport_kind_ == TransportKind::Zmq) {
        if (!transport_connected_ || !zmq_dealer_) return;
        if (zmq_outbound_.size() >= 1024) zmq_outbound_.pop_front();
        zmq_outbound_.push_back(f.encode());
        return;
    }
    // caller holds mu_; sock_ may be invalid while connecting -- drop.
    if (sock_ == INVALID_SOCKET) return;
    std::vector<uint8_t> bytes = f.encode();
    size_t sent = 0;
    while (sent < bytes.size()) {
#ifdef _WIN32
        const int flags = 0;
#else
        // a peer that vanished must yield EPIPE, never kill the host process
        // with SIGPIPE
        const int flags = MSG_NOSIGNAL;
#endif
        int r = static_cast<int>(send(sock_, reinterpret_cast<const char*>(bytes.data() + sent),
                                      static_cast<int>(bytes.size() - sent), flags));
        if (r > 0) {
            sent += static_cast<size_t>(r);
            continue;
        }
        if (error_is_interrupt(last_socket_error())) continue;
        break;  // reader thread will notice the dead socket
    }
}

namespace {

std::string json_string(const char* value, size_t size) {
    std::string out;
    out.push_back('"');
    for (size_t i = 0; i < size && value[i] != 0; ++i) {
        const unsigned char ch = static_cast<unsigned char>(value[i]);
        if (ch == '\\' || ch == '"') {
            out.push_back('\\');
            out.push_back(static_cast<char>(ch));
        } else if (ch < 0x20) {
            const char hex[] = "0123456789abcdef";
            out += "\\u00";
            out.push_back(hex[ch >> 4]);
            out.push_back(hex[ch & 15]);
        } else {
            out.push_back(static_cast<char>(ch));
        }
    }
    out.push_back('"');
    return out;
}

}  // namespace

std::string ApiCore::private_resume_json() const {
    if (private_resume_ < 0 || private_resume_ > 2) return std::string();
    return ",\"private_resume\":" + std::to_string(private_resume_);
}

void ApiCore::send_auth_locked(const char* broker, const char* user) {
    // caller holds mu_; this is the legacy automatic handshake.
    auth_in_flight_ = true;
    bound_auth_broker_ = cstr_of(broker, 11);
    bound_auth_user_ = cstr_of(user, 16);
    Frame f;
    f.msg_type = msgs::AUTH;
    f.req_id = next_req_id_++;
    auth_wire_req_id_ = f.req_id;
    std::string body = "{\"broker_id\":" + json_string(broker, 11) +
                       ",\"user_id\":" + json_string(user, 16) +
                       ",\"app_id\":\"ctpbuddy-shim\"" + private_resume_json() + "}";
    f.payload.assign(body.begin(), body.end());
    write_frame_locked(f);
}

int ApiCore::send_ctp_auth(const CThostFtdcReqAuthenticateField* req, int n_request_id) {
    CThostFtdcRspInfoField local_rsp{};
    bool local_error = false;
    {
        std::lock_guard<std::mutex> g(mu_);
        if (stopped_.load() || !connected_locked()) return -1;
        if (auth_in_flight_) return -2;
        auto fail = [&](int id, const char* msg) {
            local_rsp.ErrorID = id;
            set_text(local_rsp.ErrorMsg, sizeof(local_rsp.ErrorMsg), msg);
            local_error = true;
        };
        if (!req) {
            fail(15, "ReqAuthenticate 字段为空");
        } else {
            const std::string broker = cstr_of(req->BrokerID, sizeof(req->BrokerID));
            const std::string user = cstr_of(req->UserID, sizeof(req->UserID));
            const std::string auth_code = cstr_of(req->AuthCode, sizeof(req->AuthCode));
            const std::string app_id = cstr_of(req->AppID, sizeof(req->AppID));
            if (broker.empty() || user.empty() || auth_code.empty() || app_id.empty()) {
                fail(15, "ReqAuthenticate 必填字段为空");
            } else if (authed_) {
                // 重复认证使用通用失败兼容策略，不声称复刻柜台精确重复码。
                fail(63, "CTP:客户端认证失败");
            } else {
                Frame f;
                f.msg_type = msgs::AUTH;
                f.req_id = next_req_id_++;
                std::string body = "{\"broker_id\":" + json_string(req->BrokerID, sizeof(req->BrokerID)) +
                                   ",\"user_id\":" + json_string(req->UserID, sizeof(req->UserID)) +
                                   ",\"auth_code\":" + json_string(req->AuthCode, sizeof(req->AuthCode)) +
                                   ",\"app_id\":" + json_string(req->AppID, sizeof(req->AppID)) +
                                   private_resume_json() + "}";
                f.payload.assign(body.begin(), body.end());
                Pending pd;
                pd.req_id = f.req_id;
                pd.req_msg = msgs::AUTH;
                pd.n_request_id = n_request_id;
                set_cstr(pd.rsp_authenticate.BrokerID, sizeof(pd.rsp_authenticate.BrokerID), broker);
                set_cstr(pd.rsp_authenticate.UserID, sizeof(pd.rsp_authenticate.UserID), user);
                set_cstr(pd.rsp_authenticate.AppID, sizeof(pd.rsp_authenticate.AppID), app_id);
                set_cstr(pd.rsp_authenticate.UserProductInfo, sizeof(pd.rsp_authenticate.UserProductInfo),
                         cstr_of(req->UserProductInfo, sizeof(req->UserProductInfo)));
                // AppType 未建模：没有配置，不编造官方枚举值，保留零初始化。
                pending_[f.req_id] = pd;
                auth_in_flight_ = true;
                auth_wire_req_id_ = f.req_id;
                bound_auth_broker_ = broker;
                bound_auth_user_ = user;
                write_frame_locked(f);
            }
        }
    }
    if (local_error) {
        std::lock_guard<std::mutex> g(mu_);
        auth_errors_.emplace_back(n_request_id, local_rsp);
    }
    return 0;
}

void ApiCore::drain_auth_errors() {
    // reader thread only: flush every locally-produced callback in order
    std::vector<std::pair<int, CThostFtdcRspInfoField>> errors;
    std::vector<std::pair<int, CThostFtdcRspInfoField>> login_errors;
    {
        std::lock_guard<std::mutex> g(mu_);
        errors.swap(auth_errors_);
        login_errors.swap(login_errors_);
    }
    for (const auto& entry : errors) on_authenticate_rsp(nullptr, entry.second, entry.first);
    for (const auto& entry : login_errors) on_auth_failed(entry.first, entry.second);
    drain_deferred();
}

void ApiCore::drain_deferred() {
    std::vector<std::function<void()>> fns;
    {
        std::lock_guard<std::mutex> g(mu_);
        fns.swap(deferred_);
    }
    for (auto& fn : fns) fn();
}

int ApiCore::post_callback(std::function<void()> fn) {
    std::lock_guard<std::mutex> g(mu_);
    if (stopped_.load() || !connected_locked()) return -1;
    deferred_.push_back(std::move(fn));
    return 0;
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

        bool alive = true;
        while (!stopped_.load()) {
            Frame f;
            bool is_zmq = false;
            {
                std::lock_guard<std::mutex> g(mu_);
                is_zmq = transport_kind_ == TransportKind::Zmq;
            }
            if (is_zmq) {
                if (!read_zmq_frame(f)) {
                    alive = false;
                    break;
                }
            } else {
                uint8_t header[WIRE_HEADER_LEN];
                if (!read_exact(header, WIRE_HEADER_LEN) || header[0] != 0x43 || header[1] != 0x42 ||
                    header[2] != WIRE_VERSION) {
                    alive = false;
                    break;
                }
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
            }
            on_frame(f);
        }
        Pending auth_pd;
        Pending login_pd;
        {
            std::lock_guard<std::mutex> g(mu_);
            close_socket_locked();
            authed_ = false;
            auth_pd = take_pending(auth_wire_req_id_);
            auth_wire_req_id_ = 0;
            auth_in_flight_ = false;
            if (has_stashed_login_) login_pd = take_pending(stashed_login_.req_id);
            has_stashed_login_ = false;
            // an in-flight query can no longer complete on this socket
            qry_in_flight_ = false;
        }
        if (!stopped_.load()) {
            drain_auth_errors();
            CThostFtdcRspInfoField rsp{};
            // 断线期间的认证失败采用兼容策略，网络原因另由断线回调报告。
            rsp.ErrorID = 63;
            set_text(rsp.ErrorMsg, sizeof(rsp.ErrorMsg), "CTP:客户端认证失败");
            if (auth_pd.req_msg == msgs::AUTH) on_authenticate_rsp(nullptr, rsp, auth_pd.n_request_id);
            if (login_pd.req_msg != 0) on_auth_failed(login_pd.n_request_id, rsp);
            fire_front_disconnected(0x1001);
        }
        sleep_chunks(1.0);  // reconnect backoff
    }
    // Release() was called from an SPI callback on this very thread: the
    // object could not be deleted then, so the reader does it on its way out.
    if (delete_on_exit_) delete this;
}

void ApiCore::sleep_chunks(double seconds) {
    const auto step = std::chrono::milliseconds(100);
    auto deadline = std::chrono::steady_clock::now() + std::chrono::duration_cast<std::chrono::milliseconds>(
                                                               std::chrono::duration<double>(seconds));
    while (!stopped_.load()) {
        // callbacks posted while the socket was still up must not wait for
        // the reconnect to finish
        drain_auth_errors();
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
        case msgs::REQ_QRY_SETTLEMENT_INFO:
        case msgs::REQ_QRY_TRADING_ACCOUNT:
        case msgs::REQ_QRY_INVESTOR_POSITION:
        case msgs::REQ_QRY_INVESTOR_POSITION_DETAIL:
        case msgs::REQ_QRY_ORDER:
        case msgs::REQ_QRY_TRADE:
        case msgs::REQ_QRY_INSTRUMENT_MARGIN_RATE:
        case msgs::REQ_QRY_INSTRUMENT_COMMISSION_RATE:
        case msgs::REQ_QRY_INSTRUMENT_ORDER_COMM_RATE:
        case msgs::REQ_QRY_BROKER_TRADING_PARAMS:
        case msgs::REQ_QRY_INVESTOR_PRODUCT_GROUP_MARGIN:
            return true;
        default:
            return false;
    }
}

void ApiCore::on_auth_rsp(const Frame& f) {
    std::string body(reinterpret_cast<const char*>(f.payload.data()), f.payload.size());
    bool ok = body.find("\"ok\":true") != std::string::npos ||
              body.find("\"ok\": true") != std::string::npos;

    Frame stashed;
    int nrid = -1;
    bool have_stash = false;
    Pending auth_pd;
    {
        std::lock_guard<std::mutex> g(mu_);
        if (!auth_in_flight_ || f.req_id != auth_wire_req_id_) return;
        auth_pd = take_pending(f.req_id);
        auth_wire_req_id_ = 0;
        auth_in_flight_ = false;
        // a genuine AUTH rejection leaves authed_ false; a late "already
        // authenticated" reply to a duplicate AUTH must not downgrade us.
        if (ok) {
            authed_ = true;
            if (auth_pd.req_msg == msgs::AUTH) {
                auth_broker_ = cstr_of(auth_pd.rsp_authenticate.BrokerID,
                                       sizeof(auth_pd.rsp_authenticate.BrokerID));
                auth_user_ = cstr_of(auth_pd.rsp_authenticate.UserID,
                                     sizeof(auth_pd.rsp_authenticate.UserID));
            }
        }
        if (has_stashed_login_) {
            stashed = stashed_login_;
            has_stashed_login_ = false;
            have_stash = true;
            if (!ok) nrid = take_pending(stashed.req_id).n_request_id;
        }
    }
    CThostFtdcRspInfoField rsp{};
    rsp.ErrorID = 0;
    if (ok) {
        if (auth_pd.req_msg == msgs::AUTH) {
            on_authenticate_rsp(&auth_pd.rsp_authenticate, rsp, auth_pd.n_request_id);
        }
        if (have_stash) {
            std::lock_guard<std::mutex> g(mu_);
            write_frame_locked(stashed);
        }
        return;
    }

    rsp.ErrorID = 63;  // 兼容策略：CTP:客户端认证失败
    set_text(rsp.ErrorMsg, sizeof(rsp.ErrorMsg), "CTP:客户端认证失败");
    if (auth_pd.req_msg == msgs::AUTH) {
        on_authenticate_rsp(nullptr, rsp, auth_pd.n_request_id);
    }
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

    {
        std::lock_guard<std::mutex> g(mu_);
        // CTP: -1 = 网络连接失败. Checked before any state change so a request
        // issued before OnFrontConnected neither arms the one-in-flight query
        // gate (which would then answer -2 forever: the frame was never sent,
        // so nothing can ever complete it) nor leaks a pending entry.
        if (stopped_.load() || !connected_locked()) return -1;
        if (msg == msgs::REQ_USER_LOGIN && (authed_ || auth_in_flight_) &&
            len == sizeof(CThostFtdcReqUserLoginField)) {
            const auto* req = static_cast<const CThostFtdcReqUserLoginField*>(payload);
            const std::string broker = cstr_of(req->BrokerID, sizeof(req->BrokerID));
            const std::string user = cstr_of(req->UserID, sizeof(req->UserID));
            if (broker != bound_auth_broker_ || user != bound_auth_user_) {
                CThostFtdcRspInfoField rsp{};
                rsp.ErrorID = 15;
                set_text(rsp.ErrorMsg, sizeof(rsp.ErrorMsg),
                         "ReqUserLogin BrokerID/UserID 与认证身份不一致");
                login_errors_.emplace_back(n_request_id, rsp);
                return 0;
            }
        }
        {
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
                    // an earlier login is still waiting for AUTH: retire it now.
                    // Answered through the reader thread like every other
                    // callback -- never on the app's own stack.
                    const int retired_nrid = take_pending(stashed_login_.req_id).n_request_id;
                    CThostFtdcRspInfoField retired_rsp{};
                    retired_rsp.ErrorID = -3;
                    set_text(retired_rsp.ErrorMsg, sizeof(retired_rsp.ErrorMsg), "重复的登录请求");
                    login_errors_.emplace_back(retired_nrid, retired_rsp);
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
    }
    return 0;
}

int ApiCore::unsupported(int n_request_id, const char* method) {
    CThostFtdcRspInfoField rsp{};
    rsp.ErrorID = -1;
    set_text(rsp.ErrorMsg, sizeof(rsp.ErrorMsg), std::string("CTPBuddy 尚未实现 ") + method);
    // the vendor never calls back on the app's stack: hand it to the reader
    return post_callback([this, rsp, n_request_id] { on_rsp_error_fallback(rsp, n_request_id); });
}

// ---- pending helpers (caller holds mu_) --------------------------------------

Pending ApiCore::take_pending(uint32_t req_id) {
    // caller holds mu_: send_request() inserts from the app thread while the
    // reader completes entries here -- same unordered_map, one lock.
    auto it = pending_.find(req_id);
    if (it == pending_.end()) return Pending{};
    Pending pd = it->second;
    pending_.erase(it);
    return pd;
}

Pending* ApiCore::find_pending(uint32_t req_id) {
    // caller holds mu_; the pointer is valid only while the lock is held
    auto it = pending_.find(req_id);
    return it == pending_.end() ? nullptr : &it->second;
}

void ApiCore::erase_pending(uint32_t req_id) { pending_.erase(req_id); }  // caller holds mu_

void ApiCore::set_trading_day_locked(const char* day) {
    set_cstr(trading_day_, sizeof(trading_day_), cstr_of(day, 9));
}

}  // namespace ctpbuddy
