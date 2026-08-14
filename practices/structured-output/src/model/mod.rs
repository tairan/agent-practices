//! Model transport abstraction.
//!
//! [`ModelClient`] is the project-internal minimal interface required by
//! The Model contract requires access through a project-defined trait, not a
//! third-party Agent framework. Two implementors:
//!
//! - [`mock::MockClient`] — fixture-driven, used by every test in this concept.
//! - [`gemini_openai::GeminiOpenAiClient`] — real HTTP client to the Gemini
//!   OpenAI-compatible endpoint.
//!
//! Every implementation exposes a capability descriptor, and every successful
//! call returns identity, usage, finish, filter, and latency metadata.

pub mod gemini_openai;
pub mod mock;

use async_trait::async_trait;
use std::fmt;

use crate::{BuiltContext, error::ModelError};

/// One chat-completion turn. Intentionally minimal: no tools, no streaming,
/// no response_format — those belong to later concepts.
#[derive(Clone)]
pub struct CompletionRequest {
    /// Model family name (e.g. `gemini-3.5-flash`), never a snapshot ID.
    pub model_family: String,
    pub context: BuiltContext,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

impl fmt::Debug for CompletionRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CompletionRequest")
            .field("model_family_bytes", &self.model_family.len())
            .field("context", &self.context)
            .field("temperature", &self.temperature)
            .field("max_tokens", &self.max_tokens)
            .finish()
    }
}

/// What the model returned, plus provider metadata.
#[derive(Clone)]
pub struct CompletionResponse {
    pub content: String,
    pub fingerprint: ModelFingerprint,
    pub usage: Option<Usage>,
    pub finish_reason: FinishReason,
    pub content_filter: ContentFilterStatus,
}

impl fmt::Debug for CompletionResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CompletionResponse")
            .field("content_bytes", &self.content.len())
            .field("fingerprint", &self.fingerprint)
            .field("usage", &self.usage)
            .field("finish_reason", &self.finish_reason)
            .field("content_filter", &self.content_filter)
            .finish()
    }
}

/// Token usage as reported by the provider, when available. Optional because
/// the mock client does not report it.
#[derive(Debug, Clone, Copy)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Model identity and capability-equivalence information.
///
/// `response_model` is the exact `model` string the provider returned in the
/// response body. Tests must include this in snapshots without asserting on
/// its value (so a same-tier snapshot move does not break the test).
#[derive(Clone)]
pub struct ModelFingerprint {
    pub provider: &'static str,
    pub requested_family: String,
    pub response_model: String,
    pub response_model_missing: bool,
    pub api_version: Option<String>,
    /// Capability tier this concept declares it requires; see README.
    pub capability_tier: &'static str,
}

impl fmt::Debug for ModelFingerprint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ModelFingerprint")
            .field("provider", &self.provider)
            .field("requested_family_bytes", &self.requested_family.len())
            .field("response_model_bytes", &self.response_model.len())
            .field("response_model_missing", &self.response_model_missing)
            .field("api_version_present", &self.api_version.is_some())
            .field("capability_tier", &self.capability_tier)
            .finish()
    }
}

#[derive(Clone)]
pub struct ModelCallMetadata {
    pub fingerprint: ModelFingerprint,
    pub usage: Option<Usage>,
    pub finish_reason: FinishReason,
    pub content_filter: ContentFilterStatus,
    pub latency: std::time::Duration,
}

impl fmt::Debug for ModelCallMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ModelCallMetadata")
            .field("fingerprint", &self.fingerprint)
            .field("usage", &self.usage)
            .field("finish_reason", &self.finish_reason)
            .field("content_filter", &self.content_filter)
            .field("latency", &self.latency)
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinishReason {
    Stop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentFilterStatus {
    NotFiltered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilitySupport {
    Supported,
    Unsupported,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapabilitySet {
    pub streaming: CapabilitySupport,
    pub tool_calling: CapabilitySupport,
    pub parallel_tool_calling: CapabilitySupport,
    pub structured_output: CapabilitySupport,
    pub json_schema_dialect: Option<&'static str>,
    pub context_token_limit: Option<u32>,
    pub output_token_limit: Option<u32>,
    pub total_token_limit: Option<u32>,
    pub usage_reporting: CapabilitySupport,
    pub prompt_cache: CapabilitySupport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConformanceStatus {
    Deferred,
    DeterministicFixtureOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelCapabilities {
    pub provider_declared: CapabilitySet,
    pub adapter_implemented: CapabilitySet,
    pub conformance_status: ConformanceStatus,
}

/// The single provider/model class this adapter has declared and tested locally.
pub const CONCEPT_TIER: &str = "flash";
pub const VERIFIED_MODEL_FAMILY: &str = "gemini-3.5-flash";
pub const PROVIDER_CONTEXT_TOKEN_LIMIT: u32 = 1_048_576;
pub const PROVIDER_OUTPUT_TOKEN_LIMIT: u32 = 65_536;
pub const ADAPTER_TOTAL_TOKEN_LIMIT: u32 = PROVIDER_CONTEXT_TOKEN_LIMIT;
pub const MIN_MODEL_TIMEOUT_SECS: u64 = 1;
pub const MAX_MODEL_TIMEOUT_SECS: u64 = 120;

#[async_trait]
pub trait ModelClient: Send + Sync {
    fn capabilities(&self) -> ModelCapabilities;

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, ModelError>;
}
