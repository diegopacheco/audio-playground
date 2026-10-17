use crate::error::AppError;
use crate::resample::{downmix, resample};
use crate::wav;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

pub const WHISPER_RATE: u32 = 16_000;

static COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct TempDir(PathBuf);

impl TempDir {
    pub fn new() -> Result<Self, AppError> {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("stt-summary-{}-{n}", std::process::id()));
        std::fs::create_dir_all(&dir)?;
        Ok(TempDir(dir))
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

pub struct WhisperInput {
    pub wav: Vec<u8>,
    pub duration: f64,
}

pub fn to_whisper_input(bytes: &[u8], tmp: &TempDir) -> Result<WhisperInput, AppError> {
    let wav_bytes = if wav::is_wav(bytes) {
        bytes.to_vec()
    } else {
        decode_with_ffmpeg(bytes, tmp)?
    };
    let audio = wav::parse(&wav_bytes)?;
    if audio.frames() == 0 {
        return Err(AppError::BadRequest("audio has no samples".into()));
    }
    let mono = downmix(&audio.channels);
    let samples = resample(&mono, audio.sample_rate, WHISPER_RATE);
    Ok(WhisperInput {
        duration: samples.len() as f64 / WHISPER_RATE as f64,
        wav: wav::write_mono_16bit(&samples, WHISPER_RATE),
    })
}

fn decode_with_ffmpeg(bytes: &[u8], tmp: &TempDir) -> Result<Vec<u8>, AppError> {
    let input = tmp.path().join("input.bin");
    let output = tmp.path().join("decoded.wav");
    std::fs::write(&input, bytes)?;
    let result = Command::new("ffmpeg")
        .args(["-hide_banner", "-loglevel", "error", "-y", "-i"])
        .arg(&input)
        .args(["-vn", "-acodec", "pcm_s16le", "-f", "wav"])
        .arg(&output)
        .output()
        .map_err(|e| AppError::Decode(format!("could not run ffmpeg: {e}")))?;
    if !result.status.success() {
        return Err(AppError::Decode(
            "ffmpeg could not read the upload, send WAV or WebM audio".into(),
        ));
    }
    Ok(std::fs::read(&output)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stereo_44k1_wav_becomes_16k_mono_with_same_duration() {
        let samples = vec![0.1f32; 88_200];
        let mut bytes = wav::write_mono_16bit(&samples, 44_100);
        bytes[22] = 2;
        bytes[32] = 4;
        let tmp = TempDir::new().unwrap();
        let out = to_whisper_input(&bytes, &tmp).unwrap();
        let parsed = wav::parse(&out.wav).unwrap();
        assert_eq!(parsed.sample_rate, WHISPER_RATE);
        assert_eq!(parsed.channels.len(), 1);
        assert!((out.duration - 1.0).abs() < 0.001, "{}", out.duration);
    }

    #[test]
    fn empty_wav_is_a_bad_request() {
        let tmp = TempDir::new().unwrap();
        let bytes = wav::write_mono_16bit(&[], 16_000);
        assert!(matches!(
            to_whisper_input(&bytes, &tmp),
            Err(AppError::BadRequest(_))
        ));
    }

    #[test]
    fn temp_dir_is_removed_on_drop() {
        let path = {
            let tmp = TempDir::new().unwrap();
            tmp.path().to_path_buf()
        };
        assert!(!path.exists());
    }
}
