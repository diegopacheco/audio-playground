use crate::convert::{TempDir, to_whisper_input};
use crate::error::AppError;
use crate::transcriber::{Segment, Transcriber, join_text};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct TranscriptResponse {
    pub text: String,
    pub segments: Vec<Segment>,
    pub duration: f64,
}

pub fn transcribe_bytes(
    transcriber: &dyn Transcriber,
    bytes: &[u8],
) -> Result<TranscriptResponse, AppError> {
    if bytes.is_empty() {
        return Err(AppError::BadRequest("request body is empty".into()));
    }
    let tmp = TempDir::new()?;
    let input = to_whisper_input(bytes, &tmp)?;
    let path = tmp.path().join("speech16k.wav");
    std::fs::write(&path, &input.wav)?;
    let segments = transcriber.transcribe(&path)?;
    Ok(TranscriptResponse {
        text: join_text(&segments),
        segments,
        duration: input.duration,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wav;
    use std::path::Path;

    struct FakeTranscriber;

    impl Transcriber for FakeTranscriber {
        fn transcribe(&self, wav16k: &Path) -> Result<Vec<Segment>, AppError> {
            let audio = wav::parse(&std::fs::read(wav16k)?)?;
            Ok(vec![Segment {
                start: 0.0,
                end: audio.frames() as f64 / audio.sample_rate as f64,
                text: format!("{} Hz {} ch", audio.sample_rate, audio.channels.len()),
            }])
        }
        fn binary_available(&self) -> bool {
            true
        }
        fn model_available(&self) -> bool {
            true
        }
    }

    #[test]
    fn transcriber_always_receives_16k_mono_whatever_the_upload_rate() {
        let bytes = wav::write_mono_16bit(&vec![0.2; 48_000], 48_000);
        let r = transcribe_bytes(&FakeTranscriber, &bytes).unwrap();
        assert_eq!(r.text, "16000 Hz 1 ch");
        assert!((r.duration - 1.0).abs() < 0.001);
        assert!((r.segments[0].end - 1.0).abs() < 0.001);
    }

    #[test]
    fn empty_body_is_rejected_before_touching_the_transcriber() {
        assert!(matches!(
            transcribe_bytes(&FakeTranscriber, &[]),
            Err(AppError::BadRequest(_))
        ));
    }
}
