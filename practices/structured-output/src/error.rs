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
//! All variants that hold a raw model output excerpt truncate it to
//! [`MAX_EXCERPT_CHARS`] characters before storage, per AGENTS.md §8: traces and
//! error logs must not record full model outputs by default.

use serde::Serialize;

/// Upper bound on any raw-model-content excerpt embedded in an error.
///
/// 512 chars is large enough to spot the problem ("oh, the model added prose
/// before the JSON") and small enough to prevent log/error message bloat.
pub const MAX_EXCERPT_CHARS: usize = 512;

/// Truncate `s` to at most `MAX_EXCERPT_CHARS` *characters* (not bytes), adding
/// a trailing `…` marker when truncation occurs. Char-based to keep UTF-8 safe.
pub fn truncate_excerpt(s: &str) -> String {
    let mut iter = s.chars();
    let head: String = iter.by_ref().take(MAX_EXCERPT_CHARS).collect();
    if iter.next().is_some() {
        format!("{head}…")
    } else {
        head
    }
}

/// A single JSON Schema validation issue, normalized for logging and tests.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SchemaIssue {
    /// JSON Pointer into the validated instance (e.g. `/action_items/0/owner`).
    pub instance_path: String,
    /// JSON Pointer into the schema where the rule lives.
    pub schema_path: String,
    /// Human-readable message from the validator.
    pub message: String,
}

/// Errors raised by the model transport.
#[derive(Debug, thiserror::Error)]
pub enum ModelError {
    #[error("http transport error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("non-2xx response: status={status}, body={body_excerpt}")]
    NonSuccess { status: u16, body_excerpt: String },

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

    #[error("could not locate a JSON object in model output (excerpt: {raw_excerpt})")]
    JsonExtractionFailed { raw_excerpt: String },

    #[error("multiple JSON candidates found in model output; ambiguous (count={count})")]
    MultipleJsonCandidates { count: usize, raw_excerpt: String },

    #[error("JSON parse error: {source}")]
    JsonParseError {
        raw_excerpt: String,
        #[source]
        source: serde_json::Error,
    },

    #[error("schema validation failed with {} issue(s)", issues.len())]
    SchemaValidationFailed { issues: Vec<SchemaIssue> },

    #[error("schema passed but typed deserialize failed: {0}")]
    TypedDeserializeFailed(#[source] serde_json::Error),

    #[error("model call failed: {0}")]
    ModelCallFailed(#[from] ModelError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_excerpt_under_limit_returns_input() {
        assert_eq!(truncate_excerpt("hello"), "hello");
    }

    #[test]
    fn truncate_excerpt_at_limit_no_marker() {
        let s: String = "a".repeat(MAX_EXCERPT_CHARS);
        let out = truncate_excerpt(&s);
        assert_eq!(out.chars().count(), MAX_EXCERPT_CHARS);
        assert!(!out.ends_with('…'));
    }

    #[test]
    fn truncate_excerpt_over_limit_adds_marker() {
        let s: String = "a".repeat(MAX_EXCERPT_CHARS + 10);
        let out = truncate_excerpt(&s);
        assert!(out.ends_with('…'));
        assert_eq!(out.chars().count(), MAX_EXCERPT_CHARS + 1);
    }

    #[test]
    fn truncate_excerpt_is_utf8_safe() {
        // multi-byte chars must not slice mid-codepoint
        let s: String = "中".repeat(MAX_EXCERPT_CHARS + 10);
        let out = truncate_excerpt(&s);
        assert!(out.ends_with('…'));
        assert!(out.chars().take(MAX_EXCERPT_CHARS).all(|c| c == '中'));
    }
}
