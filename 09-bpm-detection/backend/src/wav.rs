use crate::error::AudioError;

#[derive(Debug, Clone, PartialEq)]
pub struct Audio {
    pub sample_rate: u32,
    pub samples: Vec<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum SampleFormat {
    Pcm16,
    Float32,
}

struct Format {
    sample_format: SampleFormat,
    channels: usize,
    sample_rate: u32,
}

const PCM: u16 = 1;
const IEEE_FLOAT: u16 = 3;
const EXTENSIBLE: u16 = 0xFFFE;

fn u16_at(bytes: &[u8], at: usize) -> Result<u16, AudioError> {
    bytes
        .get(at..at + 2)
        .map(|b| u16::from_le_bytes([b[0], b[1]]))
        .ok_or_else(|| AudioError::InvalidWav("unexpected end of data".into()))
}

fn u32_at(bytes: &[u8], at: usize) -> Result<u32, AudioError> {
    bytes
        .get(at..at + 4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .ok_or_else(|| AudioError::InvalidWav("unexpected end of data".into()))
}

fn parse_format(chunk: &[u8]) -> Result<Format, AudioError> {
    let mut tag = u16_at(chunk, 0)?;
    let channels = u16_at(chunk, 2)? as usize;
    let sample_rate = u32_at(chunk, 4)?;
    let bits = u16_at(chunk, 14)?;
    if tag == EXTENSIBLE {
        tag = u16_at(chunk, 24)?;
    }
    if channels == 0 || sample_rate == 0 {
        return Err(AudioError::InvalidWav(
            "zero channels or sample rate".into(),
        ));
    }
    let sample_format = match (tag, bits) {
        (PCM, 16) => SampleFormat::Pcm16,
        (IEEE_FLOAT, 32) => SampleFormat::Float32,
        _ => {
            return Err(AudioError::UnsupportedFormat(format!(
                "format tag {tag} with {bits} bits, only 16-bit PCM and 32-bit float are supported"
            )));
        }
    };
    Ok(Format {
        sample_format,
        channels,
        sample_rate,
    })
}

fn decode(data: &[u8], format: &Format) -> Vec<f32> {
    let width = match format.sample_format {
        SampleFormat::Pcm16 => 2,
        SampleFormat::Float32 => 4,
    };
    let frame_bytes = width * format.channels;
    data.chunks_exact(frame_bytes)
        .map(|frame| {
            let sum: f32 = frame
                .chunks_exact(width)
                .map(|s| match format.sample_format {
                    SampleFormat::Pcm16 => i16::from_le_bytes([s[0], s[1]]) as f32 / 32768.0,
                    SampleFormat::Float32 => f32::from_le_bytes([s[0], s[1], s[2], s[3]]),
                })
                .sum();
            sum / format.channels as f32
        })
        .collect()
}

pub fn read_wav(bytes: &[u8]) -> Result<Audio, AudioError> {
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(AudioError::InvalidWav("missing RIFF/WAVE header".into()));
    }
    let mut format = None;
    let mut pos = 12;
    while pos + 8 <= bytes.len() {
        let id = &bytes[pos..pos + 4];
        let size = u32_at(bytes, pos + 4)? as usize;
        let start = pos + 8;
        let end = start.saturating_add(size).min(bytes.len());
        let body = &bytes[start..end];
        if id == b"fmt " {
            format = Some(parse_format(body)?);
        } else if id == b"data" {
            let format = format
                .as_ref()
                .ok_or_else(|| AudioError::InvalidWav("data chunk before fmt chunk".into()))?;
            return Ok(Audio {
                sample_rate: format.sample_rate,
                samples: decode(body, format),
            });
        }
        pos = start.saturating_add(size).saturating_add(size % 2);
    }
    Err(AudioError::InvalidWav("no data chunk".into()))
}

pub fn write_wav_16(samples: &[f32], sample_rate: u32) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let mut out = Vec::with_capacity(44 + samples.len() * 2);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVE");
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&PCM.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&sample_rate.to_le_bytes());
    out.extend_from_slice(&(sample_rate * 2).to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for s in samples {
        let v = (s.clamp(-1.0, 1.0) * 32767.0).round() as i16;
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn float_stereo_wav(left: &[f32], right: &[f32], sample_rate: u32) -> Vec<u8> {
        let data_len = (left.len() * 8) as u32;
        let mut out = Vec::new();
        out.extend_from_slice(b"RIFF");
        out.extend_from_slice(&(36 + data_len).to_le_bytes());
        out.extend_from_slice(b"WAVE");
        out.extend_from_slice(b"fmt ");
        out.extend_from_slice(&16u32.to_le_bytes());
        out.extend_from_slice(&IEEE_FLOAT.to_le_bytes());
        out.extend_from_slice(&2u16.to_le_bytes());
        out.extend_from_slice(&sample_rate.to_le_bytes());
        out.extend_from_slice(&(sample_rate * 8).to_le_bytes());
        out.extend_from_slice(&8u16.to_le_bytes());
        out.extend_from_slice(&32u16.to_le_bytes());
        out.extend_from_slice(b"data");
        out.extend_from_slice(&data_len.to_le_bytes());
        for (l, r) in left.iter().zip(right) {
            out.extend_from_slice(&l.to_le_bytes());
            out.extend_from_slice(&r.to_le_bytes());
        }
        out
    }

    #[test]
    fn pcm16_round_trip_keeps_rate_and_values_within_quantization() {
        let samples = vec![0.0, 0.5, -0.5, 0.25, -1.0];
        let audio = read_wav(&write_wav_16(&samples, 22050)).unwrap();
        assert_eq!(audio.sample_rate, 22050);
        assert_eq!(audio.samples.len(), samples.len());
        for (a, b) in audio.samples.iter().zip(&samples) {
            assert!((a - b).abs() < 1.0 / 16000.0);
        }
    }

    #[test]
    fn stereo_float_is_mixed_down_to_channel_average() {
        let bytes = float_stereo_wav(&[1.0, 0.2], &[0.0, -0.2], 48000);
        let audio = read_wav(&bytes).unwrap();
        assert_eq!(audio.sample_rate, 48000);
        assert_eq!(audio.samples, vec![0.5, 0.0]);
    }

    #[test]
    fn non_wav_bytes_are_rejected() {
        assert!(matches!(
            read_wav(b"not a wav file at all"),
            Err(AudioError::InvalidWav(_))
        ));
    }

    #[test]
    fn eight_bit_pcm_is_reported_as_unsupported() {
        let mut bytes = write_wav_16(&[0.0; 4], 8000);
        bytes[34] = 8;
        assert!(matches!(
            read_wav(&bytes),
            Err(AudioError::UnsupportedFormat(_))
        ));
    }
}
