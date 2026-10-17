use crate::error::WavError;
use crate::riff::{FORMAT_FLOAT, FORMAT_PCM, Format};

pub trait SampleDecoder {
    fn bytes_per_sample(&self) -> usize;
    fn decode(&self, bytes: &[u8]) -> f32;
}

pub struct Pcm8;
pub struct Pcm16;
pub struct Pcm24;
pub struct Pcm32;
pub struct Float32;

impl SampleDecoder for Pcm8 {
    fn bytes_per_sample(&self) -> usize {
        1
    }
    fn decode(&self, b: &[u8]) -> f32 {
        (f32::from(b[0]) - 128.0) / 128.0
    }
}

impl SampleDecoder for Pcm16 {
    fn bytes_per_sample(&self) -> usize {
        2
    }
    fn decode(&self, b: &[u8]) -> f32 {
        f32::from(i16::from_le_bytes([b[0], b[1]])) / 32768.0
    }
}

impl SampleDecoder for Pcm24 {
    fn bytes_per_sample(&self) -> usize {
        3
    }
    fn decode(&self, b: &[u8]) -> f32 {
        let v = i32::from_le_bytes([0, b[0], b[1], b[2]]) >> 8;
        v as f32 / 8_388_608.0
    }
}

impl SampleDecoder for Pcm32 {
    fn bytes_per_sample(&self) -> usize {
        4
    }
    fn decode(&self, b: &[u8]) -> f32 {
        (f64::from(i32::from_le_bytes([b[0], b[1], b[2], b[3]])) / 2_147_483_648.0) as f32
    }
}

impl SampleDecoder for Float32 {
    fn bytes_per_sample(&self) -> usize {
        4
    }
    fn decode(&self, b: &[u8]) -> f32 {
        f32::from_le_bytes([b[0], b[1], b[2], b[3]])
    }
}

pub fn decoder_for(format: &Format) -> Result<Box<dyn SampleDecoder>, WavError> {
    match (format.format_tag, format.bits_per_sample) {
        (FORMAT_PCM, 8) => Ok(Box::new(Pcm8)),
        (FORMAT_PCM, 16) => Ok(Box::new(Pcm16)),
        (FORMAT_PCM, 24) => Ok(Box::new(Pcm24)),
        (FORMAT_PCM, 32) => Ok(Box::new(Pcm32)),
        (FORMAT_FLOAT, 32) => Ok(Box::new(Float32)),
        (FORMAT_PCM | FORMAT_FLOAT, bits) => Err(WavError::UnsupportedBitDepth {
            format: format.format_tag,
            bits,
        }),
        (tag, _) => Err(WavError::UnsupportedFormat(tag)),
    }
}

pub fn decode_all(format: &Format, data: &[u8]) -> Result<Vec<f32>, WavError> {
    if format.channels > 2 {
        return Err(WavError::UnsupportedChannels(format.channels));
    }
    let decoder = decoder_for(format)?;
    if !data.len().is_multiple_of(usize::from(format.block_align)) {
        return Err(WavError::MisalignedData {
            size: data.len(),
            block_align: format.block_align,
        });
    }
    Ok(data
        .chunks_exact(decoder.bytes_per_sample())
        .map(|b| decoder.decode(b))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn format(tag: u16, bits: u16, channels: u16) -> Format {
        let block_align = channels * bits.div_ceil(8);
        Format {
            format_tag: tag,
            channels,
            sample_rate: 8000,
            byte_rate: 8000 * u32::from(block_align),
            block_align,
            bits_per_sample: bits,
        }
    }

    #[test]
    fn pcm8_is_unsigned_with_128_as_silence() {
        assert_eq!(Pcm8.decode(&[128]), 0.0);
        assert_eq!(Pcm8.decode(&[0]), -1.0);
        assert!(Pcm8.decode(&[255]) > 0.99);
    }

    #[test]
    fn pcm16_full_negative_scale_maps_to_minus_one() {
        assert_eq!(Pcm16.decode(&i16::MIN.to_le_bytes()), -1.0);
        assert!((Pcm16.decode(&i16::MAX.to_le_bytes()) - 1.0).abs() < 1e-4);
    }

    #[test]
    fn pcm24_sign_extends_negative_values() {
        assert_eq!(Pcm24.decode(&[0x00, 0x00, 0x80]), -1.0);
        assert!((Pcm24.decode(&[0xFF, 0xFF, 0xFF]) + 1.0 / 8_388_608.0).abs() < 1e-9);
        assert!((Pcm24.decode(&[0x00, 0x00, 0x40]) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn pcm32_half_scale_maps_to_half() {
        assert!((Pcm32.decode(&(1i32 << 30).to_le_bytes()) - 0.5).abs() < 1e-6);
        assert_eq!(Pcm32.decode(&i32::MIN.to_le_bytes()), -1.0);
    }

    #[test]
    fn float32_is_passed_through_unscaled() {
        assert_eq!(Float32.decode(&(-0.25f32).to_le_bytes()), -0.25);
    }

    #[test]
    fn unsupported_pcm_bit_depth_is_rejected() {
        let result = decoder_for(&format(FORMAT_PCM, 12, 1));
        assert_eq!(
            result.err(),
            Some(WavError::UnsupportedBitDepth {
                format: FORMAT_PCM,
                bits: 12
            })
        );
    }

    #[test]
    fn float64_is_rejected() {
        assert!(matches!(
            decoder_for(&format(FORMAT_FLOAT, 64, 1)),
            Err(WavError::UnsupportedBitDepth { bits: 64, .. })
        ));
    }

    #[test]
    fn compressed_formats_are_rejected() {
        assert!(matches!(
            decoder_for(&format(0x55, 16, 1)),
            Err(WavError::UnsupportedFormat(0x55))
        ));
    }

    #[test]
    fn more_than_two_channels_is_rejected() {
        let f = format(FORMAT_PCM, 16, 6);
        assert_eq!(
            decode_all(&f, &[0; 12]).err(),
            Some(WavError::UnsupportedChannels(6))
        );
    }

    #[test]
    fn partial_frame_is_rejected() {
        let f = format(FORMAT_PCM, 16, 2);
        assert!(matches!(
            decode_all(&f, &[0; 6]),
            Err(WavError::MisalignedData { size: 6, .. })
        ));
    }

    #[test]
    fn decode_all_keeps_interleaved_order() {
        let f = format(FORMAT_PCM, 16, 2);
        let mut data = Vec::new();
        data.extend_from_slice(&16384i16.to_le_bytes());
        data.extend_from_slice(&(-16384i16).to_le_bytes());
        assert_eq!(decode_all(&f, &data).unwrap(), vec![0.5, -0.5]);
    }
}
