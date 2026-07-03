//! OpenAI-compatible HTTP client for the Gemini endpoint.
//!
//! Implements only the chat-completions subset required by this concept:
//! single-turn, non-streaming, no tool use, no `response_format`. The model is
//! prompted with a system message asking for a JSON object; **schema
//! validation is done by `crate::schema`, not by the provider** — that is the
//! whole point of the practice.
//!
//! AGENTS.md §4.4 invariant #1 is satisfied by reading the response body's
//! top-level `model` field and storing it in [`ModelFingerprint::response_model`].

use std::time::Duration;

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;

use super::{
    CONCEPT_TIER, CompletionRequest, CompletionResponse, ModelClient, ModelFingerprint, Usage,
};
use crate::error::{ModelError, truncate_excerpt};

/// HTTP client targeting an OpenAI-compatible `/chat/completions` endpoint.
#[derive(Debug)]
pub struct GeminiOpenAiClient {
    http: reqwest::Client,
    base_url: String,
    api_key: String,
    timeout: Duration,
    /// Stored only for fingerprint logging; e.g. `"v1beta"` extracted from base_url.
    api_version: Option<String>,
}

/// Construction parameters; kept as a struct so `main.rs` can build it from env.
pub struct GeminiConfig {
    pub base_url: String,
    pub api_key: String,
    pub timeout: Duration,
}

impl GeminiOpenAiClient {
    pub fn new(cfg: GeminiConfig) -> Result<Self, ModelError> {
        if cfg.api_key.is_empty() {
            return Err(ModelError::Configuration("empty api_key".into()));
        }
        let http = reqwest::Client::builder()
            .timeout(cfg.timeout)
            .build()
            .map_err(ModelError::Http)?;

        let api_version = extract_api_version(&cfg.base_url);

        Ok(Self {
            http,
            base_url: cfg.base_url,
            api_key: cfg.api_key,
            timeout: cfg.timeout,
            api_version,
        })
    }
}

/// Pull a path segment like `v1beta` / `v1` out of an OpenAI-compat base URL.
/// Returns `None` when the URL does not embed a version segment.
fn extract_api_version(base_url: &str) -> Option<String> {
    base_url
        .trim_end_matches('/')
        .split('/')
        .rev()
        .find(|seg| {
            seg.starts_with('v') && seg.len() <= 8 && seg[1..].chars().any(|c| c.is_ascii_digit())
        })
        .map(|s| s.to_string())
}

#[async_trait]
impl ModelClient for GeminiOpenAiClient {
    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, ModelError> {
        let url = format!("{}chat/completions", ensure_trailing_slash(&self.base_url));

        let mut messages: Vec<serde_json::Value> = Vec::new();
        if let Some(sys) = &req.system {
            messages.push(json!({ "role": "system", "content": sys }));
        }
        messages.push(json!({ "role": "user", "content": req.user }));

        let mut body = json!({
            "model": req.model_family,
            "messages": messages,
        });
        if let Some(t) = req.temperature {
            body["temperature"] = json!(t);
        }
        if let Some(m) = req.max_tokens {
            body["max_tokens"] = json!(m);
        }

        let resp = self
            .http
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    ModelError::Timeout(self.timeout)
                } else {
                    ModelError::Http(e)
                }
            })?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(ModelError::NonSuccess {
                status: status.as_u16(),
                body_excerpt: truncate_excerpt(&body),
            });
        }

        let parsed: ChatResponse = resp.json().await.map_err(ModelError::Http)?;

        let content = parsed
            .choices
            .into_iter()
            .next()
            .and_then(|c| c.message.content)
            .ok_or(ModelError::MissingField("choices[0].message.content"))?;

        let usage = parsed.usage.map(|u| Usage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        });

        let fingerprint = ModelFingerprint {
            provider: "gemini-openai-compat",
            requested_family: req.model_family,
            response_model: parsed.model,
            api_version: self.api_version.clone(),
            capability_tier: CONCEPT_TIER,
        };

        Ok(CompletionResponse {
            content,
            fingerprint,
            usage,
        })
    }
}

fn ensure_trailing_slash(s: &str) -> String {
    if s.ends_with('/') {
        s.to_string()
    } else {
        format!("{s}/")
    }
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    model: String,
    choices: Vec<ChatChoice>,
    usage: Option<ChatUsage>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Debug, Deserialize)]
struct ChatMessage {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChatUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn empty_key_is_rejected() {
        let err = GeminiOpenAiClient::new(GeminiConfig {
            base_url: "https://example.com/v1beta/openai/".into(),
            api_key: "".into(),
            timeout: Duration::from_secs(5),
        })
        .unwrap_err();
        assert!(matches!(err, ModelError::Configuration(_)));
    }

    #[test]
    fn api_version_extracted_from_base_url() {
        assert_eq!(
            extract_api_version("https://example.com/v1beta/openai/"),
            Some("v1beta".to_string())
        );
        assert_eq!(
            extract_api_version("https://example.com/v1/"),
            Some("v1".to_string())
        );
        assert_eq!(extract_api_version("https://example.com/"), None);
    }

    #[test]
    fn ensure_trailing_slash_idempotent() {
        assert_eq!(ensure_trailing_slash("a/"), "a/");
        assert_eq!(ensure_trailing_slash("a"), "a/");
    }

    #[tokio::test]
    async fn timeout_error_uses_configured_timeout() {
        let timeout = Duration::from_millis(250);
        let response_delay = Duration::from_secs(2);

        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let addr = listener.local_addr().unwrap();
        let _server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            std::thread::sleep(response_delay);
            let _ = stream.write_all(b"");
        });

        let client = GeminiOpenAiClient::new(GeminiConfig {
            base_url: format!("http://{}/v1beta/openai/", addr),
            api_key: "test-key".into(),
            timeout,
        })
        .unwrap();

        let req = CompletionRequest {
            model_family: "gemini-3.5-flash".into(),
            system: None,
            user: "test".into(),
            temperature: None,
            max_tokens: None,
        };

        let err = client.complete(req).await.unwrap_err();
        match err {
            ModelError::Timeout(actual) => {
                assert_eq!(actual, timeout);
            }
            other => panic!("expected timeout error, got {other:?}"),
        }
    }
}
