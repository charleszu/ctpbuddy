//! Wire framing: magic 'CB' + version + msg type + req id + payload length.
//!
//! Transport-agnostic by design. The TCP transport used by ctpbuddy-server in
//! M1 carries frames back-to-back on one connection per client; the ZeroMQ
//! adapter (DESIGN.md §6) reuses this exact framing inside ZMTP messages.

use std::io::{Read, Write};

pub const MAGIC: [u8; 2] = [0x43, 0x42]; // "CB"
pub const WIRE_VERSION: u8 = 1;
pub const HEADER_LEN: usize = 13;
pub const MAX_PAYLOAD: usize = 1 << 20;

#[derive(Clone, Debug)]
pub struct Frame {
    pub msg_type: u16,
    pub req_id: u32,
    pub payload: Vec<u8>,
}

impl Frame {
    pub fn new(msg_type: u16, req_id: u32, payload: Vec<u8>) -> Self {
        Frame {
            msg_type,
            req_id,
            payload,
        }
    }

    pub fn encoded_len(&self) -> usize {
        HEADER_LEN + self.payload.len()
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut v = Vec::with_capacity(self.encoded_len());
        v.extend_from_slice(&MAGIC);
        v.push(WIRE_VERSION);
        v.extend_from_slice(&self.msg_type.to_le_bytes());
        v.extend_from_slice(&self.req_id.to_le_bytes());
        v.extend_from_slice(&(self.payload.len() as u32).to_le_bytes());
        v.extend_from_slice(&self.payload);
        v
    }

    /// Read one frame. Ok(None) = clean EOF at a frame boundary.
    pub fn read_from<R: Read>(r: &mut R) -> std::io::Result<Option<Frame>> {
        let mut header = [0u8; HEADER_LEN];
        match r.read_exact(&mut header) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(e),
        }
        if header[0..2] != MAGIC {
            return Err(err("bad magic"));
        }
        if header[2] != WIRE_VERSION {
            return Err(err("unsupported wire version"));
        }
        let msg_type = u16::from_le_bytes([header[3], header[4]]);
        let req_id = u32::from_le_bytes([header[5], header[6], header[7], header[8]]);
        let len = u32::from_le_bytes([header[9], header[10], header[11], header[12]]) as usize;
        if len > MAX_PAYLOAD {
            return Err(err("payload too large"));
        }
        let mut payload = vec![0u8; len];
        if len > 0 {
            r.read_exact(&mut payload)?;
        }
        Ok(Some(Frame {
            msg_type,
            req_id,
            payload,
        }))
    }

    pub fn write_to<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        w.write_all(&self.encode())
    }
}

fn err(msg: &str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, msg)
}
