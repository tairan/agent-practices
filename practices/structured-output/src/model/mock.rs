//! Fixture-driven model client. All tests in this concept use it. It performs
//! no IO — fixtures are loaded by the caller and handed in as strings.

use async_trait::async_trait;

use super::{CONCEPT_TIER, CompletionRequest, CompletionResponse, ModelClient, ModelFingerprint};
use crate::error::ModelError;

/// Returns whatever raw string was configured at construction. The mock makes
/// no attempt to satisfy the prompt — its job is to feed canned model outputs
/// into the pipeline so each branch (success / fenced / chatty / truncated /
/// missing-field / type-mismatch / multi-json) can be exercised deterministically.
pub struct MockClient {
    scenario: String,
    content: String,
}

impl MockClient {
    /// Build a mock with a labeled scenario; `scenario` flows into
    /// `fingerprint.response_model` as `mock-fixture::<scenario>` so logs can
    /// tell which fixture produced the response.
    pub fn with_scenario(scenario: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            scenario: scenario.into(),
            content: content.into(),
        }
    }
}

#[async_trait]
impl ModelClient for MockClient {
    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, ModelError> {
        Ok(CompletionResponse {
            content: self.content.clone(),
            fingerprint: ModelFingerprint {
                provider: "mock",
                requested_family: req.model_family,
                response_model: format!("mock-fixture::{}", self.scenario),
                api_version: None,
                capability_tier: CONCEPT_TIER,
            },
            usage: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn returns_configured_content_and_fingerprint() {
        let client = MockClient::with_scenario("ok", r#"{"hello":"world"}"#);
        let resp = client
            .complete(CompletionRequest {
                model_family: "gemini-3.5-flash".into(),
                system: None,
                user: "anything".into(),
                temperature: None,
                max_tokens: None,
            })
            .await
            .unwrap();
        assert_eq!(resp.content, r#"{"hello":"world"}"#);
        assert_eq!(resp.fingerprint.provider, "mock");
        assert_eq!(resp.fingerprint.requested_family, "gemini-3.5-flash");
        assert_eq!(resp.fingerprint.response_model, "mock-fixture::ok");
        assert_eq!(resp.fingerprint.capability_tier, "flash");
    }
}
