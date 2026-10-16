use crate::error::WavError;
use serde::Serialize;

pub const FORMAT_PCM: u16 = 1;
pub const FORMAT_FLOAT: u16 = 3;
const FORMAT_EXTENSIBLE: u16 = 0xFFFE;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Chunk {
    pub id: String,
    pub offset: usize,
    pub size: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Format {
    pub format_tag: u16,
    pub channels: u16,
    pub sample_rate: u32,
    pub byte_rate: u32,
    pub block_align: u16,
    pub bits_per_sample: u16,
}

pub struct Riff<'a> {
    pub chunks: Vec<Chunk>,
    pub format: Format,
    pub data: &'a [u8],
}

fn u16_at(b: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([b[at], b[at + 1]])
}

fn u32_at(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}

pub fn parse_chunks(bytes: &[u8]) -> Result<Vec<Chunk>, WavError> {
    if bytes.len() < 12 {
        return Err(WavError::TooShort(bytes.len()));
    }
    if &bytes[0..4] != b"RIFF" {
        return Err(WavError::NotRiff);
    }
    if &bytes[8..12] != b"WAVE" {
        return Err(WavError::NotWave);
    }
    let mut chunks = Vec::new();
    let mut pos = 12;
    while pos + 8 <= bytes.len() {
        let id = String::from_utf8_lossy(&bytes[pos..pos + 4]).into_owned();
        let size = u32_at(bytes, pos + 4);
        let body = pos + 8;
        let available = bytes.len() - body;
        if size as usize > available {
            return Err(WavError::ChunkOverrun {
                id,
                declared: size,
                available,
            });
        }
        chunks.push(Chunk {
            id,
            offset: pos,
            size,
        });
        pos = body + size as usize + (size as usize & 1);
    }
    Ok(chunks)
}

fn body<'a>(bytes: &'a [u8], chunk: &Chunk) -> &'a [u8] {
    let start = chunk.offset + 8;
    &bytes[start..start + chunk.size as usize]
}

pub fn parse_format(fmt: &[u8]) -> Result<Format, WavError> {
    if fmt.len() < 16 {
        return Err(WavError::InvalidFmt(format!(
            "size {} is smaller than 16",
            fmt.len()
        )));
    }
    let raw_tag = u16_at(fmt, 0);
    let format_tag = if raw_tag == FORMAT_EXTENSIBLE {
        if fmt.len() < 26 {
            return Err(WavError::InvalidFmt(
                "extensible fmt is missing its sub format".into(),
            ));
        }
        u16_at(fmt, 24)
    } else {
        raw_tag
    };
    let format = Format {
        format_tag,
        channels: u16_at(fmt, 2),
        sample_rate: u32_at(fmt, 4),
        byte_rate: u32_at(fmt, 8),
        block_align: u16_at(fmt, 12),
        bits_per_sample: u16_at(fmt, 14),
    };
    if format.sample_rate == 0 {
        return Err(WavError::InvalidFmt("sample rate is zero".into()));
    }
    if format.channels == 0 {
        return Err(WavError::InvalidFmt("channel count is zero".into()));
    }
    let expected = u32::from(format.channels) * u32::from(format.bits_per_sample).div_ceil(8);
    if u32::from(format.block_align) != expected {
        return Err(WavError::InvalidFmt(format!(
            "block align {} does not match channels * bytes per sample {}",
            format.block_align, expected
        )));
    }
    Ok(format)
}

pub fn parse(bytes: &[u8]) -> Result<Riff<'_>, WavError> {
    let chunks = parse_chunks(bytes)?;
    let fmt = chunks
        .iter()
        .find(|c| c.id == "fmt ")
        .ok_or(WavError::MissingFmt)?;
    let format = parse_format(body(bytes, fmt))?;
    let data_chunk = chunks
        .iter()
        .find(|c| c.id == "data")
        .ok_or(WavError::MissingData)?;
    let data = body(bytes, data_chunk);
    Ok(Riff {
        chunks,
        format,
        data,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::WavBuilder;

    #[test]
    fn unknown_chunks_are_listed_and_skipped_so_data_is_still_found() {
        let bytes = WavBuilder::pcm16(8000, 1)
            .extra_chunk(b"LIST", b"INFOISFT\x05\x00\x00\x00rust\x00")
            .samples(&[0.5, -0.5])
            .build();
        let riff = parse(&bytes).unwrap();
        let ids: Vec<&str> = riff.chunks.iter().map(|c| c.id.as_str()).collect();
        assert_eq!(ids, vec!["fmt ", "LIST", "data"]);
        assert_eq!(riff.data.len(), 4);
    }

    #[test]
    fn odd_sized_chunk_is_followed_by_pad_byte() {
        let bytes = WavBuilder::pcm16(8000, 1)
            .extra_chunk(b"junk", b"abc")
            .samples(&[0.1])
            .build();
        let riff = parse(&bytes).unwrap();
        let junk = &riff.chunks[1];
        let data = &riff.chunks[2];
        assert_eq!(junk.size, 3);
        assert_eq!(data.offset, junk.offset + 8 + 4);
        assert_eq!(data.id, "data");
    }

    #[test]
    fn data_before_fmt_is_accepted() {
        let bytes = WavBuilder::pcm16(8000, 1)
            .data_first()
            .samples(&[0.25])
            .build();
        let riff = parse(&bytes).unwrap();
        assert_eq!(riff.chunks[0].id, "data");
        assert_eq!(riff.format.sample_rate, 8000);
    }

    #[test]
    fn extensible_format_resolves_to_sub_format() {
        let bytes = WavBuilder::pcm24(44100, 2)
            .extensible()
            .samples(&[0.0, 0.0])
            .build();
        let riff = parse(&bytes).unwrap();
        assert_eq!(riff.format.format_tag, FORMAT_PCM);
        assert_eq!(riff.format.bits_per_sample, 24);
    }

    #[test]
    fn empty_input_is_too_short() {
        assert_eq!(parse(&[]).err(), Some(WavError::TooShort(0)));
    }

    #[test]
    fn non_riff_signature_is_rejected() {
        let mut bytes = WavBuilder::pcm16(8000, 1).samples(&[0.0]).build();
        bytes[0..4].copy_from_slice(b"RIFX");
        assert_eq!(parse(&bytes).err(), Some(WavError::NotRiff));
    }

    #[test]
    fn non_wave_form_is_rejected() {
        let mut bytes = WavBuilder::pcm16(8000, 1).samples(&[0.0]).build();
        bytes[8..12].copy_from_slice(b"AVI ");
        assert_eq!(parse(&bytes).err(), Some(WavError::NotWave));
    }

    #[test]
    fn truncated_data_chunk_is_reported_not_read_past_end() {
        let mut bytes = WavBuilder::pcm16(8000, 1).samples(&[0.1; 100]).build();
        bytes.truncate(bytes.len() - 10);
        match parse(&bytes) {
            Err(WavError::ChunkOverrun { id, declared, .. }) => {
                assert_eq!(id, "data");
                assert_eq!(declared, 200);
            }
            other => panic!("expected overrun, got {:?}", other.err()),
        }
    }

    #[test]
    fn missing_fmt_is_rejected() {
        let bytes = WavBuilder::pcm16(8000, 1)
            .without_fmt()
            .samples(&[0.0])
            .build();
        assert_eq!(parse(&bytes).err(), Some(WavError::MissingFmt));
    }

    #[test]
    fn missing_data_is_rejected() {
        let bytes = WavBuilder::pcm16(8000, 1).without_data().build();
        assert_eq!(parse(&bytes).err(), Some(WavError::MissingData));
    }

    #[test]
    fn short_fmt_chunk_is_rejected() {
        let mut bytes = b"RIFF\x00\x00\x00\x00WAVEfmt \x04\x00\x00\x00\x01\x00\x01\x00".to_vec();
        bytes.extend_from_slice(b"data\x00\x00\x00\x00");
        assert!(matches!(parse(&bytes), Err(WavError::InvalidFmt(_))));
    }

    #[test]
    fn inconsistent_block_align_is_rejected() {
        let mut bytes = WavBuilder::pcm16(8000, 2).samples(&[0.0, 0.0]).build();
        bytes[32] = 3;
        assert!(matches!(parse(&bytes), Err(WavError::InvalidFmt(_))));
    }

    #[test]
    fn zero_sample_rate_is_rejected() {
        let mut bytes = WavBuilder::pcm16(8000, 1).samples(&[0.0]).build();
        bytes[24..28].copy_from_slice(&0u32.to_le_bytes());
        assert!(matches!(parse(&bytes), Err(WavError::InvalidFmt(_))));
    }
}
