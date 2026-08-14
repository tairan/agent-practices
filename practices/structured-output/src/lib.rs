//! `structured-output` practice of agent-practices.
//!
//! Scope (intentionally narrow): take one model call, locate a JSON object in
//! the response, validate it against a JSON Schema (Draft 2020-12), and either
//! return a typed value or a structured error. No retries, no reflection, no
//! Agent loop; see the README non-goals for the explicit boundary.

pub mod context;
pub mod error;
pub mod evaluation;
pub mod extract;
pub mod model;
pub mod schema;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub use context::{
    AccessDecision, BuiltContext, ContextBuildError, ContextBuilder, ContextItem, ContextRole,
    TrustLevel,
};
pub use error::{
    ContentFilterReason, ModelError, RetryDisposition, SchemaIssue, StructuredOutputError,
    TransportKind,
};
pub use evaluation::{
    ActualOutcome, EVALUATION_CASES, EVALUATION_MANIFEST_SRC, EvaluationCase, ExpectedOutcome,
    baseline_accepts, baseline_task_matches, classify_target, is_illegal_acceptance,
    target_matches,
};
pub use extract::extract_json;
pub use model::{
    ADAPTER_TOTAL_TOKEN_LIMIT, CONCEPT_TIER, CapabilitySet, CapabilitySupport, CompletionRequest,
    CompletionResponse, ConformanceStatus, ContentFilterStatus, FinishReason,
    MAX_MODEL_TIMEOUT_SECS, MIN_MODEL_TIMEOUT_SECS, ModelCallMetadata, ModelCapabilities,
    ModelClient, ModelFingerprint, PROVIDER_CONTEXT_TOKEN_LIMIT, PROVIDER_OUTPUT_TOKEN_LIMIT,
    Usage, VERIFIED_MODEL_FAMILY,
};
pub use schema::{SchemaCompileError, SchemaValidator};

/// The demo schema, embedded at compile time so the binary needs no fixture
/// directory at runtime.
pub const MEETING_SCHEMA_SRC: &str = include_str!("../fixtures/meeting_schema.json");
pub const PROMPT_ID: &str = "structured-output.system";
pub const PROMPT_VERSION: &str = "1";
pub const PROMPT_SHA256: &str = "7a6666b16fe63e45596ded24ee341b774773552f867dea96a92a1746a3adda6b";
pub const CONTEXT_POLICY_VERSION: &str = "1";
pub const SYSTEM_PROMPT: &str = "\
You extract structured meeting minutes from a free-form transcript.

Return ONLY a single JSON object (no prose, no Markdown fence) with this shape:
{
  \"title\": string,
  \"date\": \"YYYY-MM-DD\",
  \"attendees\": [string, ...],
  \"decisions\": [string, ...],
  \"action_items\": [
    { \"owner\": string, \"task\": string, \"due_date\": \"YYYY-MM-DD\" | null }
  ]
}
Do not add fields. If a due date is unknown, use null.
";

/// Typed model of a meeting minute, matching `fixtures/meeting_schema.json`.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct MeetingMinutes {
    pub title: String,
    pub date: String,
    pub attendees: Vec<String>,
    pub decisions: Vec<String>,
    pub action_items: Vec<ActionItem>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ActionItem {
    pub owner: String,
    pub task: String,
    pub due_date: Option<String>,
}

/// One-shot pipeline:
/// 1. Call the model once.
/// 2. Extract a JSON object from its raw text.
/// 3. Validate against `schema_value`.
/// 4. Deserialize to `T`.
///
/// On success returns `(typed, fingerprint)` so the caller can record the
/// model fingerprint alongside the parsed payload.
pub async fn extract_structured<T>(
    client: &dyn ModelClient,
    schema: &SchemaValidator,
    request: CompletionRequest,
) -> Result<(T, ModelCallMetadata), StructuredOutputError>
where
    T: serde::de::DeserializeOwned,
{
    let started = std::time::Instant::now();
    let resp = client.complete(request).await?;
    let value: Value = extract_json(&resp.content)?;
    let typed: T = schema.validate_and_deserialize(value)?;
    Ok((
        typed,
        ModelCallMetadata {
            fingerprint: resp.fingerprint,
            usage: resp.usage,
            finish_reason: resp.finish_reason,
            content_filter: resp.content_filter,
            latency: started.elapsed(),
        },
    ))
}
