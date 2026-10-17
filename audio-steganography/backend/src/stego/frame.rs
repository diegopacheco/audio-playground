use crate::crc32::crc32;
use crate::error::{Result, StegoError};

pub const MAGIC: [u8; 4] = *b"STG1";
pub const OVERHEAD: usize = 12;

pub fn build(payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(payload.len() + OVERHEAD);
    out.extend_from_slice(&MAGIC);
    out.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    out.extend_from_slice(payload);
    out.extend_from_slice(&crc32(payload).to_be_bytes());
    out
}

pub fn parse(bytes: &[u8]) -> Result<Vec<u8>> {
    if bytes.len() < 8 || bytes[0..4] != MAGIC {
        return Err(StegoError::NoPayload);
    }
    let declared = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as usize;
    let available = bytes.len().saturating_sub(OVERHEAD);
    if declared > available {
        return Err(StegoError::InvalidLength {
            declared,
            available,
        });
    }
    let payload = &bytes[8..8 + declared];
    let tail = &bytes[8 + declared..8 + declared + 4];
    let stored = u32::from_be_bytes([tail[0], tail[1], tail[2], tail[3]]);
    let computed = crc32(payload);
    if stored != computed {
        return Err(StegoError::CrcMismatch { stored, computed });
    }
    Ok(payload.to_vec())
}

pub fn to_bits(bytes: &[u8]) -> Vec<bool> {
    bytes
        .iter()
        .flat_map(|b| (0..8).rev().map(move |i| (b >> i) & 1 == 1))
        .collect()
}

pub fn from_bits(bits: &[bool]) -> Vec<u8> {
    bits.as_chunks::<8>()
        .0
        .iter()
        .map(|c| c.iter().fold(0u8, |acc, &b| (acc << 1) | b as u8))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_round_trips_with_trailing_garbage_because_carriers_hold_more_than_the_message() {
        let mut bytes = build(b"secret");
        bytes.extend_from_slice(&[0xAA; 20]);
        assert_eq!(parse(&bytes).unwrap(), b"secret");
    }

    #[test]
    fn missing_magic_means_no_payload_instead_of_garbage_text() {
        assert_eq!(parse(&[0u8; 32]), Err(StegoError::NoPayload));
    }

    #[test]
    fn length_larger_than_carrier_is_rejected_before_reading_out_of_bounds() {
        let mut bytes = build(b"abc");
        bytes[7] = 200;
        assert!(matches!(
            parse(&bytes),
            Err(StegoError::InvalidLength { .. })
        ));
    }

    #[test]
    fn bits_round_trip_msb_first() {
        let bits = to_bits(&[0b1000_0001]);
        assert!(bits[0]);
        assert!(bits[7]);
        assert_eq!(from_bits(&bits), vec![0b1000_0001]);
    }
}
