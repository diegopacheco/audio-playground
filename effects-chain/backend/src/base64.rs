use crate::error::AppError;

fn value(c: u8) -> Result<u32, AppError> {
    match c {
        b'A'..=b'Z' => Ok((c - b'A') as u32),
        b'a'..=b'z' => Ok((c - b'a' + 26) as u32),
        b'0'..=b'9' => Ok((c - b'0' + 52) as u32),
        b'+' | b'-' => Ok(62),
        b'/' | b'_' => Ok(63),
        _ => Err(AppError::InvalidBase64),
    }
}

pub fn decode(input: &str) -> Result<Vec<u8>, AppError> {
    let payload = input.split_once(',').map_or(input, |(_, rest)| rest);
    let clean: Vec<u8> = payload
        .bytes()
        .filter(|c| !c.is_ascii_whitespace() && *c != b'=')
        .collect();
    if clean.len() % 4 == 1 {
        return Err(AppError::InvalidBase64);
    }
    let mut out = Vec::with_capacity(clean.len() * 3 / 4);
    for chunk in clean.chunks(4) {
        let mut acc = 0u32;
        for (i, c) in chunk.iter().enumerate() {
            acc |= value(*c)? << (18 - 6 * i);
        }
        let bytes = [(acc >> 16) as u8, (acc >> 8) as u8, acc as u8];
        out.extend_from_slice(&bytes[..chunk.len() - 1]);
    }
    Ok(out)
}

#[cfg(test)]
pub fn encode(bytes: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let mut acc = 0u32;
        for (i, b) in chunk.iter().enumerate() {
            acc |= (*b as u32) << (16 - 8 * i);
        }
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(ALPHABET[((acc >> (18 - 6 * i)) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_reverses_encode_for_arbitrary_bytes() {
        let bytes: Vec<u8> = (0..=255).collect();
        assert_eq!(decode(&encode(&bytes)).unwrap(), bytes);
    }

    #[test]
    fn decodes_padded_and_unpadded_input_to_the_original_bytes() {
        assert_eq!(decode("TWFu").unwrap(), b"Man");
        assert_eq!(decode("TWE=").unwrap(), b"Ma");
        assert_eq!(decode("TQ").unwrap(), b"M");
    }

    #[test]
    fn accepts_data_url_prefix_so_browser_payloads_work() {
        assert_eq!(decode("data:audio/wav;base64,TWFu").unwrap(), b"Man");
    }

    #[test]
    fn rejects_characters_outside_the_alphabet() {
        assert!(decode("TW*u").is_err());
    }
}
