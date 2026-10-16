pub mod framing;
pub mod fsk;
pub mod receiver;

use crate::error::AppError;
use fsk::Fsk;
use receiver::{Event, Receiver};
use serde::{Deserialize, Serialize};

pub const MIN_BAUD: u32 = 50;
pub const MAX_BAUD: u32 = 600;
pub const MIN_SAMPLE_RATE: u32 = 8000;
pub const MAX_SAMPLE_RATE: u32 = 192_000;
pub const TX_SAMPLE_RATE: u32 = 48000;
const MIN_PREAMBLE_SYMBOLS: u32 = 24;
const PREAMBLE_SECONDS_DIVISOR: u32 = 4;
const SILENCE_SECONDS: f32 = 0.1;

pub trait Modulator {
    fn bits_per_symbol(&self) -> usize;
    fn modulate(&self, bits: &[u8]) -> Vec<f32>;
}

pub struct Decision {
    pub symbol: usize,
    pub signal: f32,
    pub noise: f32,
}

pub trait Demodulator {
    fn bits_per_symbol(&self) -> usize;
    fn symbol_len(&self) -> usize;
    fn idle_symbol(&self) -> usize;
    fn tone_powers(&self, window: &[f32]) -> Vec<f32>;

    fn decide(&self, window: &[f32]) -> Decision {
        let powers = self.tone_powers(window);
        let (symbol, peak) = powers
            .iter()
            .copied()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .unwrap_or((0, 0.0));
        let energy: f32 = window.iter().map(|x| x * x).sum();
        let signal = (2.0 * peak / window.len().max(1) as f32).min(energy);
        let noise = energy - signal;
        Decision {
            symbol,
            signal,
            noise,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub enum Scheme {
    #[serde(rename = "bfsk")]
    Bfsk,
    #[serde(rename = "4fsk")]
    Fsk4,
}

impl Scheme {
    pub const ALL: [Scheme; 2] = [Scheme::Bfsk, Scheme::Fsk4];
}

#[derive(Debug, Clone, Copy)]
pub struct ModemConfig {
    pub scheme: Scheme,
    pub baud: u32,
    pub sample_rate: u32,
}

impl ModemConfig {
    pub fn build(&self) -> Result<Fsk, AppError> {
        if !(MIN_BAUD..=MAX_BAUD).contains(&self.baud) {
            return Err(AppError::InvalidInput(format!(
                "baud must be {MIN_BAUD}..={MAX_BAUD}, got {}",
                self.baud
            )));
        }
        if !(MIN_SAMPLE_RATE..=MAX_SAMPLE_RATE).contains(&self.sample_rate) {
            return Err(AppError::InvalidInput(format!(
                "sample_rate must be {MIN_SAMPLE_RATE}..={MAX_SAMPLE_RATE}, got {}",
                self.sample_rate
            )));
        }
        Ok(match self.scheme {
            Scheme::Bfsk => Fsk::bfsk(self.sample_rate, self.baud),
            Scheme::Fsk4 => Fsk::four_level(self.sample_rate, self.baud),
        })
    }

    fn preamble_bits(&self, bits_per_symbol: usize) -> usize {
        let symbols = MIN_PREAMBLE_SYMBOLS.max(self.baud / PREAMBLE_SECONDS_DIVISOR);
        symbols as usize * bits_per_symbol
    }
}

pub struct Reception {
    pub text: String,
    pub crc_ok: bool,
    pub snr: f32,
}

pub fn transmit_bytes(config: &ModemConfig, body: &[u8]) -> Result<Vec<f32>, AppError> {
    let fsk = config.build()?;
    let bits =
        framing::bits_for_bytes(config.preamble_bits(Modulator::bits_per_symbol(&fsk)), body);
    let silence = vec![0.0f32; (config.sample_rate as f32 * SILENCE_SECONDS) as usize];
    let mut out = silence.clone();
    out.extend(fsk.modulate(&bits));
    out.extend(silence);
    Ok(out)
}

pub fn transmit(config: &ModemConfig, payload: &[u8]) -> Result<Vec<f32>, AppError> {
    transmit_bytes(config, &framing::frame_body(payload)?)
}

pub fn receive(config: &ModemConfig, samples: &[f32]) -> Result<Reception, AppError> {
    let mut receiver = Receiver::new(config.build()?);
    let mut last_problem = String::from("no preamble detected");
    for event in receiver.push(samples) {
        match event {
            Event::Frame { text, crc_ok, snr } => return Ok(Reception { text, crc_ok, snr }),
            Event::Lost { reason } => last_problem = reason,
            Event::Locked { .. } => last_problem = String::from("frame incomplete"),
            Event::Byte { .. } => {}
        }
    }
    Err(AppError::NoFrame(last_problem))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::noise::add_white_noise;

    fn config(scheme: Scheme, baud: u32) -> ModemConfig {
        ModemConfig {
            scheme,
            baud,
            sample_rate: 44100,
        }
    }

    #[test]
    fn round_trip_recovers_text_at_several_bauds_for_both_schemes() {
        let message = "Data over sound: 0123456789 ~!";
        for scheme in Scheme::ALL {
            for baud in [100, 150, 300, 600] {
                let cfg = config(scheme, baud);
                let audio = transmit(&cfg, message.as_bytes()).unwrap();
                let got = receive(&cfg, &audio).unwrap();
                assert_eq!(got.text, message, "{scheme:?} at {baud}");
                assert!(got.crc_ok);
            }
        }
    }

    #[test]
    fn four_fsk_carries_two_bits_per_symbol_so_long_messages_take_about_half_the_time() {
        let message = vec![b'x'; 200];
        let bfsk = transmit(&config(Scheme::Bfsk, 300), &message).unwrap();
        let four = transmit(&config(Scheme::Fsk4, 300), &message).unwrap();
        assert!(
            four.len() * 10 < bfsk.len() * 6,
            "{} vs {}",
            four.len(),
            bfsk.len()
        );
    }

    #[test]
    fn round_trip_survives_white_noise_at_moderate_snr() {
        for scheme in Scheme::ALL {
            let cfg = config(scheme, 300);
            let audio = transmit(&cfg, b"noisy channel works").unwrap();
            let noisy = add_white_noise(&audio, 6.0, 42);
            let got = receive(&cfg, &noisy).unwrap();
            assert_eq!(got.text, "noisy channel works", "{scheme:?}");
            assert!(got.crc_ok);
            assert!(got.snr > 3.0, "snr {}", got.snr);
        }
    }

    #[test]
    fn snr_estimate_drops_when_more_noise_is_added() {
        let cfg = config(Scheme::Bfsk, 300);
        let audio = transmit(&cfg, b"measure me").unwrap();
        let clean = receive(&cfg, &add_white_noise(&audio, 20.0, 7)).unwrap();
        let dirty = receive(&cfg, &add_white_noise(&audio, 5.0, 7)).unwrap();
        assert!(
            clean.snr > dirty.snr + 5.0,
            "{} vs {}",
            clean.snr,
            dirty.snr
        );
    }

    #[test]
    fn round_trip_with_leading_silence_at_non_symbol_aligned_offset() {
        for scheme in Scheme::ALL {
            for offset in [1, 37, 12_345, 50_001] {
                let cfg = config(scheme, 150);
                let mut audio = vec![0.0f32; offset];
                audio.extend(transmit(&cfg, b"offset ok").unwrap());
                let noisy = add_white_noise(&audio, 12.0, offset as u64);
                let got = receive(&cfg, &noisy).unwrap();
                assert_eq!(got.text, "offset ok", "{scheme:?} offset {offset}");
            }
        }
    }

    #[test]
    fn crc_detects_corruption_introduced_before_modulation() {
        let cfg = config(Scheme::Bfsk, 300);
        let mut body = framing::frame_body(b"integrity").unwrap();
        body[5] ^= 0x01;
        let audio = transmit_bytes(&cfg, &body).unwrap();
        let got = receive(&cfg, &audio).unwrap();
        assert!(!got.crc_ok);
        assert_ne!(got.text, "integrity");
    }

    #[test]
    fn streaming_in_small_chunks_emits_bytes_live_and_same_frame() {
        let cfg = config(Scheme::Fsk4, 300);
        let audio = transmit(&cfg, b"live").unwrap();
        let mut receiver = Receiver::new(cfg.build().unwrap());
        let mut events = Vec::new();
        for chunk in audio.chunks(512) {
            events.extend(receiver.push(chunk));
        }
        let bytes: Vec<u8> = events
            .iter()
            .filter_map(|e| match e {
                Event::Byte { value, .. } => Some(*value),
                _ => None,
            })
            .collect();
        assert_eq!(&bytes[..2], &framing::SYNC);
        assert_eq!(&bytes[4..8], b"live");
        assert!(matches!(events.first(), Some(Event::Locked { .. })));
        assert!(
            matches!(events.last(), Some(Event::Frame { text, crc_ok: true, .. }) if text == "live")
        );
    }

    #[test]
    fn receiver_decodes_two_consecutive_frames_in_one_stream() {
        let cfg = config(Scheme::Bfsk, 300);
        let mut audio = transmit(&cfg, b"first").unwrap();
        audio.extend(transmit(&cfg, b"second").unwrap());
        let mut receiver = Receiver::new(cfg.build().unwrap());
        let frames: Vec<String> = receiver
            .push(&audio)
            .into_iter()
            .filter_map(|e| match e {
                Event::Frame { text, .. } => Some(text),
                _ => None,
            })
            .collect();
        assert_eq!(frames, vec!["first".to_string(), "second".to_string()]);
    }

    #[test]
    fn pure_noise_never_produces_a_frame() {
        let cfg = config(Scheme::Bfsk, 300);
        let noise = add_white_noise(&[1.0f32; 44100], 0.0, 9)
            .iter()
            .map(|x| x - 1.0)
            .collect::<Vec<f32>>();
        assert!(receive(&cfg, &noise).is_err());
    }

    #[test]
    fn rejects_baud_outside_supported_range() {
        assert!(config(Scheme::Bfsk, 10).build().is_err());
        assert!(config(Scheme::Bfsk, 5000).build().is_err());
    }
}
