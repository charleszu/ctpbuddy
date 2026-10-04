//! CTPBuddy wire protocol: framing, message ids, CTP struct mirrors.

pub mod frame;
pub mod generated;
pub mod msgs;

pub use frame::{Frame, HEADER_LEN, MAGIC, MAX_PAYLOAD, WIRE_VERSION};

pub trait WireStruct: Sized {
    fn decode(buf: &[u8]) -> Option<Self>;
    fn encode(&self) -> Vec<u8>;
}

pub fn struct_from_bytes<T: WireStruct>(buf: &[u8]) -> Option<T> {
    T::decode(buf)
}

pub fn struct_to_bytes<T: WireStruct>(value: &T) -> Vec<u8> {
    value.encode()
}

#[cfg(test)]
mod tests {
    use super::*;
    use generated::CThostFtdcInputOrderField;

    #[test]
    fn order_numbers_use_little_endian_at_ctp_offsets() {
        let mut order = CThostFtdcInputOrderField::zeroed();
        order.LimitPrice = 3500.5;
        order.VolumeTotalOriginal = 0x01020304;
        let encoded = struct_to_bytes(&order);
        let price_offset = std::mem::offset_of!(CThostFtdcInputOrderField, LimitPrice);
        let volume_offset = std::mem::offset_of!(CThostFtdcInputOrderField, VolumeTotalOriginal);
        assert_eq!(&encoded[price_offset..price_offset + 8], &3500.5f64.to_le_bytes());
        assert_eq!(&encoded[volume_offset..volume_offset + 4], &[4, 3, 2, 1]);
        let decoded: CThostFtdcInputOrderField = struct_from_bytes(&encoded).unwrap();
        assert_eq!(decoded.LimitPrice, order.LimitPrice);
        assert_eq!(decoded.VolumeTotalOriginal, order.VolumeTotalOriginal);
    }
}
