//! Dependency-free, deterministic fuzz tests for the wire decoders.
//! Set `CTPBUDDY_FUZZ_ITERS` to run longer (default 20000 per case).

use ctpbuddy_wire::frame::{Frame, HEADER_LEN, MAX_PAYLOAD};
use ctpbuddy_wire::gbk;
use std::io::Cursor;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        // xorshift64*
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545F4914F6CDD1D)
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
    fn bytes(&mut self, n: usize) -> Vec<u8> {
        (0..n).map(|_| self.next() as u8).collect()
    }
}

fn iters() -> usize {
    std::env::var("CTPBUDDY_FUZZ_ITERS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(20_000)
}

#[test]
fn random_bytes_never_panic_frame_decoder() {
    let mut rng = Rng(0x9E3779B97F4A7C15);
    for _ in 0..iters() {
        let n = rng.below(64);
        let data = rng.bytes(n);
        let mut c = Cursor::new(data);
        // must terminate with Ok/Err, never panic or over-allocate
        while let Ok(Some(_)) = Frame::read_from(&mut c) {}
    }
}

#[test]
fn mutated_valid_frames_never_panic_and_roundtrip_unmutated() {
    let mut rng = Rng(42);
    for _ in 0..iters() {
        let plen = rng.below(200);
        let f = Frame::new(rng.next() as u16, rng.next() as u32, rng.bytes(plen));
        let enc = f.encode();
        let back = Frame::read_from(&mut Cursor::new(enc.clone()))
            .unwrap()
            .unwrap();
        assert_eq!(
            (back.msg_type, back.req_id, &back.payload),
            (f.msg_type, f.req_id, &f.payload)
        );
        // flip 1..4 random bytes, optionally truncate
        let mut m = enc;
        for _ in 0..=rng.below(4) {
            let i = rng.below(m.len());
            m[i] ^= 1 << rng.below(8);
        }
        if rng.below(3) == 0 {
            let cut = rng.below(m.len() + 1);
            m.truncate(cut);
        }
        let mut c = Cursor::new(m);
        while let Ok(Some(_)) = Frame::read_from(&mut c) {}
    }
}

#[test]
fn oversize_length_is_rejected_before_allocation() {
    let mut h = vec![0x43, 0x42, 1, 0, 0, 0, 0, 0, 0];
    h.extend_from_slice(&((MAX_PAYLOAD as u32) + 1).to_le_bytes());
    assert_eq!(h.len(), HEADER_LEN);
    let e = Frame::read_from(&mut Cursor::new(h)).unwrap_err();
    assert_eq!(e.kind(), std::io::ErrorKind::InvalidData);
    // u32::MAX length too
    let mut h = vec![0x43, 0x42, 1, 0, 0, 0, 0, 0, 0];
    h.extend_from_slice(&u32::MAX.to_le_bytes());
    assert!(Frame::read_from(&mut Cursor::new(h)).is_err());
}

#[test]
fn gbk_decoder_total_and_field_writer_bounded() {
    let mut rng = Rng(7);
    for _ in 0..iters() {
        let n = rng.below(40);
        let data = rng.bytes(n);
        let _ = gbk::decode(&data);
        let _ = gbk::read_field(&data);
        let s: String = (0..rng.below(20))
            .map(|_| char::from_u32((rng.next() % 0x3000) as u32 + 0x20).unwrap_or('?'))
            .collect();
        let mut buf = vec![0u8; 1 + rng.below(16)];
        gbk::write_field(&mut buf, &s);
        // round-trip through the field never panics and fits
        let _ = gbk::read_field(&buf);
        let _ = gbk::encode(&s);
    }
}
