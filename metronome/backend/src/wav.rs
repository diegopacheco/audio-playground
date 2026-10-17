use crate::error::AppError;

const HEADER_LEN: usize = 44;

pub fn encode_mono_16(samples: &[f32], sample_rate: u32) -> Result<Vec<u8>, AppError> {
    let data_len = samples
        .len()
        .checked_mul(2)
        .filter(|n| *n <= (u32::MAX as usize) - HEADER_LEN)
        .ok_or(AppError::WavTooLarge(samples.len()))?;
    let data_len_u32 = data_len as u32;
    let mut out = Vec::with_capacity(HEADER_LEN + data_len);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len_u32).to_le_bytes());
    out.extend_from_slice(b"WAVE");
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&sample_rate.to_le_bytes());
    out.extend_from_slice(&(sample_rate * 2).to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len_u32.to_le_bytes());
    for s in samples {
        let v = (s.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16;
        out.extend_from_slice(&v.to_le_bytes());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn u32_at(b: &[u8], i: usize) -> u32 {
        u32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]])
    }

    #[test]
    fn header_sizes_match_payload_so_players_read_the_whole_track() {
        let bytes = encode_mono_16(&[0.0; 100], 44_100).unwrap();
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WAVE");
        assert_eq!(u32_at(&bytes, 4) as usize, bytes.len() - 8);
        assert_eq!(u32_at(&bytes, 24), 44_100);
        assert_eq!(u32_at(&bytes, 40), 200);
        assert_eq!(bytes.len(), 44 + 200);
    }

    #[test]
    fn samples_are_clamped_and_scaled_to_full_16_bit_range() {
        let bytes = encode_mono_16(&[1.0, -2.0, 0.0], 8_000).unwrap();
        let s = |i: usize| i16::from_le_bytes([bytes[44 + i * 2], bytes[45 + i * 2]]);
        assert_eq!(s(0), i16::MAX);
        assert_eq!(s(1), -i16::MAX);
        assert_eq!(s(2), 0);
    }
}
