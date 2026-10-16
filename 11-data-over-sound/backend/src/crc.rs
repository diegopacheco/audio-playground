const POLY: u16 = 0x1021;
const INIT: u16 = 0xFFFF;

pub fn crc16(data: &[u8]) -> u16 {
    let mut crc = INIT;
    for &byte in data {
        crc ^= u16::from(byte) << 8;
        for _ in 0..8 {
            crc = if crc & 0x8000 != 0 {
                (crc << 1) ^ POLY
            } else {
                crc << 1
            };
        }
    }
    crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_ccitt_false_check_value_so_other_tools_can_verify_frames() {
        assert_eq!(crc16(b"123456789"), 0x29B1);
    }

    #[test]
    fn any_single_bit_flip_changes_the_checksum_so_corruption_is_detected() {
        let message = b"hello over sound".to_vec();
        let reference = crc16(&message);
        for index in 0..message.len() {
            for bit in 0..8 {
                let mut corrupted = message.clone();
                corrupted[index] ^= 1 << bit;
                assert_ne!(crc16(&corrupted), reference, "flip at {index}:{bit}");
            }
        }
    }

    #[test]
    fn swapped_bytes_are_detected_because_crc_is_order_sensitive() {
        assert_ne!(crc16(b"ab"), crc16(b"ba"));
    }
}
