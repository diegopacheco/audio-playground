use crate::error::FingerprintError;

#[derive(Debug, Clone, PartialEq)]
pub struct Audio {
    pub sample_rate: u32,
    pub samples: Vec<f32>,
}

struct Format {
    audio_format: u16,
    channels: u16,
    sample_rate: u32,
    bits: u16,
}

pub fn is_wav(bytes: &[u8]) -> bool {
    bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WAVE"
}

fn read_u16(bytes: &[u8], at: usize) -> Result<u16, FingerprintError> {
    bytes
        .get(at..at + 2)
        .map(|b| u16::from_le_bytes([b[0], b[1]]))
        .ok_or_else(|| FingerprintError::InvalidWav("truncated header".into()))
}

fn read_u32(bytes: &[u8], at: usize) -> Result<u32, FingerprintError> {
    bytes
        .get(at..at + 4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .ok_or_else(|| FingerprintError::InvalidWav("truncated header".into()))
}

fn parse_format(chunk: &[u8]) -> Result<Format, FingerprintError> {
    let mut audio_format = read_u16(chunk, 0)?;
    let channels = read_u16(chunk, 2)?;
    let sample_rate = read_u32(chunk, 4)?;
    let bits = read_u16(chunk, 14)?;
    if audio_format == 0xFFFE {
        audio_format = read_u16(chunk, 24)?;
    }
    if channels == 0 {
        return Err(FingerprintError::InvalidWav("zero channels".into()));
    }
    if sample_rate == 0 {
        return Err(FingerprintError::InvalidSampleRate(0));
    }
    Ok(Format {
        audio_format,
        channels,
        sample_rate,
        bits,
    })
}

fn decode_sample(format: &Format, b: &[u8]) -> Result<f32, FingerprintError> {
    match (format.audio_format, format.bits) {
        (1, 8) => Ok((b[0] as f32 - 128.0) / 128.0),
        (1, 16) => Ok(i16::from_le_bytes([b[0], b[1]]) as f32 / 32768.0),
        (1, 24) => {
            let v = i32::from_le_bytes([0, b[0], b[1], b[2]]) >> 8;
            Ok(v as f32 / 8_388_608.0)
        }
        (1, 32) => Ok(i32::from_le_bytes([b[0], b[1], b[2], b[3]]) as f32 / 2_147_483_648.0),
        (3, 32) => Ok(f32::from_le_bytes([b[0], b[1], b[2], b[3]])),
        (fmt, bits) => Err(FingerprintError::UnsupportedFormat(format!(
            "format {fmt} with {bits} bits"
        ))),
    }
}

fn decode_data(format: &Format, data: &[u8]) -> Result<Vec<f32>, FingerprintError> {
    let width = (format.bits as usize).div_ceil(8);
    let channels = format.channels as usize;
    let frame = width * channels;
    let mut samples = Vec::with_capacity(data.len() / frame.max(1));
    for chunk in data.chunks_exact(frame) {
        let mut sum = 0.0;
        for c in 0..channels {
            sum += decode_sample(format, &chunk[c * width..(c + 1) * width])?;
        }
        samples.push(sum / channels as f32);
    }
    Ok(samples)
}

pub fn read(bytes: &[u8]) -> Result<Audio, FingerprintError> {
    if !is_wav(bytes) {
        return Err(FingerprintError::InvalidWav(
            "missing RIFF/WAVE header".into(),
        ));
    }
    let mut pos = 12;
    let mut format = None;
    let mut data = None;
    while pos + 8 <= bytes.len() {
        let id = &bytes[pos..pos + 4];
        let size = read_u32(bytes, pos + 4)? as usize;
        let start = pos + 8;
        let end = (start + size).min(bytes.len());
        match id {
            b"fmt " => format = Some(parse_format(&bytes[start..end])?),
            b"data" => data = Some(&bytes[start..end]),
            _ => {}
        }
        pos = start + size + (size % 2);
    }
    let format = format.ok_or_else(|| FingerprintError::InvalidWav("missing fmt chunk".into()))?;
    let data = data.ok_or_else(|| FingerprintError::InvalidWav("missing data chunk".into()))?;
    let samples = decode_data(&format, data)?;
    if samples.is_empty() {
        return Err(FingerprintError::EmptyAudio);
    }
    Ok(Audio {
        sample_rate: format.sample_rate,
        samples,
    })
}

pub fn write(audio: &Audio) -> Vec<u8> {
    let data_len = (audio.samples.len() * 2) as u32;
    let mut out = Vec::with_capacity(44 + data_len as usize);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&audio.sample_rate.to_le_bytes());
    out.extend_from_slice(&(audio.sample_rate * 2).to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for s in &audio.samples {
        let v = (s.clamp(-1.0, 1.0) * 32767.0).round() as i16;
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

pub fn read_f32_pcm(bytes: &[u8], sample_rate: u32) -> Result<Audio, FingerprintError> {
    if sample_rate == 0 {
        return Err(FingerprintError::InvalidSampleRate(0));
    }
    let samples: Vec<f32> = bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|b| f32::from_le_bytes(*b))
        .map(|v| if v.is_finite() { v } else { 0.0 })
        .collect();
    if samples.is_empty() {
        return Err(FingerprintError::EmptyAudio);
    }
    Ok(Audio {
        sample_rate,
        samples,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_keeps_rate_length_and_values_within_16bit_precision() {
        let audio = Audio {
            sample_rate: 8000,
            samples: (0..800).map(|i| (i as f32 * 0.05).sin() * 0.8).collect(),
        };
        let back = read(&write(&audio)).unwrap();
        assert_eq!(back.sample_rate, 8000);
        assert_eq!(back.samples.len(), 800);
        for (a, b) in audio.samples.iter().zip(&back.samples) {
            assert!((a - b).abs() < 1.0 / 16000.0);
        }
    }

    #[test]
    fn stereo_is_mixed_to_mono_so_fingerprints_ignore_channel_layout() {
        let mut bytes = write(&Audio {
            sample_rate: 8000,
            samples: vec![0.5, -0.5, 0.25, 0.25],
        });
        bytes[22] = 2;
        bytes[28..32].copy_from_slice(&(8000u32 * 4).to_le_bytes());
        bytes[32] = 4;
        let audio = read(&bytes).unwrap();
        assert_eq!(audio.samples.len(), 2);
        assert!(audio.samples[0].abs() < 1e-3);
        assert!((audio.samples[1] - 0.25).abs() < 1e-3);
    }

    #[test]
    fn rejects_non_wav_bytes_instead_of_guessing() {
        assert!(matches!(
            read(b"not a wav file at all"),
            Err(FingerprintError::InvalidWav(_))
        ));
    }

    #[test]
    fn float_pcm_is_decoded_little_endian() {
        let bytes: Vec<u8> = [0.5f32, -0.25]
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect();
        let audio = read_f32_pcm(&bytes, 44100).unwrap();
        assert_eq!(audio.samples, vec![0.5, -0.25]);
    }
}
