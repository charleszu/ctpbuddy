//! CTPBuddy wire protocol: framing, message ids, CTP struct mirrors.

pub mod frame;
pub mod generated;
pub mod msgs;

pub use frame::{Frame, HEADER_LEN, MAGIC, MAX_PAYLOAD, WIRE_VERSION};

/// Read a wire payload as a CTP struct mirror (repr(C), same layout as C).
/// Uses unaligned read: wire buffers carry no alignment guarantee.
pub fn struct_from_bytes<T: Copy>(buf: &[u8]) -> Option<T> {
    if buf.len() != std::mem::size_of::<T>() {
        return None;
    }
    // SAFETY: T is repr(C) plain data; read_unaligned avoids alignment assumptions.
    Some(unsafe { std::ptr::read_unaligned(buf.as_ptr() as *const T) })
}

/// Serialize a CTP struct mirror into wire payload bytes (same layout as C).
pub fn struct_to_bytes<T>(v: &T) -> Vec<u8> {
    let size = std::mem::size_of::<T>();
    // SAFETY: T is repr(C) plain data; byte view is valid for its full size.
    let slice = unsafe { std::slice::from_raw_parts(v as *const T as *const u8, size) };
    slice.to_vec()
}
