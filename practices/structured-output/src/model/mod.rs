//! Model transport abstraction.
//!
//! [`ModelClient`] is the project-internal minimal interface required by
//! AGENTS.md §4: model access must go through a project-defined trait, not a
//! third-party Agent framework. Two implementors:
//!
//! - [`mock::MockClient`] — fixture-driven, used by every test in this concept.
//! - [`gemini_openai::GeminiOpenAiClient`] — real HTTP client to the Gemini
//!   OpenAI-compatible endpoint.
//!
//! Every successful call returns a [`ModelFingerprint`] satisfying §4.4
//! invariant #1: each call records provider, requested family, response model,
//! API version, and the concept's declared capability tier.

pub mod gemini_openai;
pub mod mock;

use async_trait::async_trait;

use crate::error::ModelError;

/// One chat-completion turn. Intentionally minimal: no tools, no streaming,
/// no response_format — those belong to later concepts.
#[derive(Debug, Clone)]
pub struct CompletionRequest {
    /// Model family name (e.g. `gemini-3.5-flash`). Never a snapshot ID; see §4.4.
    pub model_family: String,
    pub system: Option<String>,
    pub user: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

/// What the model returned, plus the §4.4 fingerprint.
#[derive(Debug, Clone)]
pub struct CompletionResponse {
    pub content: String,
    pub fingerprint: ModelFingerprint,
    pub usage: Option<Usage>,
}

/// Token usage as reported by the provider, when available. Optional because
/// the mock client does not report it.
#[derive(Debug, Clone, Copy)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// AGENTS.md §4.4 invariant #1 + #2 in one struct.
///
/// `response_model` is the exact `model` string the provider returned in the
/// response body. Tests must include this in snapshots without asserting on
/// its value (so a same-tier snapshot move does not break the test).
#[derive(Debug, Clone)]
pub struct ModelFingerprint {
    pub provider: &'static str,
    pub requested_family: String,
    pub response_model: String,
    pub api_version: Option<String>,
    /// Capability tier this concept declares it requires; see README.
    pub capability_tier: &'static str,
}

/// The capability tier this concept requires. Any model in the `flash` tier
/// (Gemini Flash, Claude Haiku, GPT-4o-mini family) is a valid swap.
pub const CONCEPT_TIER: &str = "flash";

#[async_trait]
pub trait ModelClient: Send + Sync {
    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, ModelError>;
}
