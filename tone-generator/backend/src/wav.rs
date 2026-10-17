pub const HEADER_LEN: usize = 44;
const CHANNELS: u16 = 1;
const BITS_PER_SAMPLE: u16 = 16;
const PCM_FORMAT: u16 = 1;
const FMT_CHUNK_LEN: u32 = 16;

pub fn encode_header(sample_rate: u32, sample_count: usize) -> [u8; HEADER_LEN] {
    let block_align = CHANNELS * BITS_PER_SAMPLE / 8;
    let byte_rate = sample_rate * u32::from(block_align);
    let data_len = (sample_count * usize::from(block_align)) as u32;
    let mut h = [0u8; HEADER_LEN];
    h[0..4].copy_from_slice(b"RIFF");
    h[4..8].copy_from_slice(&(36 + data_len).to_le_bytes());
    h[8..12].copy_from_slice(b"WAVE");
    h[12..16].copy_from_slice(b"fmt ");
    h[16..20].copy_from_slice(&FMT_CHUNK_LEN.to_le_bytes());
    h[20..22].copy_from_slice(&PCM_FORMAT.to_le_bytes());
    h[22..24].copy_from_slice(&CHANNELS.to_le_bytes());
    h[24..28].copy_from_slice(&sample_rate.to_le_bytes());
    h[28..32].copy_from_slice(&byte_rate.to_le_bytes());
    h[32..34].copy_from_slice(&block_align.to_le_bytes());
    h[34..36].copy_from_slice(&BITS_PER_SAMPLE.to_le_bytes());
    h[36..40].copy_from_slice(b"data");
    h[40..44].copy_from_slice(&data_len.to_le_bytes());
    h
}

pub fn encode(sample_rate: u32, samples: &[i16]) -> Vec<u8> {
    let mut out = Vec::with_capacity(HEADER_LEN + samples.len() * 2);
    out.extend_from_slice(&encode_header(sample_rate, samples.len()));
    for s in samples {
        out.extend_from_slice(&s.to_le_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn u32_at(b: &[u8], i: usize) -> u32 {
        u32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]])
    }

    fn u16_at(b: &[u8], i: usize) -> u16 {
        u16::from_le_bytes([b[i], b[i + 1]])
    }

    #[test]
    fn header_matches_the_canonical_44_byte_riff_layout_byte_for_byte() {
        let h = encode_header(44100, 1000);
        let expected: [u8; 44] = [
            b'R', b'I', b'F', b'F', 0xF4, 0x07, 0x00, 0x00, b'W', b'A', b'V', b'E', b'f', b'm',
            b't', b' ', 0x10, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x44, 0xAC, 0x00, 0x00,
            0x88, 0x58, 0x01, 0x00, 0x02, 0x00, 0x10, 0x00, b'd', b'a', b't', b'a', 0xD0, 0x07,
            0x00, 0x00,
        ];
        assert_eq!(h, expected);
    }

    #[test]
    fn riff_size_is_file_length_minus_8_so_players_know_where_the_file_ends() {
        let bytes = encode(8000, &[0; 123]);
        assert_eq!(u32_at(&bytes, 4) as usize, bytes.len() - 8);
        assert_eq!(u32_at(&bytes, 40) as usize, bytes.len() - HEADER_LEN);
    }

    #[test]
    fn byte_rate_and_block_align_follow_from_mono_16_bit_so_seeking_is_correct() {
        let h = encode_header(48000, 1);
        assert_eq!(u16_at(&h, 22), 1);
        assert_eq!(u16_at(&h, 34), 16);
        assert_eq!(u16_at(&h, 32), 2);
        assert_eq!(u32_at(&h, 28), 96000);
    }

    #[test]
    fn samples_are_written_little_endian_after_the_header() {
        let bytes = encode(8000, &[1, -1, i16::MAX, i16::MIN]);
        assert_eq!(
            &bytes[HEADER_LEN..],
            &[0x01, 0x00, 0xFF, 0xFF, 0xFF, 0x7F, 0x00, 0x80]
        );
    }
}
