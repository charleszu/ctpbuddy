//! Wire message ids. Hand-assigned and never reused (DESIGN.md §6.3).
//!
//!   0x00xx  protocol: PING / PONG
//!   0x01xx  session core-extension messages (JSON payloads, no CTP struct)
//!   0x02xx  admin control plane (JSON payloads)
//!   0x10xx  CTP message passthrough (raw struct bytes; see registry.json)
//!   0x20xx+ CTP struct ids (generated, shim/generated/registry.hpp)
//!
//! Response conventions (the shim maps frames 1:1 to SPI callbacks):
//! - every request frame carries `req_id`; all RSP/Rtn answer frames echo it;
//! - a successful `RSP_*` frame's payload is the response struct itself, with
//!   `CThostFtdcRspInfoField` implied to be `{ErrorID: 0}`;
//! - a failed request is answered with `RSP_ERROR` (payload =
//!   `CThostFtdcRspInfoField`): the pending request completes as
//!   `OnRsp*(NULL, pRspInfo, bIsLast=true)`; outside a pending request it
//!   surfaces as `OnRspError`;
//! - `ERR_RTN_ORDER_INSERT` (payload = `CThostFtdcInputOrderField` ++
//!   `CThostFtdcRspInfoField`) surfaces as `OnErrRtnOrderInsert` — the
//!   exchange half of a rejection, sent after the front office already
//!   answered the request, so the client's input rides along (the shim's
//!   pending entry is consumed by then);
//! - `ERR_RTN_ORDER_ACTION` (payload = `CThostFtdcInputOrderActionField` ++
//!   `CThostFtdcRspInfoField`) surfaces as `OnErrRtnOrderAction`, paired with
//!   the `RSP_ERROR` response half (官方报单回调规则 场景 6/7: 先响应后回报);
//! - a query stream is terminated by `QRY_LAST` (empty payload) which
//!   completes the pending `OnRspQry*(NULL, {0}, bIsLast=true)`;
//! - `RTN_ORDER` / `RTN_TRADE` / `RTN_DEPTH_MD` are pushes with `req_id = 0`.

pub const PING: u16 = 0x0001;
pub const PONG: u16 = 0x0002;

pub const AUTH: u16 = 0x0101;
pub const AUTH_RSP: u16 = 0x0102;
pub const LOGOUT: u16 = 0x0103;
pub const LOGOUT_RSP: u16 = 0x0104;

pub const ADMIN_REQ: u16 = 0x0201;
pub const ADMIN_RSP: u16 = 0x0202;

pub const REQ_USER_LOGIN: u16 = 0x1001;
pub const RSP_USER_LOGIN: u16 = 0x1002;
pub const REQ_USER_LOGOUT: u16 = 0x1003;
pub const RSP_USER_LOGOUT: u16 = 0x1004;
pub const REQ_SETTLE_CONFIRM: u16 = 0x1005;
pub const RSP_SETTLE_CONFIRM: u16 = 0x1006;
pub const REQ_ORDER_INSERT: u16 = 0x1010;
pub const RSP_ORDER_INSERT: u16 = 0x1011;
pub const ERR_RTN_ORDER_INSERT: u16 = 0x1012;
pub const RTN_ORDER: u16 = 0x1013;
pub const RTN_TRADE: u16 = 0x1014;
pub const REQ_ORDER_ACTION: u16 = 0x1015;
pub const RSP_ORDER_ACTION: u16 = 0x1016;
pub const ERR_RTN_ORDER_ACTION: u16 = 0x1017;
pub const SUB_MD: u16 = 0x1020;
pub const RSP_SUB_MD: u16 = 0x1021;
pub const RTN_DEPTH_MD: u16 = 0x1022;
pub const UNSUB_MD: u16 = 0x1023;
pub const RSP_UNSUB_MD: u16 = 0x1024;
pub const RSP_ERROR: u16 = 0x1030;
pub const REQ_QRY_SETTLEMENT_INFO: u16 = 0x1031;
pub const RSP_QRY_SETTLEMENT_INFO: u16 = 0x1032;

pub const REQ_QRY_INSTRUMENT: u16 = 0x1040;
pub const RSP_QRY_INSTRUMENT: u16 = 0x1041;
pub const REQ_QRY_TRADING_ACCOUNT: u16 = 0x1042;
pub const RSP_QRY_TRADING_ACCOUNT: u16 = 0x1043;
pub const REQ_QRY_INVESTOR_POSITION: u16 = 0x1044;
pub const RSP_QRY_INVESTOR_POSITION: u16 = 0x1045;
pub const REQ_QRY_ORDER: u16 = 0x1046;
pub const RSP_QRY_ORDER: u16 = 0x1047;
pub const REQ_QRY_TRADE: u16 = 0x1048;
pub const RSP_QRY_TRADE: u16 = 0x1049;
// Reference-data queries. Each one answers from the *same* tables the ledger
// computes from (notes/04 G), so a client can cross-check a query response
// against its own margin total — the numbers agree by construction.
pub const REQ_QRY_INSTRUMENT_MARGIN_RATE: u16 = 0x1051;
pub const RSP_QRY_INSTRUMENT_MARGIN_RATE: u16 = 0x1052;
pub const REQ_QRY_INSTRUMENT_COMMISSION_RATE: u16 = 0x1053;
pub const RSP_QRY_INSTRUMENT_COMMISSION_RATE: u16 = 0x1054;
pub const REQ_QRY_INSTRUMENT_ORDER_COMM_RATE: u16 = 0x1055;
pub const RSP_QRY_INSTRUMENT_ORDER_COMM_RATE: u16 = 0x1056;
pub const REQ_QRY_BROKER_TRADING_PARAMS: u16 = 0x1057;
pub const RSP_QRY_BROKER_TRADING_PARAMS: u16 = 0x1058;
pub const REQ_QRY_INVESTOR_POSITION_DETAIL: u16 = 0x1059;
pub const RSP_QRY_INVESTOR_POSITION_DETAIL: u16 = 0x105A;
/// Terminates every query stream (CTP's `bIsLast`).
pub const QRY_LAST: u16 = 0x1050;
pub const REQ_QRY_INVESTOR_PRODUCT_GROUP_MARGIN: u16 = 0x105B;
pub const RSP_QRY_INVESTOR_PRODUCT_GROUP_MARGIN: u16 = 0x105C;
