use crate::fingerprint::spectrogram::Spectrum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Peak {
    pub t: u32,
    pub f: u32,
}

pub trait PeakPicker: Send + Sync {
    fn pick(&self, spectrum: &Spectrum) -> Vec<Peak>;
}

pub struct NeighborhoodPeakPicker {
    pub freq_radius: usize,
    pub time_radius: usize,
    pub range_db: f32,
    pub min_db: f32,
    pub contrast_db: f32,
    pub min_bin: usize,
}

fn median(values: &[f32]) -> f32 {
    let mut sorted = values.to_vec();
    sorted.sort_by(f32::total_cmp);
    sorted[sorted.len() / 2]
}

fn sliding_max(values: &[f32], radius: usize) -> Vec<f32> {
    (0..values.len())
        .map(|i| {
            let lo = i.saturating_sub(radius);
            let hi = (i + radius + 1).min(values.len());
            values[lo..hi]
                .iter()
                .fold(f32::NEG_INFINITY, |m, v| m.max(*v))
        })
        .collect()
}

impl PeakPicker for NeighborhoodPeakPicker {
    fn pick(&self, spectrum: &Spectrum) -> Vec<Peak> {
        let frames = &spectrum.frames;
        if frames.is_empty() {
            return Vec::new();
        }
        let loudest = frames
            .iter()
            .flatten()
            .fold(f32::NEG_INFINITY, |m, v| m.max(*v));
        let floor = (loudest - self.range_db).max(self.min_db);
        let thresholds: Vec<f32> = frames
            .iter()
            .map(|f| floor.max(median(f) + self.contrast_db))
            .collect();
        let freq_max: Vec<Vec<f32>> = frames
            .iter()
            .map(|f| sliding_max(f, self.freq_radius))
            .collect();
        let mut peaks = Vec::new();
        for bin in self.min_bin..spectrum.bins {
            let column: Vec<f32> = freq_max.iter().map(|f| f[bin]).collect();
            let neighborhood = sliding_max(&column, self.time_radius);
            for (t, frame) in frames.iter().enumerate() {
                let v = frame[bin];
                if v > thresholds[t] && v >= neighborhood[t] {
                    peaks.push(Peak {
                        t: t as u32,
                        f: bin as u32,
                    });
                }
            }
        }
        peaks.sort();
        peaks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spectrum(frames: Vec<Vec<f32>>) -> Spectrum {
        let bins = frames[0].len();
        Spectrum {
            frames,
            bins,
            hop: 256,
            sample_rate: 8000,
        }
    }

    fn picker() -> NeighborhoodPeakPicker {
        NeighborhoodPeakPicker {
            freq_radius: 3,
            time_radius: 2,
            range_db: 40.0,
            min_db: -100.0,
            contrast_db: 10.0,
            min_bin: 1,
        }
    }

    #[test]
    fn single_bright_point_is_the_only_peak() {
        let mut frames = vec![vec![-60.0; 32]; 10];
        frames[5][12] = 0.0;
        assert_eq!(picker().pick(&spectrum(frames)), vec![Peak { t: 5, f: 12 }]);
    }

    #[test]
    fn weaker_point_inside_neighborhood_is_suppressed_but_far_one_survives() {
        let mut frames = vec![vec![-80.0; 32]; 20];
        frames[5][12] = 0.0;
        frames[6][13] = -5.0;
        frames[15][25] = -10.0;
        assert_eq!(
            picker().pick(&spectrum(frames)),
            vec![Peak { t: 5, f: 12 }, Peak { t: 15, f: 25 }]
        );
    }

    #[test]
    fn silence_has_no_constellation() {
        assert!(
            picker()
                .pick(&spectrum(vec![vec![-120.0; 32]; 10]))
                .is_empty()
        );
    }
}
