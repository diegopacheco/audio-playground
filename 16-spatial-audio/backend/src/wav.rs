use crate::audio::Stereo;
use crate::error::AppError;

const CHANNELS: u16 = 2;
const BITS_PER_SAMPLE: u16 = 16;

fn to_i16(sample: f32) -> i16 {
    (sample.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16
}

pub fn write_stereo(audio: &Stereo) -> Result<Vec<u8>, AppError> {
    if audio.left.len() != audio.right.len() {
        return Err(AppError::InvalidParam(format!(
            "channel lengths differ: left {} right {}",
            audio.left.len(),
            audio.right.len()
        )));
    }
    let block_align = CHANNELS * BITS_PER_SAMPLE / 8;
    let data_len = u32::try_from(audio.left.len() * block_align as usize)
        .map_err(|_| AppError::InvalidParam("audio too long for wav".into()))?;
    let byte_rate = audio.sample_rate * block_align as u32;
    let mut out = Vec::with_capacity(44 + data_len as usize);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVE");
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&CHANNELS.to_le_bytes());
    out.extend_from_slice(&audio.sample_rate.to_le_bytes());
    out.extend_from_slice(&byte_rate.to_le_bytes());
    out.extend_from_slice(&block_align.to_le_bytes());
    out.extend_from_slice(&BITS_PER_SAMPLE.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for (l, r) in audio.left.iter().zip(&audio.right) {
        out.extend_from_slice(&to_i16(*l).to_le_bytes());
        out.extend_from_slice(&to_i16(*r).to_le_bytes());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn u16_at(b: &[u8], i: usize) -> u16 {
        u16::from_le_bytes([b[i], b[i + 1]])
    }

    fn u32_at(b: &[u8], i: usize) -> u32 {
        u32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]])
    }

    #[test]
    fn header_describes_interleaved_16_bit_stereo_so_players_route_ears_correctly() {
        let audio = Stereo {
            sample_rate: 44_100,
            left: vec![1.0, 0.0, -1.0],
            right: vec![0.0, 0.5, 0.0],
        };
        let bytes = write_stereo(&audio).unwrap();
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WAVE");
        assert_eq!(u16_at(&bytes, 22), 2);
        assert_eq!(u32_at(&bytes, 24), 44_100);
        assert_eq!(u32_at(&bytes, 28), 44_100 * 4);
        assert_eq!(u16_at(&bytes, 32), 4);
        assert_eq!(u16_at(&bytes, 34), 16);
        assert_eq!(u32_at(&bytes, 40), 12);
        assert_eq!(bytes.len(), 44 + 12);
        assert_eq!(i16::from_le_bytes([bytes[44], bytes[45]]), i16::MAX);
        assert_eq!(i16::from_le_bytes([bytes[46], bytes[47]]), 0);
        assert_eq!(i16::from_le_bytes([bytes[50], bytes[51]]), 16_384);
    }

    #[test]
    fn mismatched_channels_are_rejected_instead_of_silently_truncated() {
        let audio = Stereo {
            sample_rate: 44_100,
            left: vec![0.0; 3],
            right: vec![0.0; 2],
        };
        assert!(write_stereo(&audio).is_err());
    }
}
