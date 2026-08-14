//! Structured, typed errors for the structured-output pipeline.
//!
//! Two error families intentionally kept separate:
//!
//! - [`ModelError`] — transport-layer failures from a [`crate::model::ModelClient`]
//!   implementation (HTTP errors, timeouts, malformed provider response).
//! - [`StructuredOutputError`] — pipeline-layer failures *after* the model
//!   returned content (extraction, parsing, schema validation, typed deserialize).
//!   Wraps `ModelError` for the model-call branch.
//!
//! Errors retain only stable classifications and safe metadata. They never
//! retain provider bodies, model values, credentials, or untrusted field names.

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryDisposition {
    Retryable,
    NonRetryable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportKind {
    Connection,
    Request,
}

/// A single JSON Schema validation issue, normalized for logging and tests.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SchemaIssue {
    /// Number of instance-path segments; model-provided values and names are not retained.
    pub instance_depth: usize,
    /// JSON Pointer into the schema where the rule lives.
    pub schema_path: String,
    /// Stable validation keyword derived from the trusted schema path.
    pub code: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentFilterReason {
    Safety,
    Recitation,
    Language,
    ProhibitedContent,
    SensitivePersonalInformation,
    Blocklist,
    OtherPolicy,
}

/// Errors raised by the model transport.
#[derive(Debug, thiserror::Error)]
pub enum ModelError {
    #[error("model transport error: kind={kind:?}, retry={retry:?}")]
    Transport {
        kind: TransportKind,
        retry: RetryDisposition,
    },

    #[error("model HTTP status error: status={status}, retry={retry:?}")]
    HttpStatus {
        status: u16,
        retry: RetryDisposition,
    },

    #[error("model provider rate limited the request: retry_after_seconds={retry_after_seconds:?}")]
    RateLimited { retry_after_seconds: Option<u64> },

    #[error("model provider protocol error: {code}")]
    Protocol { code: &'static str },

    #[error("model refused the request")]
    Refused,

    #[error("model response was content-filtered: {reason:?}")]
    ContentFiltered { reason: ContentFilterReason },

    #[error("model response stopped at its output limit")]
    OutputTruncated,

    #[error("model response used an unsupported finish reason")]
    AbnormalFinish,

    #[error("requested model family is outside the verified capability class")]
    UnsupportedModelFamily,

    #[error("estimated model input exceeds token limit: estimated={estimated}, max={max}")]
    InputTokenLimitExceeded { estimated: u32, max: u32 },

    #[error("requested model output exceeds token limit: requested={requested}, max={max}")]
    OutputTokenLimitExceeded { requested: u32, max: u32 },

    #[error("estimated model request exceeds total token limit: estimated={estimated}, max={max}")]
    TotalTokenLimitExceeded { estimated: u32, max: u32 },

    #[error("model response exceeded byte limit: limit={limit}")]
    ResponseTooLarge { limit: usize },

    #[error("response payload missing required field: {0}")]
    MissingField(&'static str),

    #[error("model call timed out after {0:?}")]
    Timeout(std::time::Duration),

    #[error("model configuration error: {0}")]
    Configuration(String),
}

/// Errors raised by the structured-output pipeline.
#[derive(Debug, thiserror::Error)]
pub enum StructuredOutputError {
    #[error("model returned empty content")]
    EmptyResponse,

    #[error("could not locate a JSON object in model output")]
    JsonExtractionFailed,

    #[error("model output used unexpected top-level JSON type: {actual}")]
    UnexpectedTopLevelType { actual: &'static str },

    #[error("multiple JSON candidates found in model output; ambiguous (count={count})")]
    MultipleJsonCandidates { count: usize },

    #[error("JSON parse error: {0}")]
    JsonParseError(#[source] serde_json::Error),

    #[error("schema validation failed with {} issue(s)", issues.len())]
    SchemaValidationFailed { issues: Vec<SchemaIssue> },

    #[error("schema passed but typed deserialize failed")]
    TypedDeserializeFailed,

    #[error("model call failed: {0}")]
    ModelCallFailed(#[from] ModelError),
}
