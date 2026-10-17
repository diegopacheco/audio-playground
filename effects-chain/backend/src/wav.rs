use crate::error::AppError;

#[derive(Debug, Clone, PartialEq)]
pub struct Audio {
    pub sample_rate: u32,
    pub samples: Vec<f32>,
}

struct Format {
    encoding: u16,
    channels: u16,
    sample_rate: u32,
    bits: u16,
}

fn u16_at(b: &[u8], i: usize) -> Result<u16, AppError> {
    b.get(i..i + 2)
        .map(|s| u16::from_le_bytes([s[0], s[1]]))
        .ok_or_else(|| AppError::InvalidWav("truncated header".into()))
}

fn u32_at(b: &[u8], i: usize) -> Result<u32, AppError> {
    b.get(i..i + 4)
        .map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
        .ok_or_else(|| AppError::InvalidWav("truncated header".into()))
}

fn parse_format(chunk: &[u8]) -> Result<Format, AppError> {
    let mut encoding = u16_at(chunk, 0)?;
    if encoding == 0xFFFE {
        encoding = u16_at(chunk, 24)?;
    }
    Ok(Format {
        encoding,
        channels: u16_at(chunk, 2)?,
        sample_rate: u32_at(chunk, 4)?,
        bits: u16_at(chunk, 14)?,
    })
}

fn decode_sample(encoding: u16, bits: u16, s: &[u8]) -> Result<f32, AppError> {
    match (encoding, bits) {
        (1, 8) => Ok((s[0] as f32 - 128.0) / 128.0),
        (1, 16) => Ok(i16::from_le_bytes([s[0], s[1]]) as f32 / 32768.0),
        (1, 24) => Ok((i32::from_le_bytes([0, s[0], s[1], s[2]]) >> 8) as f32 / 8_388_608.0),
        (1, 32) => Ok(i32::from_le_bytes([s[0], s[1], s[2], s[3]]) as f32 / 2_147_483_648.0),
        (3, 32) => Ok(f32::from_le_bytes([s[0], s[1], s[2], s[3]])),
        _ => Err(AppError::UnsupportedWav(format!(
            "encoding {encoding} with {bits} bits"
        ))),
    }
}

fn decode_data(format: &Format, data: &[u8]) -> Result<Vec<f32>, AppError> {
    let width = (format.bits / 8) as usize;
    let channels = format.channels as usize;
    if width == 0 || channels == 0 {
        return Err(AppError::InvalidWav("zero channels or bit depth".into()));
    }
    let frame = width * channels;
    data.chunks_exact(frame)
        .map(|f| {
            let mut sum = 0.0;
            for c in 0..channels {
                sum += decode_sample(format.encoding, format.bits, &f[c * width..(c + 1) * width])?;
            }
            Ok(sum / channels as f32)
        })
        .collect()
}

pub fn read(bytes: &[u8]) -> Result<Audio, AppError> {
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(AppError::InvalidWav("missing RIFF/WAVE header".into()));
    }
    let mut pos = 12;
    let mut format = None;
    let mut data = None;
    while pos + 8 <= bytes.len() {
        let id = &bytes[pos..pos + 4];
        let size = u32_at(bytes, pos + 4)? as usize;
        let start = pos + 8;
        let end = start.saturating_add(size).min(bytes.len());
        match id {
            b"fmt " => format = Some(parse_format(&bytes[start..end])?),
            b"data" => data = Some(&bytes[start..end]),
            _ => {}
        }
        pos = start.saturating_add(size).saturating_add(size % 2);
    }
    let format = format.ok_or_else(|| AppError::InvalidWav("missing fmt chunk".into()))?;
    let data = data.ok_or_else(|| AppError::InvalidWav("missing data chunk".into()))?;
    if format.sample_rate == 0 {
        return Err(AppError::InvalidWav("sample rate is zero".into()));
    }
    Ok(Audio {
        sample_rate: format.sample_rate,
        samples: decode_data(&format, data)?,
    })
}

pub fn write(audio: &Audio) -> Vec<u8> {
    let data_len = (audio.samples.len() * 2) as u32;
    let mut out = Vec::with_capacity(44 + data_len as usize);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVE");
    out.extend_from_slice(b"fmt ");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_then_read_preserves_rate_and_samples_within_16_bit_precision() {
        let audio = Audio {
            sample_rate: 22050,
            samples: vec![0.0, 0.5, -0.5, 0.25, -1.0],
        };
        let back = read(&write(&audio)).unwrap();
        assert_eq!(back.sample_rate, 22050);
        assert_eq!(back.samples.len(), audio.samples.len());
        for (a, b) in audio.samples.iter().zip(&back.samples) {
            assert!((a - b).abs() < 1.0 / 16000.0);
        }
    }

    #[test]
    fn out_of_range_samples_are_clipped_instead_of_wrapping_around() {
        let audio = Audio {
            sample_rate: 8000,
            samples: vec![3.0, -3.0],
        };
        let back = read(&write(&audio)).unwrap();
        assert!(back.samples[0] > 0.99);
        assert!(back.samples[1] < -0.99);
    }

    #[test]
    fn stereo_input_is_mixed_down_to_mono() {
        let mut bytes = write(&Audio {
            sample_rate: 8000,
            samples: vec![0.5, -0.5],
        });
        bytes[22] = 2;
        bytes[32] = 4;
        let audio = read(&bytes).unwrap();
        assert_eq!(audio.samples.len(), 1);
        assert!(audio.samples[0].abs() < 1e-3);
    }

    #[test]
    fn rejects_bytes_that_are_not_a_wav_file() {
        assert!(read(b"not a wav file at all").is_err());
    }
}
