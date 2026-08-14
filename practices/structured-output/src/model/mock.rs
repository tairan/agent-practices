//! Fixture-driven model client. All tests in this concept use it. It performs
//! no IO — fixtures are loaded by the caller and handed in as strings.

use async_trait::async_trait;

use super::{
    CONCEPT_TIER, CapabilitySet, CapabilitySupport, CompletionRequest, CompletionResponse,
    ConformanceStatus, ContentFilterStatus, FinishReason, ModelCapabilities, ModelClient,
    ModelFingerprint,
};
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
    fn capabilities(&self) -> ModelCapabilities {
        ModelCapabilities {
            provider_declared: CapabilitySet {
                streaming: CapabilitySupport::Unsupported,
                tool_calling: CapabilitySupport::Unsupported,
                parallel_tool_calling: CapabilitySupport::Unsupported,
                structured_output: CapabilitySupport::Unsupported,
                json_schema_dialect: None,
                context_token_limit: None,
                output_token_limit: None,
                total_token_limit: None,
                usage_reporting: CapabilitySupport::Unsupported,
                prompt_cache: CapabilitySupport::Unsupported,
            },
            adapter_implemented: CapabilitySet {
                streaming: CapabilitySupport::Unsupported,
                tool_calling: CapabilitySupport::Unsupported,
                parallel_tool_calling: CapabilitySupport::Unsupported,
                structured_output: CapabilitySupport::Unsupported,
                json_schema_dialect: None,
                context_token_limit: None,
                output_token_limit: None,
                total_token_limit: None,
                usage_reporting: CapabilitySupport::Unsupported,
                prompt_cache: CapabilitySupport::Unsupported,
            },
            conformance_status: ConformanceStatus::DeterministicFixtureOnly,
        }
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, ModelError> {
        Ok(CompletionResponse {
            content: self.content.clone(),
            fingerprint: ModelFingerprint {
                provider: "mock",
                requested_family: req.model_family,
                response_model: format!("mock-fixture::{}", self.scenario),
                response_model_missing: false,
                api_version: None,
                capability_tier: CONCEPT_TIER,
            },
            usage: None,
            finish_reason: FinishReason::Stop,
            content_filter: ContentFilterStatus::NotFiltered,
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
                context: crate::ContextBuilder::new("test", 1, 100)
                    .add(crate::ContextItem::new(
                        crate::ContextRole::System,
                        "system",
                        "test://system",
                        "test",
                        crate::TrustLevel::TrustedInstruction,
                        "test",
                        crate::AccessDecision::allowed("test"),
                        1,
                        "v1",
                        None,
                        "test",
                    ))
                    .add(crate::ContextItem::new(
                        crate::ContextRole::User,
                        "anything",
                        "test://user",
                        "test",
                        crate::TrustLevel::UntrustedData,
                        "test",
                        crate::AccessDecision::allowed("test"),
                        1,
                        "v1",
                        None,
                        "test",
                    ))
                    .build()
                    .unwrap(),
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

    #[tokio::test]
    async fn request_and_response_debug_omit_raw_content_and_identifiers() {
        let canary = "SECRET_MODEL_DEBUG_CANARY_9f31";
        let request = CompletionRequest {
            model_family: canary.into(),
            context: crate::ContextBuilder::new("test", 1, 100)
                .add(crate::ContextItem::new(
                    crate::ContextRole::System,
                    canary,
                    canary,
                    canary,
                    crate::TrustLevel::TrustedInstruction,
                    "test",
                    crate::AccessDecision::allowed(canary),
                    1,
                    canary,
                    None,
                    canary,
                ))
                .add(crate::ContextItem::new(
                    crate::ContextRole::User,
                    canary,
                    canary,
                    canary,
                    crate::TrustLevel::UntrustedData,
                    "test",
                    crate::AccessDecision::allowed(canary),
                    1,
                    canary,
                    None,
                    canary,
                ))
                .build()
                .unwrap(),
            temperature: None,
            max_tokens: None,
        };
        assert!(!format!("{request:?}").contains(canary));

        let response = MockClient::with_scenario(canary, canary)
            .complete(request)
            .await
            .unwrap();
        assert_eq!(response.content, canary);
        assert!(!format!("{response:?}").contains(canary));
    }
}
