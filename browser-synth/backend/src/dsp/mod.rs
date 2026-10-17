pub mod envelope;
pub mod filter;
pub mod note;
pub mod oscillator;

use std::fmt;

pub const SAMPLE_RATE: u32 = 44_100;

#[derive(Debug, Clone, PartialEq)]
pub enum DspError {
    CutoffOutOfRange(f32),
}

impl fmt::Display for DspError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DspError::CutoffOutOfRange(c) => {
                write!(f, "cutoff {c} Hz must be between 0 and Nyquist")
            }
        }
    }
}

impl std::error::Error for DspError {}
