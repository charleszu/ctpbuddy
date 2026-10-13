//! Transport-independent recording and deterministic replay of front budgets.
//! The ordinary PULSE timer stays real; only offline budget calls use the trace clock.
use crate::{
    flow,
    json::{self, Value},
    Config, World,
};
use ctpbuddy_wire::generated::{CThostFtdcReqUserLoginField, CThostFtdcRspInfoField};
use ctpbuddy_wire::{msgs, struct_from_bytes, Frame};
use std::{
    collections::HashMap,
    sync::mpsc,
    time::{Duration, Instant},
};

pub(crate) const REPLAY_ID_BASE: u64 = 1 << 32;

#[derive(Default)]
pub(crate) struct ReplayState {
    connect: HashMap<String, flow::WindowBudget>,
    last_clock: HashMap<String, u64>,
}

fn enabled(cfg: &Config) -> bool {
    cfg.flow_replay
        || cfg.ftd_max_comm_flux != 0
        || cfg.connect_freq != 0
        || cfg.exchange_freq != 0
        || cfg.exchange_limits.values().any(|v| *v != 0)
}

pub(crate) fn config(cfg: &Config) -> Value {
    json::obj_sorted(vec![
        ("enabled".into(), json::b(enabled(cfg))),
        ("broker_ids".into(), json::s(&cfg.broker_id)),
        (
            "ftd_max_comm_flux".into(),
            json::n(cfg.ftd_max_comm_flux as f64),
        ),
        ("connect_freq".into(), json::n(cfg.connect_freq as f64)),
        ("exchange_freq".into(), json::n(cfg.exchange_freq as f64)),
        (
            "exchange_limits".into(),
            json::obj_sorted(
                cfg.exchange_limits
                    .iter()
                    .map(|(k, v)| (k.clone(), json::n(*v as f64)))
                    .collect(),
            ),
        ),
    ])
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(DIGITS[(b >> 4) as usize] as char);
        out.push(DIGITS[(b & 15) as usize] as char);
    }
    out
}

fn unhex(text: &str) -> Result<Vec<u8>, String> {
    if !text.len().is_multiple_of(2) || text.len() > ctpbuddy_wire::MAX_PAYLOAD * 2 {
        return Err("invalid flow payload length".into());
    }
    text.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| {
            let digit = |b: u8| match b {
                b'0'..=b'9' => Ok(b - b'0'),
                b'a'..=b'f' => Ok(b - b'a' + 10),
                _ => Err("invalid flow payload hex".to_string()),
            };
            Ok(digit(p[0])? * 16 + digit(p[1])?)
        })
        .collect()
}

fn safe_payload(frame: &Frame, replay: bool) -> Vec<u8> {
    if frame.msg_type == msgs::AUTH {
        let Ok(Value::Obj(fields)) = json::parse(&String::from_utf8_lossy(&frame.payload)) else {
            return b"{".to_vec(); // never persist opaque credentials in malformed AUTH
        };
        let mut safe: Vec<_> = fields
            .into_iter()
            .filter(|(k, _)| {
                ["broker_id", "user_id", "app_id", "private_resume"].contains(&k.as_str())
            })
            .collect();
        let original = json::parse(&String::from_utf8_lossy(&frame.payload)).unwrap();
        let presented = original.get_str("auth_code").unwrap_or_default();
        let accepted = if replay {
            presented == "accepted"
        } else {
            crate::handlers::auth_code_ok(
                std::env::var(crate::handlers::TD_TOKEN_ENV).ok().as_deref(),
                &presented,
            )
        };
        safe.push((
            "auth_code".into(),
            json::s(if accepted { "accepted" } else { "rejected" }),
        ));
        return json::obj_sorted(safe).to_json().into_bytes();
    }
    let mut bytes = frame.payload.clone();
    if frame.msg_type == msgs::REQ_USER_LOGIN {
        for (start, size) in [
            (
                std::mem::offset_of!(CThostFtdcReqUserLoginField, Password),
                41,
            ),
            (
                std::mem::offset_of!(CThostFtdcReqUserLoginField, OneTimePassword),
                41,
            ),
        ] {
            let end = (start + size).min(bytes.len());
            if start < end {
                bytes[start..end].fill(0);
            }
        }
    }
    bytes
}

fn integer(data: &Value, key: &str, max: u64) -> Result<u64, String> {
    match data.get(key) {
        Some(Value::Num(n))
            if n.is_finite() && n.fract() == 0.0 && *n >= 0.0 && *n <= max as f64 =>
        {
            Ok(*n as u64)
        }
        _ => Err(format!("invalid flow {key}")),
    }
}

impl World {
    fn trace_id(&self, id: u64) -> u64 {
        if id >= REPLAY_ID_BASE {
            id - REPLAY_ID_BASE
        } else {
            id
        }
    }

    pub(crate) fn record_flow(&mut self, action: &str, id: u64, frame: Option<&Frame>) {
        if !enabled(&self.cfg) {
            return;
        }
        let mut data = vec![
            ("action".into(), json::s(action)),
            ("conn_id".into(), json::n(self.trace_id(id) as f64)),
            (
                "clock_ns".into(),
                json::s(
                    &self
                        .flow_clock
                        .saturating_duration_since(flow::origin())
                        .as_nanos()
                        .to_string(),
                ),
            ),
        ];
        if let Some(f) = frame {
            data.push((
                "frame".into(),
                json::obj_sorted(vec![
                    ("msg_type".into(), json::n(f.msg_type as f64)),
                    ("req_id".into(), json::n(f.req_id as f64)),
                    (
                        "payload".into(),
                        json::s(&hex(&safe_payload(f, self.cfg.flow_replay))),
                    ),
                ]),
            ));
        }
        self.journal_record_json("flow_input", "", "", json::obj_sorted(data));
    }

    pub(crate) fn record_connect(&mut self, scope: &str, now: Instant, allowed: bool) {
        if !enabled(&self.cfg) {
            return;
        }
        self.journal_record_json(
            "flow_input",
            "",
            "",
            json::obj_sorted(vec![
                ("action".into(), json::s("connect")),
                ("conn_id".into(), json::n(0.0)),
                (
                    "clock_ns".into(),
                    json::s(
                        &now.saturating_duration_since(flow::origin())
                            .as_nanos()
                            .to_string(),
                    ),
                ),
                ("scope".into(), json::s(scope)),
                ("allowed".into(), json::b(allowed)),
            ]),
        );
    }

    pub(crate) fn record_flow_output(&mut self, id: u64, frame: &Frame) {
        if !enabled(&self.cfg) || !self.conns.get(&id).is_some_and(|c| !c.is_admin) {
            return;
        }
        // Compare the SPI surface as well as business journal hashes. Credentials
        // never occur in this summary; malformed AUTH diagnostics are not copied.
        let error = if frame.msg_type == msgs::RSP_ERROR {
            struct_from_bytes::<CThostFtdcRspInfoField>(&frame.payload).map(|v| v.ErrorID)
        } else if [msgs::ERR_RTN_ORDER_INSERT, msgs::ERR_RTN_ORDER_ACTION].contains(&frame.msg_type)
        {
            frame
                .payload
                .len()
                .checked_sub(std::mem::size_of::<CThostFtdcRspInfoField>())
                .and_then(|n| struct_from_bytes::<CThostFtdcRspInfoField>(&frame.payload[n..]))
                .map(|v| v.ErrorID)
        } else {
            None
        };
        let mut data = vec![
            ("conn_id".into(), json::n(self.trace_id(id) as f64)),
            ("msg_type".into(), json::n(frame.msg_type as f64)),
            ("req_id".into(), json::n(frame.req_id as f64)),
            (
                "error_id".into(),
                error.map(|v| json::n(v as f64)).unwrap_or(Value::Null),
            ),
            (
                "payload".into(),
                if frame.msg_type == msgs::AUTH_RSP {
                    json::s("")
                } else {
                    json::s(&hex(&frame.payload))
                },
            ),
        ];
        if frame.msg_type == msgs::AUTH_RSP {
            let ok = json::parse(&String::from_utf8_lossy(&frame.payload))
                .ok()
                .and_then(|v| v.get_bool("ok"))
                .unwrap_or(false);
            data.push(("auth_ok".into(), json::b(ok)));
        }
        self.journal_record_json("flow_output", "", "", json::obj_sorted(data));
    }

    pub(crate) fn apply_flow_input(&mut self, command: &Value) -> Result<(), String> {
        if !self.cfg.flow_replay {
            return Err("flow_replay requires explicit offline --flow-replay mode".into());
        }
        if !matches!(command, Value::Obj(p) if p.len()==2) {
            return Err("flow_replay accepts cmd/data only".into());
        }
        let data = command.get("data").ok_or("missing flow data")?;
        let action = data.get_str("action").ok_or("missing flow action")?;
        let ns_text = data
            .get_str("clock_ns")
            .ok_or("flow clock_ns must be decimal text")?;
        if ns_text.is_empty() || !ns_text.bytes().all(|b| b.is_ascii_digit()) {
            return Err("invalid flow clock_ns".into());
        }
        let ns = ns_text
            .parse::<u64>()
            .map_err(|_| "flow clock_ns overflow")?;
        let original_id = integer(data, "conn_id", i32::MAX as u64)?;
        let clock_key = if action == "connect" {
            data.get_str("scope").ok_or("missing connect scope")?
        } else {
            "world".into()
        };
        if self
            .flow_state
            .last_clock
            .get(&clock_key)
            .is_some_and(|last| ns < *last)
        {
            return Err("flow clock moved backwards within a scope".into());
        }
        let now = flow::origin()
            .checked_add(Duration::from_nanos(ns))
            .ok_or("flow clock overflow")?;
        let id = REPLAY_ID_BASE + original_id;
        match action.as_str() {
            "open" => {
                if original_id == 0 || self.conns.contains_key(&id) {
                    return Err("invalid duplicate flow open".into());
                }
                let (writer, _) = mpsc::channel();
                self.on_conn_opened(id, writer, false);
                self.conns.get_mut(&id).unwrap().front_id = original_id as i32;
                self.flow_clock = now;
                self.record_flow("open", id, None);
            }
            "close" => {
                if !self.conns.contains_key(&id) {
                    return Err("flow close without open".into());
                }
                self.flow_clock = now;
                self.on_conn_closed(id);
            }
            "inbound" => {
                if !self.conns.contains_key(&id) {
                    return Err("flow inbound without open".into());
                }
                let f = data.get("frame").ok_or("missing flow frame")?;
                let msg_type = integer(f, "msg_type", u16::MAX as u64)? as u16;
                let req_id = integer(f, "req_id", u32::MAX as u64)? as u32;
                let payload = if let Some(path) = f.get_str("payload_file") {
                    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
                    use std::io::Read;
                    let mut buf = Vec::new();
                    file.take((ctpbuddy_wire::MAX_PAYLOAD + 1) as u64)
                        .read_to_end(&mut buf)
                        .map_err(|e| e.to_string())?;
                    if buf.len() > ctpbuddy_wire::MAX_PAYLOAD {
                        return Err("flow payload too large".into());
                    }
                    buf
                } else {
                    unhex(&f.get_str("payload").ok_or("missing flow payload")?)?
                };
                self.flow_clock = now;
                self.on_frame(id, Frame::new(msg_type, req_id, payload));
            }
            "drain" => {
                self.drain_ftd(now);
            }
            "connect" => {
                // "curve" stays accepted only to replay journals recorded
                // before ADR 0002 removed the CURVE transport; no code path
                // produces that scope anymore.
                if !["tcp", "zmq", "curve"].contains(&clock_key.as_str()) {
                    return Err("unknown flow connect scope".into());
                }
                let expected = data.get_bool("allowed").ok_or("missing connect outcome")?;
                let allowed = self
                    .flow_state
                    .connect
                    .entry(clock_key.clone())
                    .or_default()
                    .take(now, self.cfg.connect_freq);
                self.record_connect(&clock_key, now, allowed);
                if allowed != expected {
                    return Err("connect gate outcome differs from recording".into());
                }
            }
            _ => return Err("unknown flow action".into()),
        }
        self.flow_state.last_clock.insert(clock_key, ns);
        Ok(())
    }
}
