use super::{Demodulator, Modulator};
use crate::dsp::goertzel;
use std::f32::consts::PI;

pub const MARK_HZ: f32 = 1200.0;
pub const SPACE_HZ: f32 = 2200.0;
pub const FOUR_FSK_HZ: [f32; 4] = [1200.0, 1800.0, 2400.0, 3000.0];
const AMPLITUDE: f32 = 0.6;
const RAMP_SECONDS: f32 = 0.002;

pub struct Fsk {
    tones: Vec<f32>,
    sample_rate: u32,
    symbol_len: usize,
    bits_per_symbol: usize,
}

impl Fsk {
    pub fn bfsk(sample_rate: u32, baud: u32) -> Self {
        Self::new(vec![SPACE_HZ, MARK_HZ], sample_rate, baud, 1)
    }

    pub fn four_level(sample_rate: u32, baud: u32) -> Self {
        Self::new(FOUR_FSK_HZ.to_vec(), sample_rate, baud, 2)
    }

    fn new(tones: Vec<f32>, sample_rate: u32, baud: u32, bits_per_symbol: usize) -> Self {
        let symbol_len = ((sample_rate as f32 / baud.max(1) as f32).round() as usize).max(8);
        Self {
            tones,
            sample_rate,
            symbol_len,
            bits_per_symbol,
        }
    }

    pub fn tones(&self) -> &[f32] {
        &self.tones
    }

    fn symbols(&self, bits: &[u8]) -> Vec<usize> {
        bits.chunks(self.bits_per_symbol)
            .map(|group| {
                (0..self.bits_per_symbol).fold(0usize, |acc, i| {
                    (acc << 1) | usize::from(group.get(i).copied().unwrap_or(1) & 1)
                })
            })
            .collect()
    }

    fn apply_ramp(&self, out: &mut [f32]) {
        let ramp = ((self.sample_rate as f32 * RAMP_SECONDS) as usize).min(out.len() / 2);
        let len = out.len();
        for i in 0..ramp {
            let gain = i as f32 / ramp as f32;
            out[i] *= gain;
            out[len - 1 - i] *= gain;
        }
    }
}

impl Modulator for Fsk {
    fn bits_per_symbol(&self) -> usize {
        self.bits_per_symbol
    }

    fn modulate(&self, bits: &[u8]) -> Vec<f32> {
        let symbols = self.symbols(bits);
        let mut out = Vec::with_capacity(symbols.len() * self.symbol_len);
        let mut phase = 0.0f32;
        for symbol in symbols {
            let step = 2.0 * PI * self.tones[symbol] / self.sample_rate as f32;
            for _ in 0..self.symbol_len {
                out.push(phase.sin() * AMPLITUDE);
                phase = (phase + step) % (2.0 * PI);
            }
        }
        self.apply_ramp(&mut out);
        out
    }
}

impl Demodulator for Fsk {
    fn bits_per_symbol(&self) -> usize {
        self.bits_per_symbol
    }

    fn symbol_len(&self) -> usize {
        self.symbol_len
    }

    fn idle_symbol(&self) -> usize {
        self.tones.len() - 1
    }

    fn tone_powers(&self, window: &[f32]) -> Vec<f32> {
        self.tones
            .iter()
            .map(|&f| goertzel::power(window, f, self.sample_rate))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bfsk_maps_bit_one_to_mark_like_bell_202() {
        let fsk = Fsk::bfsk(48000, 300);
        assert_eq!(fsk.tones()[fsk.idle_symbol()], MARK_HZ);
        assert_eq!(fsk.symbols(&[1, 0]), vec![1, 0]);
    }

    #[test]
    fn four_fsk_packs_two_bits_per_symbol_msb_first() {
        let fsk = Fsk::four_level(48000, 300);
        assert_eq!(fsk.symbols(&[0, 0, 0, 1, 1, 0, 1, 1]), vec![0, 1, 2, 3]);
        assert_eq!(fsk.symbols(&[0]), vec![1]);
    }

    #[test]
    fn modulated_signal_has_one_symbol_length_per_symbol() {
        let fsk = Fsk::four_level(48000, 200);
        let audio = fsk.modulate(&[1, 0, 0, 1, 1, 1]);
        assert_eq!(audio.len(), 3 * 240);
    }

    #[test]
    fn every_clean_symbol_is_decided_back_to_the_symbol_that_was_sent() {
        for fsk in [Fsk::bfsk(44100, 300), Fsk::four_level(44100, 300)] {
            let bits: Vec<u8> = (0..40).map(|i| ((i * 7 + 3) % 5 % 2) as u8).collect();
            let sent = fsk.symbols(&bits);
            let audio = fsk.modulate(&bits);
            let len = Demodulator::symbol_len(&fsk);
            for (i, expected) in sent.iter().enumerate() {
                let decision = fsk.decide(&audio[i * len..(i + 1) * len]);
                assert_eq!(decision.symbol, *expected, "symbol {i}");
            }
        }
    }
}
