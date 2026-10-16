use super::{Stego, check_capacity, frame};
use crate::error::{Result, StegoError};
use crate::wav::Audio;

pub struct LsbStego {
    bits: u8,
}

impl LsbStego {
    pub fn new(bits: u8) -> Result<Self> {
        if !(1..=4).contains(&bits) {
            return Err(StegoError::InvalidBits(bits));
        }
        Ok(Self { bits })
    }

    fn mask(&self) -> u16 {
        (1u16 << self.bits) - 1
    }

    fn raw_bytes(&self, samples: usize) -> usize {
        samples * self.bits as usize / 8
    }
}

impl Stego for LsbStego {
    fn capacity(&self, carrier: &Audio) -> Result<usize> {
        Ok(self
            .raw_bytes(carrier.samples.len())
            .saturating_sub(frame::OVERHEAD))
    }

    fn encode(&self, carrier: &Audio, payload: &[u8]) -> Result<Audio> {
        check_capacity(payload, self.capacity(carrier)?)?;
        let bits = frame::to_bits(&frame::build(payload));
        let mask = self.mask();
        let mut samples = carrier.samples.clone();
        for (sample, chunk) in samples.iter_mut().zip(bits.chunks(self.bits as usize)) {
            let value = chunk
                .iter()
                .chain(std::iter::repeat(&false))
                .take(self.bits as usize)
                .fold(0u16, |acc, &b| (acc << 1) | b as u16);
            *sample = (((*sample as u16) & !mask) | value) as i16;
        }
        Ok(Audio {
            sample_rate: carrier.sample_rate,
            samples,
        })
    }

    fn decode(&self, stego: &Audio) -> Result<Vec<u8>> {
        let mask = self.mask();
        let bits: Vec<bool> = stego
            .samples
            .iter()
            .flat_map(|&s| {
                let v = (s as u16) & mask;
                (0..self.bits).rev().map(move |i| (v >> i) & 1 == 1)
            })
            .collect();
        frame::parse(&frame::from_bits(&bits))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::carrier;

    #[test]
    fn round_trip_recovers_the_message_for_every_bit_depth() {
        let audio = carrier::generate();
        for bits in 1..=4 {
            let stego = LsbStego::new(bits).unwrap();
            let encoded = stego
                .encode(&audio, "hidden in plain sound".as_bytes())
                .unwrap();
            assert_eq!(stego.decode(&encoded).unwrap(), b"hidden in plain sound");
        }
    }

    #[test]
    fn each_sample_moves_at_most_two_pow_bits_minus_one_so_noise_stays_bounded() {
        let audio = carrier::generate();
        let payload = vec![0xA5u8; 2000];
        for bits in 1..=4u8 {
            let stego = LsbStego::new(bits).unwrap();
            let encoded = stego.encode(&audio, &payload).unwrap();
            let limit = (1i32 << bits) - 1;
            let max = audio
                .samples
                .iter()
                .zip(&encoded.samples)
                .map(|(a, b)| (*a as i32 - *b as i32).abs())
                .max()
                .unwrap();
            assert!(max <= limit, "bits {bits} moved a sample by {max}");
            assert!(max > 0, "bits {bits} changed nothing");
        }
    }

    #[test]
    fn message_larger_than_capacity_is_rejected() {
        let audio = Audio {
            sample_rate: 44_100,
            samples: vec![0; 800],
        };
        let stego = LsbStego::new(1).unwrap();
        assert_eq!(stego.capacity(&audio).unwrap(), 88);
        assert_eq!(
            stego.encode(&audio, &[1u8; 89]),
            Err(StegoError::CapacityExceeded {
                needed: 89,
                available: 88
            })
        );
        assert!(stego.encode(&audio, &[1u8; 88]).is_ok());
    }

    #[test]
    fn flipped_payload_bit_is_caught_by_crc() {
        let audio = carrier::generate();
        let stego = LsbStego::new(1).unwrap();
        let mut encoded = stego.encode(&audio, b"integrity matters").unwrap();
        encoded.samples[70] ^= 1;
        assert!(matches!(
            stego.decode(&encoded),
            Err(StegoError::CrcMismatch { .. })
        ));
    }

    #[test]
    fn decoding_with_wrong_bit_depth_finds_no_payload() {
        let audio = carrier::generate();
        let encoded = LsbStego::new(2).unwrap().encode(&audio, b"x").unwrap();
        assert_eq!(
            LsbStego::new(1).unwrap().decode(&encoded),
            Err(StegoError::NoPayload)
        );
    }

    #[test]
    fn bit_depth_outside_one_to_four_is_rejected() {
        assert!(LsbStego::new(0).is_err());
        assert!(LsbStego::new(5).is_err());
    }
}
