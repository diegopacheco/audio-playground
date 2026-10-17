use crate::error::AppError;

const FORMAT_PCM: u16 = 1;
const FORMAT_FLOAT: u16 = 3;
const FORMAT_EXTENSIBLE: u16 = 0xFFFE;

pub struct Audio {
    pub sample_rate: u32,
    pub samples: Vec<f32>,
}

struct Format {
    code: u16,
    channels: u16,
    sample_rate: u32,
    bits: u16,
}

pub fn encode(audio: &Audio) -> Vec<u8> {
    let data_len = (audio.samples.len() * 2) as u32;
    let mut out = Vec::with_capacity(44 + data_len as usize);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVE");
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&FORMAT_PCM.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&audio.sample_rate.to_le_bytes());
    out.extend_from_slice(&(audio.sample_rate * 2).to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for sample in &audio.samples {
        let value = (sample.clamp(-1.0, 1.0) * 32767.0).round() as i16;
        out.extend_from_slice(&value.to_le_bytes());
    }
    out
}

pub fn is_wav(bytes: &[u8]) -> bool {
    bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WAVE"
}

pub fn decode(bytes: &[u8]) -> Result<Audio, AppError> {
    if !is_wav(bytes) {
        return Err(AppError::Wav("missing RIFF/WAVE header".into()));
    }
    let mut format = None;
    let mut offset = 12;
    while offset + 8 <= bytes.len() {
        let id = &bytes[offset..offset + 4];
        let size = read_u32(bytes, offset + 4)? as usize;
        let start = offset + 8;
        let end = start
            .checked_add(size)
            .filter(|end| *end <= bytes.len())
            .ok_or_else(|| AppError::Wav("chunk exceeds file length".into()))?;
        match id {
            b"fmt " => format = Some(parse_format(&bytes[start..end])?),
            b"data" => {
                let format =
                    format.ok_or_else(|| AppError::Wav("data chunk before fmt chunk".into()))?;
                return decode_samples(&format, &bytes[start..end]);
            }
            _ => {}
        }
        offset = end + (size & 1);
    }
    Err(AppError::Wav("no data chunk".into()))
}

fn parse_format(chunk: &[u8]) -> Result<Format, AppError> {
    if chunk.len() < 16 {
        return Err(AppError::Wav("fmt chunk too short".into()));
    }
    let mut code = read_u16(chunk, 0)?;
    if code == FORMAT_EXTENSIBLE {
        code = read_u16(chunk, 24)?;
    }
    let format = Format {
        code,
        channels: read_u16(chunk, 2)?,
        sample_rate: read_u32(chunk, 4)?,
        bits: read_u16(chunk, 14)?,
    };
    if format.channels == 0 || format.sample_rate == 0 {
        return Err(AppError::Wav("zero channels or sample rate".into()));
    }
    Ok(format)
}

fn decode_samples(format: &Format, data: &[u8]) -> Result<Audio, AppError> {
    let width = match (format.code, format.bits) {
        (FORMAT_PCM, 16) => 2,
        (FORMAT_FLOAT, 32) => 4,
        (code, bits) => {
            return Err(AppError::Wav(format!(
                "unsupported format code {code} with {bits} bits"
            )));
        }
    };
    let channels = usize::from(format.channels);
    let frame = width * channels;
    let samples = data
        .chunks_exact(frame)
        .map(|chunk| {
            let sum: f32 = chunk
                .chunks_exact(width)
                .map(|s| match width {
                    2 => f32::from(i16::from_le_bytes([s[0], s[1]])) / 32768.0,
                    _ => f32::from_le_bytes([s[0], s[1], s[2], s[3]]),
                })
                .sum();
            sum / channels as f32
        })
        .collect();
    Ok(Audio {
        sample_rate: format.sample_rate,
        samples,
    })
}

pub fn decode_f32_le(bytes: &[u8]) -> Result<Vec<f32>, AppError> {
    if !bytes.len().is_multiple_of(4) {
        return Err(AppError::InvalidInput(
            "float32 pcm length must be a multiple of 4".into(),
        ));
    }
    Ok(bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|c| f32::from_le_bytes(*c))
        .collect())
}

fn read_u16(bytes: &[u8], at: usize) -> Result<u16, AppError> {
    bytes
        .get(at..at + 2)
        .map(|b| u16::from_le_bytes([b[0], b[1]]))
        .ok_or_else(|| AppError::Wav("unexpected end of file".into()))
}

fn read_u32(bytes: &[u8], at: usize) -> Result<u32, AppError> {
    bytes
        .get(at..at + 4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .ok_or_else(|| AppError::Wav("unexpected end of file".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pcm16_round_trip_keeps_samples_within_one_quantization_step() {
        let samples: Vec<f32> = (0..500).map(|i| (i as f32 * 0.05).sin() * 0.8).collect();
        let bytes = encode(&Audio {
            sample_rate: 48000,
            samples: samples.clone(),
        });
        assert_eq!(bytes.len(), 44 + samples.len() * 2);
        let audio = decode(&bytes).unwrap();
        assert_eq!(audio.sample_rate, 48000);
        assert_eq!(audio.samples.len(), samples.len());
        for (a, b) in audio.samples.iter().zip(&samples) {
            assert!((a - b).abs() < 2.0 / 32768.0);
        }
    }

    #[test]
    fn float_stereo_wav_is_mixed_down_to_mono_so_any_recording_can_be_demodulated() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&(36u32 + 16).to_le_bytes());
        bytes.extend_from_slice(b"WAVE");
        bytes.extend_from_slice(b"fmt ");
        bytes.extend_from_slice(&16u32.to_le_bytes());
        bytes.extend_from_slice(&FORMAT_FLOAT.to_le_bytes());
        bytes.extend_from_slice(&2u16.to_le_bytes());
        bytes.extend_from_slice(&44100u32.to_le_bytes());
        bytes.extend_from_slice(&(44100u32 * 8).to_le_bytes());
        bytes.extend_from_slice(&8u16.to_le_bytes());
        bytes.extend_from_slice(&32u16.to_le_bytes());
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&16u32.to_le_bytes());
        for value in [0.5f32, 0.1, -0.2, -0.4] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        let audio = decode(&bytes).unwrap();
        assert_eq!(audio.sample_rate, 44100);
        assert_eq!(audio.samples.len(), 2);
        assert!((audio.samples[0] - 0.3).abs() < 1e-6);
        assert!((audio.samples[1] + 0.3).abs() < 1e-6);
    }

    #[test]
    fn rejects_bytes_without_riff_header() {
        assert!(decode(b"not a wav file at all").is_err());
    }

    #[test]
    fn rejects_truncated_chunk_instead_of_reading_out_of_bounds() {
        let mut bytes = encode(&Audio {
            sample_rate: 8000,
            samples: vec![0.0; 10],
        });
        bytes.truncate(50);
        assert!(decode(&bytes).is_err());
    }
}
