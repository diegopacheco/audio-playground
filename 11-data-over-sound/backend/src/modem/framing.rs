use crate::crc::crc16;
use crate::error::AppError;

pub const SYNC: [u8; 2] = [0x2D, 0xD4];
pub const MAX_PAYLOAD: usize = 512;
pub const UART_BITS: usize = 10;
const TRAILING_IDLE_BITS: usize = 4;

#[derive(Debug, Clone, PartialEq)]
pub struct DecodedByte {
    pub value: u8,
    pub bits: [u8; UART_BITS],
    pub framing_ok: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    pub payload: Vec<u8>,
    pub crc_ok: bool,
}

#[derive(Debug, PartialEq)]
pub enum Status {
    NeedMore,
    NoSync,
    TooLong(usize),
    Complete(Frame),
}

#[derive(Debug)]
pub struct Parsed {
    pub bytes: Vec<DecodedByte>,
    pub status: Status,
    pub resume: usize,
}

pub fn uart_bits(byte: u8) -> [u8; UART_BITS] {
    let mut bits = [0u8; UART_BITS];
    for (i, bit) in bits.iter_mut().enumerate().skip(1).take(8) {
        *bit = (byte >> (i - 1)) & 1;
    }
    bits[UART_BITS - 1] = 1;
    bits
}

pub fn frame_body(payload: &[u8]) -> Result<Vec<u8>, AppError> {
    if payload.is_empty() || payload.len() > MAX_PAYLOAD {
        return Err(AppError::InvalidInput(format!(
            "payload must be 1..={MAX_PAYLOAD} bytes, got {}",
            payload.len()
        )));
    }
    let len = (payload.len() as u16).to_be_bytes();
    let mut checked = len.to_vec();
    checked.extend_from_slice(payload);
    let crc = crc16(&checked).to_be_bytes();
    let mut body = SYNC.to_vec();
    body.extend_from_slice(&checked);
    body.extend_from_slice(&crc);
    Ok(body)
}

pub fn bits_for_bytes(preamble_bits: usize, bytes: &[u8]) -> Vec<u8> {
    let mut bits = vec![1u8; preamble_bits];
    for &byte in bytes {
        bits.extend_from_slice(&uart_bits(byte));
    }
    bits.extend(std::iter::repeat_n(1u8, TRAILING_IDLE_BITS));
    bits
}

fn sync_pattern() -> Vec<u8> {
    SYNC.iter().flat_map(|b| uart_bits(*b)).collect()
}

fn read_byte(chunk: &[u8; UART_BITS]) -> DecodedByte {
    let value = (0..8).fold(0u8, |acc, i| acc | ((chunk[i + 1] & 1) << i));
    DecodedByte {
        value,
        bits: *chunk,
        framing_ok: chunk[0] == 0 && chunk[UART_BITS - 1] == 1,
    }
}

pub fn parse(bits: &[u8], search_limit: usize) -> Parsed {
    let pattern = sync_pattern();
    let mut start = None;
    for s in 0..=search_limit {
        if s + pattern.len() > bits.len() {
            return Parsed {
                bytes: Vec::new(),
                status: Status::NeedMore,
                resume: 0,
            };
        }
        if bits[s..s + pattern.len()] == pattern[..] {
            start = Some(s);
            break;
        }
    }
    let Some(start) = start else {
        return Parsed {
            bytes: Vec::new(),
            status: Status::NoSync,
            resume: search_limit,
        };
    };
    let bytes: Vec<DecodedByte> = bits[start..]
        .as_chunks::<UART_BITS>()
        .0
        .iter()
        .map(read_byte)
        .collect();
    let status = frame_status(&bytes);
    let keep = match &status {
        Status::Complete(frame) => SYNC.len() + 4 + frame.payload.len(),
        Status::TooLong(_) => SYNC.len() + 2,
        _ => bytes.len(),
    };
    Parsed {
        bytes: bytes.into_iter().take(keep).collect(),
        status,
        resume: start + keep * UART_BITS,
    }
}

fn frame_status(bytes: &[DecodedByte]) -> Status {
    let header = SYNC.len() + 2;
    if bytes.len() < header {
        return Status::NeedMore;
    }
    let len = usize::from(u16::from_be_bytes([bytes[2].value, bytes[3].value]));
    if len == 0 || len > MAX_PAYLOAD {
        return Status::TooLong(len);
    }
    let total = header + len + 2;
    if bytes.len() < total {
        return Status::NeedMore;
    }
    let checked: Vec<u8> = bytes[SYNC.len()..header + len]
        .iter()
        .map(|b| b.value)
        .collect();
    let received = u16::from_be_bytes([bytes[total - 2].value, bytes[total - 1].value]);
    Status::Complete(Frame {
        payload: checked[2..].to_vec(),
        crc_ok: crc16(&checked) == received,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uart_byte_has_start_bit_zero_lsb_first_data_and_stop_bit_one() {
        assert_eq!(uart_bits(0b1000_0001), [0, 1, 0, 0, 0, 0, 0, 0, 1, 1]);
    }

    #[test]
    fn framed_bits_parse_back_to_the_same_payload_with_valid_crc() {
        let body = frame_body(b"hi!").unwrap();
        let bits = bits_for_bytes(16, &body);
        let parsed = parse(&bits, 32);
        assert_eq!(
            parsed.status,
            Status::Complete(Frame {
                payload: b"hi!".to_vec(),
                crc_ok: true
            })
        );
        assert_eq!(parsed.bytes.len(), body.len());
        assert!(parsed.bytes.iter().all(|b| b.framing_ok));
    }

    #[test]
    fn flipped_payload_bit_is_reported_as_crc_failure() {
        let body = frame_body(b"secret").unwrap();
        let mut bits = bits_for_bytes(16, &body);
        let first_payload_data_bit = 16 + 4 * UART_BITS + 1;
        bits[first_payload_data_bit] ^= 1;
        match parse(&bits, 32).status {
            Status::Complete(frame) => {
                assert!(!frame.crc_ok);
                assert_ne!(frame.payload, b"secret".to_vec());
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn partial_frame_asks_for_more_bits_instead_of_failing() {
        let body = frame_body(b"partial").unwrap();
        let bits = bits_for_bytes(16, &body);
        assert_eq!(parse(&bits[..bits.len() / 2], 32).status, Status::NeedMore);
    }

    #[test]
    fn missing_sync_word_is_reported_once_search_window_is_exhausted() {
        let bits = vec![1u8; 200];
        assert_eq!(parse(&bits, 32).status, Status::NoSync);
    }

    #[test]
    fn rejects_empty_and_oversized_payloads() {
        assert!(frame_body(b"").is_err());
        assert!(frame_body(&vec![b'a'; MAX_PAYLOAD + 1]).is_err());
    }
}
