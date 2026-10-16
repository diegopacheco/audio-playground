use crate::error::AppError;
use serde::Serialize;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Segment {
    pub start: f64,
    pub end: f64,
    pub text: String,
}

pub trait Transcriber: Send + Sync {
    fn transcribe(&self, wav16k: &Path) -> Result<Vec<Segment>, AppError>;
    fn binary_available(&self) -> bool;
    fn model_available(&self) -> bool;
}

pub struct WhisperCppTranscriber {
    binary: String,
    model: PathBuf,
}

impl WhisperCppTranscriber {
    pub fn new(binary: String, model: PathBuf) -> Self {
        Self { binary, model }
    }
}

fn on_path(binary: &str) -> bool {
    if binary.contains('/') {
        return Path::new(binary).is_file();
    }
    std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).any(|dir| dir.join(binary).is_file()))
        .unwrap_or(false)
}

impl Transcriber for WhisperCppTranscriber {
    fn transcribe(&self, wav16k: &Path) -> Result<Vec<Segment>, AppError> {
        if !self.binary_available() {
            return Err(AppError::Whisper(format!(
                "{} not found, run scripts/setup.sh",
                self.binary
            )));
        }
        if !self.model_available() {
            return Err(AppError::Whisper(format!(
                "model {} not found, run scripts/setup.sh",
                self.model.display()
            )));
        }
        let base = wav16k.with_extension("");
        let output = Command::new(&self.binary)
            .arg("-m")
            .arg(&self.model)
            .arg("-f")
            .arg(wav16k)
            .args(["-l", "en", "-oj", "-np", "-of"])
            .arg(&base)
            .output()
            .map_err(|e| AppError::Whisper(format!("could not run {}: {e}", self.binary)))?;
        if !output.status.success() {
            return Err(AppError::Whisper(
                String::from_utf8_lossy(&output.stderr).trim().to_string(),
            ));
        }
        let json = std::fs::read_to_string(base.with_extension("json"))?;
        parse_whisper_json(&json)
    }

    fn binary_available(&self) -> bool {
        on_path(&self.binary)
    }

    fn model_available(&self) -> bool {
        self.model.is_file()
    }
}

pub fn parse_whisper_json(json: &str) -> Result<Vec<Segment>, AppError> {
    let root: Value =
        serde_json::from_str(json).map_err(|e| AppError::WhisperOutput(e.to_string()))?;
    let items = root["transcription"]
        .as_array()
        .ok_or_else(|| AppError::WhisperOutput("missing transcription array".into()))?;
    items
        .iter()
        .map(|item| {
            let from = item["offsets"]["from"].as_f64();
            let to = item["offsets"]["to"].as_f64();
            let text = item["text"].as_str();
            match (from, to, text) {
                (Some(from), Some(to), Some(text)) => Ok(Segment {
                    start: from / 1000.0,
                    end: to / 1000.0,
                    text: text.trim().to_string(),
                }),
                _ => Err(AppError::WhisperOutput(
                    "segment without offsets or text".into(),
                )),
            }
        })
        .filter(|s| !matches!(s, Ok(seg) if seg.text.is_empty()))
        .collect()
}

pub fn join_text(segments: &[Segment]) -> String {
    segments
        .iter()
        .map(|s| s.text.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    const OUTPUT: &str = r#"{
      "systeminfo": "AVX = 0",
      "model": {"type": "base"},
      "result": {"language": "en"},
      "transcription": [
        {"timestamps": {"from": "00:00:00,000", "to": "00:00:02,500"},
         "offsets": {"from": 0, "to": 2500}, "text": " Hello team."},
        {"timestamps": {"from": "00:00:02,500", "to": "00:00:05,120"},
         "offsets": {"from": 2500, "to": 5120}, "text": " Ship the release on Friday."},
        {"timestamps": {"from": "00:00:05,120", "to": "00:00:05,200"},
         "offsets": {"from": 5120, "to": 5200}, "text": "   "}
      ]
    }"#;

    #[test]
    fn offsets_in_milliseconds_become_seconds_and_text_is_trimmed() {
        let segments = parse_whisper_json(OUTPUT).unwrap();
        assert_eq!(segments.len(), 2);
        assert_eq!(
            segments[1],
            Segment {
                start: 2.5,
                end: 5.12,
                text: "Ship the release on Friday.".into()
            }
        );
    }

    #[test]
    fn blank_segments_are_dropped_so_transcript_has_no_gaps_of_whitespace() {
        let text = join_text(&parse_whisper_json(OUTPUT).unwrap());
        assert_eq!(text, "Hello team. Ship the release on Friday.");
    }

    #[test]
    fn output_without_transcription_is_an_error_not_an_empty_transcript() {
        assert!(parse_whisper_json(r#"{"result":{}}"#).is_err());
        assert!(parse_whisper_json("garbage").is_err());
        assert!(parse_whisper_json(r#"{"transcription":[{"text":"x"}]}"#).is_err());
    }

    #[test]
    fn missing_binary_is_reported_by_health_and_by_transcribe() {
        let t = WhisperCppTranscriber::new(
            "whisper-cli-does-not-exist".into(),
            PathBuf::from("/nonexistent/model.bin"),
        );
        assert!(!t.binary_available());
        assert!(!t.model_available());
        assert!(matches!(
            t.transcribe(Path::new("/tmp/x.wav")),
            Err(AppError::Whisper(_))
        ));
    }
}
