use crate::error::{Result, StegoError};

#[derive(Debug, Clone, PartialEq)]
pub struct Audio {
    pub sample_rate: u32,
    pub samples: Vec<i16>,
}

fn u16_at(b: &[u8], i: usize) -> Result<u16> {
    b.get(i..i + 2)
        .map(|s| u16::from_le_bytes([s[0], s[1]]))
        .ok_or_else(|| StegoError::InvalidWav("truncated".into()))
}

fn u32_at(b: &[u8], i: usize) -> Result<u32> {
    b.get(i..i + 4)
        .map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
        .ok_or_else(|| StegoError::InvalidWav("truncated".into()))
}

struct Format {
    channels: u16,
    sample_rate: u32,
}

fn parse_format(chunk: &[u8]) -> Result<Format> {
    let format = u16_at(chunk, 0)?;
    let channels = u16_at(chunk, 2)?;
    let sample_rate = u32_at(chunk, 4)?;
    let bits = u16_at(chunk, 14)?;
    if format != 1 {
        return Err(StegoError::UnsupportedWav(format!(
            "format {format}, only PCM"
        )));
    }
    if bits != 16 {
        return Err(StegoError::UnsupportedWav(format!(
            "{bits} bits, only 16-bit"
        )));
    }
    if channels == 0 {
        return Err(StegoError::InvalidWav("zero channels".into()));
    }
    Ok(Format {
        channels,
        sample_rate,
    })
}

fn downmix(data: &[u8], channels: u16) -> Vec<i16> {
    let ch = channels as usize;
    data.chunks_exact(2 * ch)
        .map(|frame| {
            let sum: i32 = frame
                .as_chunks::<2>()
                .0
                .iter()
                .map(|s| i16::from_le_bytes(*s) as i32)
                .sum();
            (sum / ch as i32) as i16
        })
        .collect()
}

pub fn read(bytes: &[u8]) -> Result<Audio> {
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(StegoError::InvalidWav("missing RIFF/WAVE header".into()));
    }
    let mut pos = 12;
    let mut format = None;
    while pos + 8 <= bytes.len() {
        let id = &bytes[pos..pos + 4];
        let size = u32_at(bytes, pos + 4)? as usize;
        let start = pos + 8;
        let end = start.saturating_add(size).min(bytes.len());
        let body = &bytes[start..end];
        if id == b"fmt " {
            format = Some(parse_format(body)?);
        } else if id == b"data" {
            let f = format.ok_or_else(|| StegoError::InvalidWav("data before fmt".into()))?;
            return Ok(Audio {
                sample_rate: f.sample_rate,
                samples: downmix(body, f.channels),
            });
        }
        pos = start.saturating_add(size).saturating_add(size % 2);
    }
    Err(StegoError::InvalidWav("no data chunk".into()))
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
        out.extend_from_slice(&s.to_le_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_then_read_is_lossless_so_hidden_bits_survive_the_file_format() {
        let audio = Audio {
            sample_rate: 44_100,
            samples: vec![0, 1, -1, i16::MAX, i16::MIN, 1234, -4321],
        };
        assert_eq!(read(&write(&audio)).unwrap(), audio);
    }

    #[test]
    fn stereo_is_downmixed_to_mono_by_averaging_channels() {
        let mut bytes = write(&Audio {
            sample_rate: 8000,
            samples: vec![100, 300, -50, 50],
        });
        bytes[22] = 2;
        let audio = read(&bytes).unwrap();
        assert_eq!(audio.samples, vec![200, 0]);
    }

    #[test]
    fn non_wav_bytes_are_rejected() {
        assert!(matches!(
            read(b"not a wav file at all"),
            Err(StegoError::InvalidWav(_))
        ));
    }

    #[test]
    fn eight_bit_wav_is_rejected_because_lsb_math_assumes_16_bit() {
        let mut bytes = write(&Audio {
            sample_rate: 8000,
            samples: vec![1, 2],
        });
        bytes[34] = 8;
        assert!(matches!(read(&bytes), Err(StegoError::UnsupportedWav(_))));
    }
}
