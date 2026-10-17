pub fn encode_wav(samples: &[f32], sample_rate: u32) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let mut out = Vec::with_capacity(44 + data_len as usize);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
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
    out.extend_from_slice(&data_len.to_le_bytes());
    for s in samples {
        let v = (s.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16;
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn u32_at(b: &[u8], i: usize) -> u32 {
        u32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]])
    }

    #[test]
    fn header_sizes_must_match_payload_so_players_read_the_whole_file() {
        let wav = encode_wav(&[0.0; 100], 44100);
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(u32_at(&wav, 4) as usize, wav.len() - 8);
        assert_eq!(u32_at(&wav, 24), 44100);
        assert_eq!(u32_at(&wav, 40), 200);
        assert_eq!(wav.len(), 244);
    }

    #[test]
    fn out_of_range_samples_must_clip_instead_of_wrapping_around() {
        let wav = encode_wav(&[2.0, -2.0], 8000);
        let hi = i16::from_le_bytes([wav[44], wav[45]]);
        let lo = i16::from_le_bytes([wav[46], wav[47]]);
        assert_eq!(hi, i16::MAX);
        assert_eq!(lo, -i16::MAX);
    }
}
