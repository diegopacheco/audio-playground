use crate::error::AppError;

#[derive(Debug, Clone, PartialEq)]
pub struct Audio {
    pub sample_rate: u32,
    pub channels: Vec<Vec<f32>>,
}

impl Audio {
    pub fn frames(&self) -> usize {
        self.channels.first().map_or(0, Vec::len)
    }
}

pub fn is_wav(bytes: &[u8]) -> bool {
    bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WAVE"
}

fn u16_at(b: &[u8], i: usize) -> u16 {
    u16::from_le_bytes([b[i], b[i + 1]])
}

fn u32_at(b: &[u8], i: usize) -> u32 {
    u32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]])
}

struct Format {
    tag: u16,
    channels: usize,
    sample_rate: u32,
    bits: u16,
}

pub fn parse(bytes: &[u8]) -> Result<Audio, AppError> {
    if !is_wav(bytes) {
        return Err(AppError::InvalidWav("missing RIFF/WAVE header".into()));
    }
    let mut pos = 12;
    let mut format: Option<Format> = None;
    while pos + 8 <= bytes.len() {
        let id = &bytes[pos..pos + 4];
        let size = u32_at(bytes, pos + 4) as usize;
        let start = pos + 8;
        let end = (start + size).min(bytes.len());
        if id == b"fmt " {
            if end - start < 16 {
                return Err(AppError::InvalidWav("fmt chunk too short".into()));
            }
            let mut tag = u16_at(bytes, start);
            if tag == 0xFFFE && end - start >= 26 {
                tag = u16_at(bytes, start + 24);
            }
            format = Some(Format {
                tag,
                channels: u16_at(bytes, start + 2) as usize,
                sample_rate: u32_at(bytes, start + 4),
                bits: u16_at(bytes, start + 14),
            });
        } else if id == b"data" {
            let fmt = format.ok_or_else(|| AppError::InvalidWav("data before fmt".into()))?;
            return decode(&fmt, &bytes[start..end]);
        }
        pos = start + size + (size & 1);
    }
    Err(AppError::InvalidWav("no data chunk".into()))
}

fn decode(fmt: &Format, data: &[u8]) -> Result<Audio, AppError> {
    if fmt.channels == 0 || fmt.sample_rate == 0 {
        return Err(AppError::InvalidWav("zero channels or sample rate".into()));
    }
    let width = (fmt.bits / 8) as usize;
    let read: fn(&[u8]) -> f32 = match (fmt.tag, fmt.bits) {
        (1, 8) => |s| (s[0] as f32 - 128.0) / 128.0,
        (1, 16) => |s| i16::from_le_bytes([s[0], s[1]]) as f32 / 32768.0,
        (1, 24) => |s| (i32::from_le_bytes([0, s[0], s[1], s[2]]) >> 8) as f32 / 8_388_608.0,
        (1, 32) => |s| i32::from_le_bytes([s[0], s[1], s[2], s[3]]) as f32 / 2_147_483_648.0,
        (3, 32) => |s| f32::from_le_bytes([s[0], s[1], s[2], s[3]]),
        (tag, bits) => {
            return Err(AppError::InvalidWav(format!(
                "unsupported format tag {tag} with {bits} bits"
            )));
        }
    };
    let frame = width * fmt.channels;
    let frames = data.len() / frame;
    let mut channels = vec![Vec::with_capacity(frames); fmt.channels];
    for f in 0..frames {
        for (c, channel) in channels.iter_mut().enumerate() {
            let at = f * frame + c * width;
            channel.push(read(&data[at..at + width]));
        }
    }
    Ok(Audio {
        sample_rate: fmt.sample_rate,
        channels,
    })
}

pub fn write_mono_16bit(samples: &[f32], sample_rate: u32) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let mut out = Vec::with_capacity(44 + samples.len() * 2);
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
        let v = (s.clamp(-1.0, 1.0) * 32767.0).round() as i16;
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_matches_whisper_input_contract_16k_mono_16bit() {
        let wav = write_mono_16bit(&[0.0; 160], 16_000);
        assert_eq!(wav.len(), 44 + 320);
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(u32_at(&wav, 4), 36 + 320);
        assert_eq!(u16_at(&wav, 20), 1);
        assert_eq!(u16_at(&wav, 22), 1);
        assert_eq!(u32_at(&wav, 24), 16_000);
        assert_eq!(u32_at(&wav, 28), 32_000);
        assert_eq!(u16_at(&wav, 32), 2);
        assert_eq!(u16_at(&wav, 34), 16);
        assert_eq!(u32_at(&wav, 40), 320);
    }

    #[test]
    fn written_samples_survive_a_round_trip_within_one_lsb() {
        let input = [0.0, 0.5, -0.5, 0.999, -1.0, 0.25];
        let audio = parse(&write_mono_16bit(&input, 16_000)).unwrap();
        assert_eq!(audio.sample_rate, 16_000);
        assert_eq!(audio.channels.len(), 1);
        for (a, b) in input.iter().zip(&audio.channels[0]) {
            assert!((a - b).abs() < 1.0 / 16_000.0, "{a} vs {b}");
        }
    }

    #[test]
    fn out_of_range_samples_are_clipped_not_wrapped() {
        let audio = parse(&write_mono_16bit(&[2.0, -2.0], 16_000)).unwrap();
        assert!(audio.channels[0][0] > 0.99);
        assert!(audio.channels[0][1] < -0.99);
    }

    #[test]
    fn stereo_16bit_is_split_into_channels() {
        let mut wav = write_mono_16bit(&[0.5, -0.5, 0.25, -0.25], 44_100);
        wav[22] = 2;
        wav[32] = 4;
        let audio = parse(&wav).unwrap();
        assert_eq!(audio.channels.len(), 2);
        assert_eq!(audio.frames(), 2);
        assert!(audio.channels[0].iter().all(|s| *s > 0.0));
        assert!(audio.channels[1].iter().all(|s| *s < 0.0));
    }

    #[test]
    fn non_wav_bytes_are_rejected() {
        assert!(!is_wav(b"\x1aE\xdf\xa3webm"));
        assert!(parse(b"not a wav file at all").is_err());
    }
}
