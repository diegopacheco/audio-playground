use crate::error::AppError;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::time::Duration;

pub const API_URL: &str = "https://api.anthropic.com/v1/messages";
pub const API_VERSION: &str = "2023-06-01";
pub const FALLBACK_BETA: &str = "server-side-fallback-2026-07-01";
pub const DEFAULT_MODEL: &str = "claude-fable-5-1";
const MAX_TOKENS: u32 = 16_000;

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Summary,
    ActionItems,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SummaryResult {
    pub summary: String,
    pub action_items: Vec<String>,
}

pub trait Summarizer: Send + Sync {
    fn summarize(&self, text: &str, mode: Mode) -> Result<SummaryResult, AppError>;
    fn has_key(&self) -> bool;
}

pub struct ClaudeSummarizer {
    api_key: Option<String>,
    model: String,
}

impl ClaudeSummarizer {
    pub fn new(api_key: Option<String>, model: String) -> Self {
        let api_key = api_key.filter(|k| !k.trim().is_empty());
        Self { api_key, model }
    }

    pub fn from_env() -> Self {
        let model = std::env::var("CLAUDE_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.to_string());
        Self::new(std::env::var("ANTHROPIC_API_KEY").ok(), model)
    }
}

fn instruction(mode: Mode) -> &'static str {
    match mode {
        Mode::Summary => {
            "Write a concise summary of the transcript in 2 to 5 sentences. Put it in \"summary\". Set \"action_items\" to an empty array."
        }
        Mode::ActionItems => {
            "Extract every concrete action item, task, decision to follow up, or commitment from the transcript, one short imperative sentence each, in \"action_items\". Put a one sentence overview in \"summary\". Use an empty array when there are none."
        }
    }
}

pub fn build_request(model: &str, text: &str, mode: Mode) -> Value {
    json!({
        "model": model,
        "max_tokens": MAX_TOKENS,
        "fallbacks": "default",
        "output_config": {"effort": "low"},
        "system": "You process speech-to-text transcripts. The transcript may contain recognition errors; infer intent. Reply with only a JSON object of the form {\"summary\": string, \"action_items\": string[]} and no other text.",
        "messages": [{
            "role": "user",
            "content": format!("{}\n\n<transcript>\n{}\n</transcript>", instruction(mode), text)
        }]
    })
}

pub fn parse_response(body: &str) -> Result<SummaryResult, AppError> {
    let root: Value = serde_json::from_str(body).map_err(|e| AppError::Claude(e.to_string()))?;
    if root["type"] == "error" {
        let message = root["error"]["message"].as_str().unwrap_or("unknown error");
        return Err(AppError::Claude(message.to_string()));
    }
    if root["stop_reason"] == "refusal" {
        return Err(AppError::Claude("the model declined this request".into()));
    }
    let text: String = root["content"]
        .as_array()
        .ok_or_else(|| AppError::Claude("response has no content".into()))?
        .iter()
        .filter(|block| block["type"] == "text")
        .filter_map(|block| block["text"].as_str())
        .collect();
    Ok(extract_result(&text))
}

fn extract_result(text: &str) -> SummaryResult {
    let json_slice = match (text.find('{'), text.rfind('}')) {
        (Some(a), Some(b)) if a < b => &text[a..=b],
        _ => "",
    };
    serde_json::from_str(json_slice).unwrap_or_else(|_| SummaryResult {
        summary: text.trim().to_string(),
        action_items: Vec::new(),
    })
}

impl Summarizer for ClaudeSummarizer {
    fn summarize(&self, text: &str, mode: Mode) -> Result<SummaryResult, AppError> {
        let key = self.api_key.as_deref().ok_or(AppError::MissingApiKey)?;
        if text.trim().is_empty() {
            return Err(AppError::BadRequest("text is empty".into()));
        }
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(600))
            .build()
            .map_err(|e| AppError::Claude(e.to_string()))?;
        let response = client
            .post(API_URL)
            .header("x-api-key", key)
            .header("anthropic-version", API_VERSION)
            .header("anthropic-beta", FALLBACK_BETA)
            .header("content-type", "application/json")
            .body(build_request(&self.model, text, mode).to_string())
            .send()
            .map_err(|e| AppError::Claude(e.without_url().to_string()))?;
        let status = response.status();
        let body = response
            .text()
            .map_err(|e| AppError::Claude(e.to_string()))?;
        let parsed = parse_response(&body);
        if !status.is_success() {
            return Err(match parsed {
                Err(e) => e,
                Ok(_) => AppError::Claude(format!("http status {status}")),
            });
        }
        parsed
    }

    fn has_key(&self) -> bool {
        self.api_key.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_targets_configured_model_with_transcript_and_json_contract() {
        let req = build_request(DEFAULT_MODEL, "we ship friday", Mode::ActionItems);
        assert_eq!(req["model"], "claude-fable-5-1");
        assert_eq!(req["max_tokens"], 16_000);
        assert_eq!(req["fallbacks"], "default");
        assert!(req.get("thinking").is_none());
        assert!(req.get("temperature").is_none());
        assert!(req["system"].as_str().unwrap().contains("action_items"));
        let messages = req["messages"].as_array().unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["role"], "user");
        let content = messages[0]["content"].as_str().unwrap();
        assert!(content.contains("<transcript>\nwe ship friday\n</transcript>"));
        assert!(content.contains("action item"));
    }

    #[test]
    fn summary_and_action_item_modes_ask_for_different_things() {
        let a = build_request(DEFAULT_MODEL, "t", Mode::Summary);
        let b = build_request(DEFAULT_MODEL, "t", Mode::ActionItems);
        assert_ne!(a["messages"][0]["content"], b["messages"][0]["content"]);
    }

    #[test]
    fn json_text_block_is_parsed_and_thinking_blocks_are_ignored() {
        let body = r#"{"type":"message","role":"assistant","stop_reason":"end_turn",
          "content":[{"type":"thinking","thinking":"","signature":"x"},
          {"type":"text","text":"```json\n{\"summary\":\"Release moved.\",\"action_items\":[\"Ship on Friday\",\"Email QA\"]}\n```"}]}"#;
        let r = parse_response(body).unwrap();
        assert_eq!(r.summary, "Release moved.");
        assert_eq!(r.action_items, vec!["Ship on Friday", "Email QA"]);
    }

    #[test]
    fn plain_text_reply_still_returns_a_summary() {
        let body = r#"{"type":"message","stop_reason":"end_turn","content":[{"type":"text","text":"Just a summary."}]}"#;
        let r = parse_response(body).unwrap();
        assert_eq!(r.summary, "Just a summary.");
        assert!(r.action_items.is_empty());
    }

    #[test]
    fn api_errors_and_refusals_surface_as_errors() {
        let err = r#"{"type":"error","error":{"type":"authentication_error","message":"invalid x-api-key"}}"#;
        assert!(
            matches!(parse_response(err), Err(AppError::Claude(m)) if m == "invalid x-api-key")
        );
        let refusal = r#"{"type":"message","stop_reason":"refusal","content":[]}"#;
        assert!(matches!(parse_response(refusal), Err(AppError::Claude(_))));
    }

    #[test]
    fn missing_or_blank_key_fails_before_any_network_call() {
        for key in [None, Some("   ".to_string())] {
            let s = ClaudeSummarizer::new(key, DEFAULT_MODEL.into());
            assert!(!s.has_key());
            assert!(matches!(
                s.summarize("hello", Mode::Summary),
                Err(AppError::MissingApiKey)
            ));
        }
    }

    #[test]
    fn mode_deserializes_from_api_strings() {
        let m: Mode = serde_json::from_str("\"action_items\"").unwrap();
        assert_eq!(m, Mode::ActionItems);
        assert!(serde_json::from_str::<Mode>("\"poem\"").is_err());
    }
}
