use super::Demodulator;
use super::framing::{self, DecodedByte, Status};
use serde::Serialize;

const STEPS_PER_SYMBOL: usize = 8;
const MIN_PREAMBLE_SYMBOLS: usize = 6;
const CONCENTRATION_THRESHOLD: f32 = 0.5;
const ENERGY_FLOOR: f32 = 1e-6;
const LEAD_SYMBOLS: usize = 2;
const SYNC_SEARCH_SYMBOLS: usize = 16;

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    Locked {
        sample: usize,
    },
    Byte {
        value: u8,
        bits: String,
        framing_ok: bool,
    },
    Frame {
        text: String,
        crc_ok: bool,
        snr: f32,
    },
    Lost {
        reason: String,
    },
}

enum Search {
    Found(usize),
    NeedMore(usize),
}

struct Lock {
    pos: usize,
    bits: Vec<u8>,
    emitted: usize,
    signal: f64,
    noise: f64,
}

enum State {
    Searching,
    Locked(Lock),
}

pub struct Receiver<D: Demodulator> {
    demod: D,
    buf: Vec<f32>,
    consumed: usize,
    state: State,
}

impl<D: Demodulator> Receiver<D> {
    pub fn new(demod: D) -> Self {
        Self {
            demod,
            buf: Vec::new(),
            consumed: 0,
            state: State::Searching,
        }
    }

    pub fn push(&mut self, samples: &[f32]) -> Vec<Event> {
        self.buf.extend_from_slice(samples);
        let mut events = Vec::new();
        loop {
            match std::mem::replace(&mut self.state, State::Searching) {
                State::Searching => match self.find_lock() {
                    Search::Found(boundary) => {
                        events.push(Event::Locked {
                            sample: self.consumed + boundary,
                        });
                        let lead = LEAD_SYMBOLS * self.demod.symbol_len();
                        self.state = State::Locked(Lock {
                            pos: boundary.saturating_sub(lead),
                            bits: Vec::new(),
                            emitted: 0,
                            signal: 0.0,
                            noise: 0.0,
                        });
                    }
                    Search::NeedMore(keep_from) => {
                        self.drain(keep_from);
                        return events;
                    }
                },
                State::Locked(lock) => match self.advance(lock, &mut events) {
                    Some(lock) => {
                        self.state = State::Locked(lock);
                        return events;
                    }
                    None => continue,
                },
            }
        }
    }

    fn drain(&mut self, count: usize) {
        let count = count.min(self.buf.len());
        self.buf.drain(..count);
        self.consumed += count;
    }

    fn find_lock(&self) -> Search {
        let sym = self.demod.symbol_len();
        let step = (sym / STEPS_PER_SYMBOL).max(1);
        let idle = self.demod.idle_symbol();
        let min_run = MIN_PREAMBLE_SYMBOLS * STEPS_PER_SYMBOL;
        let mut run = 0;
        let mut run_start = 0;
        let mut p = 0;
        while p + sym <= self.buf.len() {
            let window = &self.buf[p..p + sym];
            let powers = self.demod.tone_powers(window);
            if run >= min_run {
                let total: f32 = powers.iter().sum();
                if powers[idle] < 0.5 * total {
                    return Search::Found(p + sym / 2 - step / 2);
                }
            } else {
                let energy: f32 = window.iter().map(|x| x * x).sum();
                let concentration = powers[idle] / (energy * sym as f32 / 2.0 + f32::EPSILON);
                if energy > ENERGY_FLOOR * sym as f32 && concentration > CONCENTRATION_THRESHOLD {
                    if run == 0 {
                        run_start = p;
                    }
                    run += 1;
                } else {
                    run = 0;
                }
            }
            p += step;
        }
        Search::NeedMore(if run > 0 { run_start } else { p })
    }

    fn advance(&mut self, mut lock: Lock, events: &mut Vec<Event>) -> Option<Lock> {
        let sym = self.demod.symbol_len();
        let bps = self.demod.bits_per_symbol();
        while lock.pos + sym <= self.buf.len() {
            let decision = self.demod.decide(&self.buf[lock.pos..lock.pos + sym]);
            for i in (0..bps).rev() {
                lock.bits.push(((decision.symbol >> i) & 1) as u8);
            }
            lock.signal += f64::from(decision.signal);
            lock.noise += f64::from(decision.noise);
            lock.pos += sym;
        }
        let limit = (LEAD_SYMBOLS + SYNC_SEARCH_SYMBOLS) * bps;
        let parsed = framing::parse(&lock.bits, limit);
        for byte in parsed.bytes.iter().skip(lock.emitted) {
            events.push(byte_event(byte));
        }
        lock.emitted = lock.emitted.max(parsed.bytes.len());
        let decoded_symbols = lock.bits.len() / bps;
        let resume_symbols = parsed.resume.div_ceil(bps).min(decoded_symbols);
        let resume = lock
            .pos
            .saturating_sub((decoded_symbols - resume_symbols) * sym);
        let pos = lock.pos;
        match parsed.status {
            Status::NeedMore => {
                self.drain(pos);
                lock.pos = 0;
                Some(lock)
            }
            Status::Complete(frame) => {
                let snr = 10.0 * ((lock.signal + 1e-12) / (lock.noise + 1e-12)).log10();
                events.push(Event::Frame {
                    text: String::from_utf8_lossy(&frame.payload).into_owned(),
                    crc_ok: frame.crc_ok,
                    snr: ((snr * 10.0).round() / 10.0) as f32,
                });
                self.drain(resume);
                None
            }
            Status::NoSync => {
                events.push(Event::Lost {
                    reason: "sync word not found after preamble".into(),
                });
                self.drain(resume);
                None
            }
            Status::TooLong(len) => {
                events.push(Event::Lost {
                    reason: format!("invalid frame length {len}"),
                });
                self.drain(resume);
                None
            }
        }
    }
}

fn byte_event(byte: &DecodedByte) -> Event {
    Event::Byte {
        value: byte.value,
        bits: byte.bits.iter().map(|b| char::from(b'0' + b)).collect(),
        framing_ok: byte.framing_ok,
    }
}
