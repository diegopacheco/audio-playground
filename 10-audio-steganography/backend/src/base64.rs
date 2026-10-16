use crate::error::{Result, StegoError};

fn value(c: u8) -> Result<u32> {
    match c {
        b'A'..=b'Z' => Ok((c - b'A') as u32),
        b'a'..=b'z' => Ok((c - b'a' + 26) as u32),
        b'0'..=b'9' => Ok((c - b'0' + 52) as u32),
        b'+' => Ok(62),
        b'/' => Ok(63),
        _ => Err(StegoError::InvalidBase64),
    }
}

pub fn decode(input: &str) -> Result<Vec<u8>> {
    let data = input.rsplit(',').next().unwrap_or(input);
    let clean: Vec<u8> = data
        .bytes()
        .filter(|c| !c.is_ascii_whitespace() && *c != b'=')
        .collect();
    if clean.len() % 4 == 1 {
        return Err(StegoError::InvalidBase64);
    }
    let mut out = Vec::with_capacity(clean.len() * 3 / 4);
    for chunk in clean.chunks(4) {
        let mut acc = 0u32;
        for (i, &c) in chunk.iter().enumerate() {
            acc |= value(c)? << (18 - 6 * i);
        }
        let bytes = acc.to_be_bytes();
        out.extend_from_slice(&bytes[1..chunk.len()]);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_padded_and_unpadded_input_from_browser_file_readers() {
        assert_eq!(decode("aGVsbG8=").unwrap(), b"hello");
        assert_eq!(decode("aGVsbG8").unwrap(), b"hello");
        assert_eq!(decode("TWFu").unwrap(), b"Man");
    }

    #[test]
    fn strips_data_url_prefix() {
        assert_eq!(decode("data:audio/wav;base64,TWE=").unwrap(), b"Ma");
    }

    #[test]
    fn rejects_characters_outside_the_alphabet() {
        assert_eq!(decode("ab$d"), Err(StegoError::InvalidBase64));
    }
}
