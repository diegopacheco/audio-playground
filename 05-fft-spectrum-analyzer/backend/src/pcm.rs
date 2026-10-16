use crate::error::DspError;

pub fn decode_f32_le(bytes: &[u8]) -> Result<Vec<f32>, DspError> {
    if !bytes.len().is_multiple_of(4) {
        return Err(DspError::MisalignedPcm(bytes.len()));
    }
    Ok(bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|c| f32::from_le_bytes(*c))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_little_endian_floats_as_sent_by_a_browser_float32array() {
        let values = [0.0f32, 1.0, -0.5, 0.25];
        let bytes: Vec<u8> = values.iter().flat_map(|v| v.to_le_bytes()).collect();
        assert_eq!(decode_f32_le(&bytes).unwrap(), values);
    }

    #[test]
    fn rejects_payloads_that_split_a_float() {
        assert_eq!(decode_f32_le(&[0, 0, 0]), Err(DspError::MisalignedPcm(3)));
    }
}
