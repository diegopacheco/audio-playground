use crate::analysis::{envelope, peak, rms, to_mono};
use crate::decode::decode_all;
use crate::error::WavError;
use crate::render::Renderer;
use crate::riff::{Chunk, FORMAT_FLOAT, Format, parse};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Header {
    pub encoding: String,
    pub format_tag: u16,
    pub sample_rate: u32,
    pub channels: u16,
    pub bits_per_sample: u16,
    pub byte_rate: u32,
    pub block_align: u16,
    pub frames: usize,
}

#[derive(Debug, Serialize)]
pub struct Report {
    pub header: Header,
    pub chunks: Vec<Chunk>,
    pub duration: f64,
    pub peak: f32,
    pub rms: f32,
    pub ascii: String,
    pub envelope: Vec<f32>,
}

fn header(format: &Format, frames: usize) -> Header {
    let kind = if format.format_tag == FORMAT_FLOAT {
        "float"
    } else {
        "PCM"
    };
    Header {
        encoding: format!("{}-bit {kind}", format.bits_per_sample),
        format_tag: format.format_tag,
        sample_rate: format.sample_rate,
        channels: format.channels,
        bits_per_sample: format.bits_per_sample,
        byte_rate: format.byte_rate,
        block_align: format.block_align,
        frames,
    }
}

pub fn inspect(
    bytes: &[u8],
    renderer: &dyn Renderer,
    envelope_points: usize,
) -> Result<Report, WavError> {
    let riff = parse(bytes)?;
    let samples = decode_all(&riff.format, riff.data)?;
    let channels = usize::from(riff.format.channels);
    let mono = to_mono(&samples, channels);
    let frames = mono.len();
    Ok(Report {
        header: header(&riff.format, frames),
        duration: frames as f64 / f64::from(riff.format.sample_rate),
        peak: peak(&samples),
        rms: rms(&samples),
        ascii: renderer.render(&mono),
        envelope: envelope(&mono, envelope_points),
        chunks: riff.chunks,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::WavBuilder;
    use crate::render::AsciiRenderer;

    fn run(bytes: &[u8]) -> Result<Report, WavError> {
        inspect(bytes, &AsciiRenderer::new(32, 8), 64)
    }

    fn tone(n: usize, amp: f32) -> Vec<f32> {
        (0..n).map(|i| amp * (i as f32 * 0.3).sin()).collect()
    }

    #[test]
    fn duration_is_frames_over_sample_rate() {
        let bytes = WavBuilder::pcm16(8000, 1).samples(&tone(4000, 0.5)).build();
        let r = run(&bytes).unwrap();
        assert_eq!(r.header.frames, 4000);
        assert!((r.duration - 0.5).abs() < 1e-9);
    }

    #[test]
    fn stereo_frames_count_pairs_not_samples() {
        let bytes = WavBuilder::pcm16(1000, 2).samples(&tone(2000, 0.5)).build();
        let r = run(&bytes).unwrap();
        assert_eq!(r.header.channels, 2);
        assert_eq!(r.header.frames, 1000);
        assert!((r.duration - 1.0).abs() < 1e-9);
    }

    #[test]
    fn every_supported_encoding_measures_the_same_peak() {
        let s = tone(800, 0.7);
        let builders = vec![
            (WavBuilder::pcm8(8000, 1), 0.02),
            (WavBuilder::pcm16(8000, 1), 1e-3),
            (WavBuilder::pcm24(8000, 1), 1e-4),
            (WavBuilder::pcm32(8000, 1), 1e-4),
            (WavBuilder::float32(8000, 1), 1e-6),
        ];
        let expected = peak(&s);
        for (b, tol) in builders {
            let r = run(&b.samples(&s).build()).unwrap();
            assert!(
                (r.peak - expected).abs() < tol,
                "{} peak {} expected {}",
                r.header.encoding,
                r.peak,
                expected
            );
        }
    }

    #[test]
    fn report_lists_all_chunks_including_unknown_ones() {
        let bytes = WavBuilder::float32(8000, 1)
            .extra_chunk(b"LIST", b"INFO")
            .samples(&tone(10, 0.1))
            .build();
        let r = run(&bytes).unwrap();
        let ids: Vec<&str> = r.chunks.iter().map(|c| c.id.as_str()).collect();
        assert_eq!(ids, vec!["fmt ", "LIST", "data"]);
        assert_eq!(r.header.encoding, "32-bit float");
    }

    #[test]
    fn envelope_and_ascii_respect_requested_sizes() {
        let bytes = WavBuilder::pcm16(8000, 1).samples(&tone(1000, 0.5)).build();
        let r = run(&bytes).unwrap();
        assert_eq!(r.envelope.len(), 64);
        assert_eq!(r.ascii.lines().count(), 8);
    }

    #[test]
    fn empty_data_chunk_yields_zero_duration_not_error() {
        let bytes = WavBuilder::pcm16(8000, 1).samples(&[]).build();
        let r = run(&bytes).unwrap();
        assert_eq!(r.duration, 0.0);
        assert!(r.envelope.is_empty());
    }

    #[test]
    fn malformed_inputs_return_errors() {
        let cases: Vec<Vec<u8>> = vec![
            b"hello".to_vec(),
            b"RIFF\x04\x00\x00\x00WAVX".to_vec(),
            WavBuilder::new(1, 12, 8000, 1).samples(&[0.0]).build(),
            WavBuilder::new(2, 4, 8000, 1).raw_data(&[0, 0]).build(),
            WavBuilder::pcm16(8000, 2).raw_data(&[0, 0, 0]).build(),
            WavBuilder::pcm16(8000, 1).without_data().build(),
        ];
        for bytes in cases {
            assert!(run(&bytes).is_err());
        }
    }
}
