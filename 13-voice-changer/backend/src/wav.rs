use crate::error::VoiceError;

const PCM: u16 = 1;
const FLOAT: u16 = 3;
const EXTENSIBLE: u16 = 0xFFFE;

#[derive(Debug, Clone, PartialEq)]
pub struct Wav {
    pub sample_rate: u32,
    pub samples: Vec<f32>,
}

struct Format {
    code: u16,
    channels: u16,
    sample_rate: u32,
    bits: u16,
}

fn u16_at(bytes: &[u8], at: usize) -> Result<u16, VoiceError> {
    bytes
        .get(at..at + 2)
        .map(|b| u16::from_le_bytes([b[0], b[1]]))
        .ok_or_else(|| VoiceError::InvalidWav("truncated header".into()))
}

fn u32_at(bytes: &[u8], at: usize) -> Result<u32, VoiceError> {
    bytes
        .get(at..at + 4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .ok_or_else(|| VoiceError::InvalidWav("truncated header".into()))
}

fn parse_format(chunk: &[u8]) -> Result<Format, VoiceError> {
    let mut code = u16_at(chunk, 0)?;
    if code == EXTENSIBLE {
        code = u16_at(chunk, 24)?;
    }
    Ok(Format {
        code,
        channels: u16_at(chunk, 2)?,
        sample_rate: u32_at(chunk, 4)?,
        bits: u16_at(chunk, 14)?,
    })
}

fn decode_sample(format: &Format, b: &[u8]) -> Result<f32, VoiceError> {
    Ok(match (format.code, format.bits) {
        (PCM, 8) => (b[0] as f32 - 128.0) / 128.0,
        (PCM, 16) => i16::from_le_bytes([b[0], b[1]]) as f32 / 32768.0,
        (PCM, 24) => (i32::from_le_bytes([0, b[0], b[1], b[2]]) >> 8) as f32 / 8_388_608.0,
        (PCM, 32) => i32::from_le_bytes([b[0], b[1], b[2], b[3]]) as f32 / 2_147_483_648.0,
        (FLOAT, 32) => f32::from_le_bytes([b[0], b[1], b[2], b[3]]),
        (code, bits) => {
            return Err(VoiceError::UnsupportedWav(format!(
                "format {code} with {bits} bits"
            )));
        }
    })
}

pub fn decode(bytes: &[u8]) -> Result<Wav, VoiceError> {
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(VoiceError::InvalidWav("missing RIFF/WAVE header".into()));
    }
    let mut format = None;
    let mut data = None;
    let mut at = 12;
    while at + 8 <= bytes.len() {
        let id = &bytes[at..at + 4];
        let size = u32_at(bytes, at + 4)? as usize;
        let start = at + 8;
        let end = start.saturating_add(size).min(bytes.len());
        match id {
            b"fmt " => format = Some(parse_format(&bytes[start..end])?),
            b"data" => data = Some(&bytes[start..end]),
            _ => {}
        }
        at = start.saturating_add(size).saturating_add(size % 2);
    }
    let format = format.ok_or_else(|| VoiceError::InvalidWav("missing fmt chunk".into()))?;
    let data = data.ok_or_else(|| VoiceError::InvalidWav("missing data chunk".into()))?;
    if format.channels == 0 || format.sample_rate == 0 || format.bits % 8 != 0 || format.bits == 0 {
        return Err(VoiceError::InvalidWav("bad format values".into()));
    }
    let width = format.bits as usize / 8;
    let channels = format.channels as usize;
    let mut samples = Vec::with_capacity(data.len() / (width * channels));
    for frame in data.chunks_exact(width * channels) {
        let mut sum = 0.0;
        for sample in frame.chunks_exact(width) {
            sum += decode_sample(&format, sample)?;
        }
        samples.push(sum / channels as f32);
    }
    Ok(Wav {
        sample_rate: format.sample_rate,
        samples,
    })
}

pub fn encode(wav: &Wav) -> Vec<u8> {
    let data_len = (wav.samples.len() * 2) as u32;
    let mut out = Vec::with_capacity(44 + data_len as usize);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVE");
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&PCM.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&wav.sample_rate.to_le_bytes());
    out.extend_from_slice(&(wav.sample_rate * 2).to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for sample in &wav.samples {
        let value = (sample.clamp(-1.0, 1.0) * 32767.0).round() as i16;
        out.extend_from_slice(&value.to_le_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_then_decode_keeps_rate_and_samples_within_16_bit_precision() {
        let wav = Wav {
            sample_rate: 22050,
            samples: vec![0.0, 0.5, -0.5, 1.0, -1.0, 0.123],
        };
        let decoded = decode(&encode(&wav)).unwrap();
        assert_eq!(decoded.sample_rate, 22050);
        for (a, b) in decoded.samples.iter().zip(&wav.samples) {
            assert!((a - b).abs() < 1.0 / 16000.0);
        }
    }

    #[test]
    fn stereo_float_is_mixed_down_to_mono_so_effects_see_one_voice() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&(36u32 + 16).to_le_bytes());
        bytes.extend_from_slice(b"WAVEfmt ");
        bytes.extend_from_slice(&16u32.to_le_bytes());
        bytes.extend_from_slice(&FLOAT.to_le_bytes());
        bytes.extend_from_slice(&2u16.to_le_bytes());
        bytes.extend_from_slice(&8000u32.to_le_bytes());
        bytes.extend_from_slice(&64000u32.to_le_bytes());
        bytes.extend_from_slice(&8u16.to_le_bytes());
        bytes.extend_from_slice(&32u16.to_le_bytes());
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&16u32.to_le_bytes());
        for value in [1.0f32, 0.0, 0.5, -0.5] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        let wav = decode(&bytes).unwrap();
        assert_eq!(wav.sample_rate, 8000);
        assert_eq!(wav.samples, vec![0.5, 0.0]);
    }

    #[test]
    fn rejects_files_that_are_not_riff_wave() {
        assert!(matches!(
            decode(b"OggS0000000000"),
            Err(VoiceError::InvalidWav(_))
        ));
    }
}
