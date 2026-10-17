use crate::dsp::noise::add_white_noise;
use crate::dsp::resample::to_rate;
use crate::dsp::synth::{SONGS, render};
use crate::error::FingerprintError;
use crate::fingerprint::index::{Index, MemoryIndex};
use crate::fingerprint::matcher::{Matcher, OffsetHistogramMatcher};
use crate::fingerprint::{Pipeline, SAMPLE_RATE};
use crate::wav::Audio;
use serde::Serialize;

pub const SONG_SECONDS: f32 = 30.0;
pub const MIN_SCORE: u32 = 12;

pub struct Track {
    pub name: String,
    pub samples: Vec<f32>,
    pub hashes: usize,
    pub builtin: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TrackInfo {
    pub id: usize,
    pub name: String,
    pub duration_sec: f64,
    pub hashes: usize,
    pub builtin: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct MatchedTrack {
    pub track_id: usize,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CandidateInfo {
    pub track_id: usize,
    pub name: String,
    pub score: u32,
    pub offset_sec: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PeakPoint {
    pub t: f64,
    pub hz: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct HistogramBin {
    pub offset_sec: f64,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Identification {
    #[serde(rename = "match")]
    pub matched: Option<MatchedTrack>,
    pub score: u32,
    pub offset_sec: f64,
    pub candidates: Vec<CandidateInfo>,
    pub peaks: Vec<PeakPoint>,
    pub histogram: Vec<HistogramBin>,
    pub duration_sec: f64,
    pub hash_count: usize,
    pub min_score: u32,
}

pub struct Library {
    pipeline: Pipeline,
    index: Box<dyn Index>,
    matcher: Box<dyn Matcher>,
    tracks: Vec<Track>,
}

impl Library {
    pub fn empty() -> Result<Self, FingerprintError> {
        Ok(Library {
            pipeline: Pipeline::standard()?,
            index: Box::new(MemoryIndex::default()),
            matcher: Box::new(OffsetHistogramMatcher { max_candidates: 6 }),
            tracks: Vec::new(),
        })
    }

    pub fn builtin() -> Result<Self, FingerprintError> {
        let mut library = Library::empty()?;
        for spec in SONGS {
            let samples = render(spec, SAMPLE_RATE, SONG_SECONDS);
            library.insert(spec.name.to_string(), samples, true)?;
        }
        Ok(library)
    }

    fn insert(
        &mut self,
        name: String,
        samples: Vec<f32>,
        builtin: bool,
    ) -> Result<TrackInfo, FingerprintError> {
        let analysis = self.pipeline.analyze(&samples)?;
        let id = self.tracks.len();
        self.index.insert(id as u32, &analysis.prints);
        self.tracks.push(Track {
            name,
            samples,
            hashes: analysis.prints.len(),
            builtin,
        });
        self.info(id)
    }

    pub fn add(&mut self, name: &str, audio: &Audio) -> Result<TrackInfo, FingerprintError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(FingerprintError::InvalidParameter(
                "name is required".into(),
            ));
        }
        let samples = to_rate(audio, SAMPLE_RATE)?;
        self.insert(name.to_string(), samples, false)
    }

    fn track(&self, id: usize) -> Result<&Track, FingerprintError> {
        self.tracks
            .get(id)
            .ok_or(FingerprintError::UnknownTrack(id))
    }

    pub fn info(&self, id: usize) -> Result<TrackInfo, FingerprintError> {
        let t = self.track(id)?;
        Ok(TrackInfo {
            id,
            name: t.name.clone(),
            duration_sec: t.samples.len() as f64 / SAMPLE_RATE as f64,
            hashes: t.hashes,
            builtin: t.builtin,
        })
    }

    pub fn list(&self) -> Result<Vec<TrackInfo>, FingerprintError> {
        (0..self.tracks.len()).map(|id| self.info(id)).collect()
    }

    pub fn index_size(&self) -> usize {
        self.index.len()
    }

    pub fn clip(
        &self,
        id: usize,
        start: f64,
        len: f64,
        noise: f32,
    ) -> Result<Audio, FingerprintError> {
        let track = self.track(id)?;
        if !start.is_finite() || start < 0.0 || !len.is_finite() || len <= 0.0 {
            return Err(FingerprintError::InvalidParameter(format!(
                "start {start} len {len}"
            )));
        }
        if !noise.is_finite() || !(0.0..=10.0).contains(&noise) {
            return Err(FingerprintError::InvalidParameter(format!("noise {noise}")));
        }
        let rate = SAMPLE_RATE as f64;
        let from = (start * rate) as usize;
        let to = ((start + len) * rate) as usize;
        if from >= track.samples.len() {
            return Err(FingerprintError::InvalidParameter(format!(
                "start {start} beyond track end"
            )));
        }
        let slice = &track.samples[from..to.min(track.samples.len())];
        let samples = if noise > 0.0 {
            add_white_noise(slice, noise, (id as u64) << 32 | from as u64)
        } else {
            slice.to_vec()
        };
        Ok(Audio {
            sample_rate: SAMPLE_RATE,
            samples,
        })
    }

    pub fn identify(&self, audio: &Audio) -> Result<Identification, FingerprintError> {
        let samples = to_rate(audio, SAMPLE_RATE)?;
        let analysis = self.pipeline.analyze(&samples)?;
        let result = self.matcher.rank(self.index.as_ref(), &analysis.prints);
        let frame = analysis.frame_seconds;
        let candidates = result
            .candidates
            .iter()
            .map(|c| {
                Ok(CandidateInfo {
                    track_id: c.track as usize,
                    name: self.track(c.track as usize)?.name.clone(),
                    score: c.score,
                    offset_sec: c.offset as f64 * frame,
                })
            })
            .collect::<Result<Vec<_>, FingerprintError>>()?;
        let (score, offset_sec) = candidates
            .first()
            .map(|c| (c.score, c.offset_sec))
            .unwrap_or((0, 0.0));
        let matched = candidates
            .first()
            .filter(|c| c.score >= MIN_SCORE)
            .map(|c| MatchedTrack {
                track_id: c.track_id,
                name: c.name.clone(),
            });
        Ok(Identification {
            matched,
            score,
            offset_sec,
            candidates,
            peaks: analysis
                .peaks
                .iter()
                .map(|p| PeakPoint {
                    t: p.t as f64 * frame,
                    hz: p.f as f64 * analysis.bin_hz,
                })
                .collect(),
            histogram: result
                .histogram
                .iter()
                .map(|&(offset, count)| HistogramBin {
                    offset_sec: offset as f64 * frame,
                    count,
                })
                .collect(),
            duration_sec: samples.len() as f64 / SAMPLE_RATE as f64,
            hash_count: analysis.prints.len(),
            min_score: MIN_SCORE,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::noise::Rng;
    use std::f32::consts::PI;
    use std::sync::OnceLock;

    fn library() -> &'static Library {
        static LIB: OnceLock<Library> = OnceLock::new();
        LIB.get_or_init(|| Library::builtin().unwrap())
    }

    fn mid_start(id: usize) -> f64 {
        11.37 + id as f64 * 0.91
    }

    #[test]
    fn clip_from_middle_of_each_song_identifies_song_and_offset() {
        let lib = library();
        for id in 0..SONGS.len() {
            let start = mid_start(id);
            let clip = lib.clip(id, start, 5.0, 0.0).unwrap();
            let r = lib.identify(&clip).unwrap();
            let m = r
                .matched
                .clone()
                .unwrap_or_else(|| panic!("song {id} not matched"));
            assert_eq!(m.track_id, id, "song {id} matched {}", m.name);
            assert!(
                (r.offset_sec - start).abs() <= 0.1,
                "song {id} offset {} expected {start}",
                r.offset_sec
            );
            if let Some(second) = r.candidates.get(1) {
                assert!(r.score > second.score * 3, "song {id} weak margin");
            }
        }
    }

    #[test]
    fn noisy_clip_still_identifies_the_right_song() {
        let lib = library();
        for id in 0..SONGS.len() {
            let start = mid_start(id) + 2.0;
            let clip = lib.clip(id, start, 6.0, 1.0).unwrap();
            let r = lib.identify(&clip).unwrap();
            let m = r
                .matched
                .clone()
                .unwrap_or_else(|| panic!("noisy song {id} not matched, score {}", r.score));
            assert_eq!(m.track_id, id);
            assert!((r.offset_sec - start).abs() <= 0.1);
        }
    }

    #[test]
    fn query_at_another_sample_rate_is_resampled_before_matching() {
        let lib = library();
        let clip = lib.clip(3, 9.0, 5.0, 0.0).unwrap();
        let up: Vec<f32> = clip.samples.iter().flat_map(|&s| [s; 2]).collect();
        let r = lib
            .identify(&Audio {
                sample_rate: 16000,
                samples: up,
            })
            .unwrap();
        assert_eq!(r.matched.map(|m| m.track_id), Some(3));
    }

    #[test]
    fn white_noise_is_not_identified_as_any_song() {
        let mut rng = Rng::new(99);
        let samples: Vec<f32> = (0..40000).map(|_| rng.signed() * 0.5).collect();
        let r = library()
            .identify(&Audio {
                sample_rate: SAMPLE_RATE,
                samples,
            })
            .unwrap();
        assert!(r.matched.is_none(), "noise matched with score {}", r.score);
        assert!(r.score < MIN_SCORE);
    }

    #[test]
    fn unrelated_chirp_is_not_identified_as_any_song() {
        let samples: Vec<f32> = (0..40000)
            .map(|i| {
                let t = i as f32 / SAMPLE_RATE as f32;
                (2.0 * PI * (200.0 * t + 150.0 * t * t)).sin() * 0.5
            })
            .collect();
        let r = library()
            .identify(&Audio {
                sample_rate: SAMPLE_RATE,
                samples,
            })
            .unwrap();
        assert!(r.matched.is_none(), "chirp matched with score {}", r.score);
    }

    #[test]
    fn library_fingerprints_are_deterministic_across_builds() {
        let a = Library::builtin().unwrap().list().unwrap();
        assert_eq!(a, library().list().unwrap());
        assert_eq!(a.len(), SONGS.len());
        assert!(a.iter().all(|t| t.hashes > 1000), "{a:?}");
    }

    #[test]
    fn added_track_becomes_identifiable() {
        let mut lib = Library::builtin().unwrap();
        let samples: Vec<f32> = (0..16000 * 12)
            .map(|i| {
                let t = i as f32 / 16000.0;
                let note = 300.0 + 100.0 * ((t * 3.0).floor() % 7.0);
                (2.0 * PI * note * t).sin() * 0.3 + (2.0 * PI * note * 1.5 * t).sin() * 0.2
            })
            .collect();
        let info = lib
            .add(
                "Uploaded",
                &Audio {
                    sample_rate: 16000,
                    samples,
                },
            )
            .unwrap();
        let clip = lib.clip(info.id, 4.0, 4.0, 0.0).unwrap();
        let r = lib.identify(&clip).unwrap();
        assert_eq!(r.matched.map(|m| m.track_id), Some(info.id));
    }

    #[test]
    fn clip_rejects_out_of_range_parameters() {
        let lib = library();
        assert!(lib.clip(99, 0.0, 1.0, 0.0).is_err());
        assert!(lib.clip(0, 100.0, 1.0, 0.0).is_err());
        assert!(lib.clip(0, 0.0, -1.0, 0.0).is_err());
    }
}
