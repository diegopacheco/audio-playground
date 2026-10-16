use std::f64::consts::PI;

use super::{Stego, check_capacity, frame};
use crate::dsp::goertzel;
use crate::error::{Result, StegoError};
use crate::wav::Audio;

pub struct UltrasonicStego {
    pub freq_zero: f64,
    pub freq_one: f64,
    pub symbol_seconds: f64,
    pub amplitude: f64,
}

impl Default for UltrasonicStego {
    fn default() -> Self {
        Self {
            freq_zero: 18_200.0,
            freq_one: 19_200.0,
            symbol_seconds: 0.008,
            amplitude: 0.015,
        }
    }
}

impl UltrasonicStego {
    fn symbol_len(&self, sample_rate: u32) -> Result<usize> {
        let nyquist = sample_rate as f64 / 2.0;
        if nyquist < self.freq_one + 500.0 {
            return Err(StegoError::SampleRateTooLow(sample_rate));
        }
        Ok((sample_rate as f64 * self.symbol_seconds).round() as usize)
    }
}

impl Stego for UltrasonicStego {
    fn capacity(&self, carrier: &Audio) -> Result<usize> {
        let symbol = self.symbol_len(carrier.sample_rate)?;
        Ok((carrier.samples.len() / symbol / 8).saturating_sub(frame::OVERHEAD))
    }

    fn encode(&self, carrier: &Audio, payload: &[u8]) -> Result<Audio> {
        check_capacity(payload, self.capacity(carrier)?)?;
        let symbol = self.symbol_len(carrier.sample_rate)?;
        let sr = carrier.sample_rate as f64;
        let peak = self.amplitude * i16::MAX as f64;
        let mut samples = carrier.samples.clone();
        let mut phase = 0.0f64;
        for (i, bit) in frame::to_bits(&frame::build(payload))
            .into_iter()
            .enumerate()
        {
            let freq = if bit { self.freq_one } else { self.freq_zero };
            let step = 2.0 * PI * freq / sr;
            for sample in &mut samples[i * symbol..(i + 1) * symbol] {
                let mixed = *sample as f64 + peak * phase.sin();
                *sample = mixed.round().clamp(i16::MIN as f64, i16::MAX as f64) as i16;
                phase = (phase + step) % (2.0 * PI);
            }
        }
        Ok(Audio {
            sample_rate: carrier.sample_rate,
            samples,
        })
    }

    fn decode(&self, stego: &Audio) -> Result<Vec<u8>> {
        let symbol = self.symbol_len(stego.sample_rate)?;
        let sr = stego.sample_rate as f64;
        let window = goertzel::hann(symbol);
        let bits: Vec<bool> = stego
            .samples
            .chunks_exact(symbol)
            .map(|chunk| {
                let block: Vec<f64> = chunk
                    .iter()
                    .zip(&window)
                    .map(|(&s, w)| s as f64 * w)
                    .collect();
                goertzel::power(&block, self.freq_one, sr)
                    > goertzel::power(&block, self.freq_zero, sr)
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
    fn round_trip_recovers_the_message_through_the_music() {
        let audio = carrier::generate();
        let stego = UltrasonicStego::default();
        let encoded = stego.encode(&audio, b"meet at 18 kHz").unwrap();
        assert_eq!(stego.decode(&encoded).unwrap(), b"meet at 18 kHz");
    }

    #[test]
    fn added_energy_sits_above_17_khz_so_it_stays_near_inaudible() {
        let audio = carrier::generate();
        let encoded = UltrasonicStego::default()
            .encode(&audio, b"quiet please")
            .unwrap();
        let n = 4096;
        let window = goertzel::hann(n);
        let diff: Vec<f64> = encoded.samples[..n]
            .iter()
            .zip(&audio.samples[..n])
            .zip(&window)
            .map(|((a, b), w)| (*a as f64 - *b as f64) * w)
            .collect();
        let sr = audio.sample_rate as f64;
        let (mut high, mut total) = (0.0, 0.0);
        let mut freq = 50.0;
        while freq < sr / 2.0 {
            let p = goertzel::power(&diff, freq, sr);
            total += p;
            if freq > 17_000.0 {
                high += p;
            }
            freq += 25.0;
        }
        assert!(high / total > 0.99, "only {:.4} above 17 kHz", high / total);
    }

    #[test]
    fn message_larger_than_capacity_is_rejected() {
        let audio = carrier::generate();
        let stego = UltrasonicStego::default();
        let cap = stego.capacity(&audio).unwrap();
        let too_big = vec![b'a'; cap + 1];
        assert_eq!(
            stego.encode(&audio, &too_big),
            Err(StegoError::CapacityExceeded {
                needed: cap + 1,
                available: cap
            })
        );
    }

    #[test]
    fn corrupted_symbol_is_caught_by_crc() {
        let audio = carrier::generate();
        let stego = UltrasonicStego::default();
        let payload = b"tamper evident";
        let mut encoded = stego.encode(&audio, payload).unwrap();
        let symbol = stego.symbol_len(audio.sample_rate).unwrap();
        let index = 70;
        let bit = frame::to_bits(&frame::build(payload))[index];
        let other = if bit { stego.freq_zero } else { stego.freq_one };
        let sr = audio.sample_rate as f64;
        for i in 0..symbol {
            let n = index * symbol + i;
            let tone = stego.amplitude * 32767.0 * (2.0 * PI * other * i as f64 / sr).sin();
            encoded.samples[n] = (audio.samples[n] as f64 + tone).round() as i16;
        }
        assert!(matches!(
            stego.decode(&encoded),
            Err(StegoError::CrcMismatch { .. })
        ));
    }

    #[test]
    fn low_sample_rate_cannot_carry_the_band() {
        let audio = Audio {
            sample_rate: 22_050,
            samples: vec![0; 22_050],
        };
        assert_eq!(
            UltrasonicStego::default().capacity(&audio),
            Err(StegoError::SampleRateTooLow(22_050))
        );
    }
}
