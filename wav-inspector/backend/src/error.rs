use std::fmt;

#[derive(Debug, PartialEq)]
pub enum WavError {
    TooShort(usize),
    NotRiff,
    NotWave,
    ChunkOverrun {
        id: String,
        declared: u32,
        available: usize,
    },
    MissingFmt,
    MissingData,
    InvalidFmt(String),
    UnsupportedFormat(u16),
    UnsupportedBitDepth {
        format: u16,
        bits: u16,
    },
    UnsupportedChannels(u16),
    MisalignedData {
        size: usize,
        block_align: u16,
    },
}

impl fmt::Display for WavError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WavError::TooShort(n) => write!(f, "file too short: {n} bytes, need at least 12"),
            WavError::NotRiff => write!(f, "missing RIFF signature"),
            WavError::NotWave => write!(f, "RIFF form type is not WAVE"),
            WavError::ChunkOverrun {
                id,
                declared,
                available,
            } => write!(
                f,
                "chunk '{id}' declares {declared} bytes but only {available} remain"
            ),
            WavError::MissingFmt => write!(f, "no fmt chunk found"),
            WavError::MissingData => write!(f, "no data chunk found"),
            WavError::InvalidFmt(msg) => write!(f, "invalid fmt chunk: {msg}"),
            WavError::UnsupportedFormat(tag) => write!(f, "unsupported format tag 0x{tag:04X}"),
            WavError::UnsupportedBitDepth { format, bits } => {
                write!(f, "unsupported bit depth {bits} for format tag {format}")
            }
            WavError::UnsupportedChannels(n) => {
                write!(f, "unsupported channel count {n}, only mono and stereo")
            }
            WavError::MisalignedData { size, block_align } => write!(
                f,
                "data size {size} is not a multiple of block align {block_align}"
            ),
        }
    }
}

impl std::error::Error for WavError {}
