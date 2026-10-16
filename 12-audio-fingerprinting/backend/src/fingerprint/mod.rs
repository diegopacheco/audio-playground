pub mod hasher;
pub mod index;
pub mod matcher;
pub mod peaks;
pub mod spectrogram;

use crate::error::FingerprintError;
use hasher::{Fingerprint, Hasher, PairHasher};
use peaks::{NeighborhoodPeakPicker, Peak, PeakPicker};
use spectrogram::{Spectrogram, Stft};

pub const SAMPLE_RATE: u32 = 8000;

pub struct Analysis {
    pub peaks: Vec<Peak>,
    pub prints: Vec<Fingerprint>,
    pub frame_seconds: f64,
    pub bin_hz: f64,
}

pub struct Pipeline {
    spectrogram: Box<dyn Spectrogram>,
    picker: Box<dyn PeakPicker>,
    hasher: Box<dyn Hasher>,
}

impl Pipeline {
    pub fn standard() -> Result<Self, FingerprintError> {
        Ok(Pipeline {
            spectrogram: Box::new(Stft::new(1024, 256)?),
            picker: Box::new(NeighborhoodPeakPicker {
                freq_radius: 10,
                time_radius: 5,
                range_db: 50.0,
                min_db: -30.0,
                contrast_db: 15.0,
                min_bin: 4,
            }),
            hasher: Box::new(PairHasher {
                min_dt: 1,
                max_dt: 40,
                max_df: 120,
                fan_out: 6,
            }),
        })
    }

    pub fn analyze(&self, samples: &[f32]) -> Result<Analysis, FingerprintError> {
        let spectrum = self.spectrogram.compute(samples, SAMPLE_RATE)?;
        let peaks = self.picker.pick(&spectrum);
        let prints = self.hasher.hash(&peaks);
        Ok(Analysis {
            peaks,
            prints,
            frame_seconds: spectrum.frame_seconds(),
            bin_hz: spectrum.bin_hz(),
        })
    }
}
