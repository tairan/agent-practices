//! `structured-output` — practice #1 of agent-practices.
//!
//! Scope (intentionally narrow): take one model call, locate a JSON object in
//! the response, validate it against a JSON Schema (Draft 2020-12), and either
//! return a typed value or a structured error. No retries, no reflection, no
//! Agent loop — see README §1 for the explicit non-goals.

pub mod error;
pub mod extract;
pub mod model;
pub mod schema;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub use error::{ModelError, SchemaIssue, StructuredOutputError};
pub use extract::extract_json;
pub use model::{
    CONCEPT_TIER, CompletionRequest, CompletionResponse, ModelClient, ModelFingerprint, Usage,
};
pub use schema::{SchemaCompileError, SchemaValidator};

/// The demo schema, embedded at compile time so the binary needs no fixture
/// directory at runtime.
pub const MEETING_SCHEMA_SRC: &str = include_str!("../fixtures/meeting_schema.json");

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
/// §4.4 invariant-#1 fingerprint alongside the parsed payload.
pub async fn extract_structured<T>(
    client: &dyn ModelClient,
    schema: &SchemaValidator,
    request: CompletionRequest,
) -> Result<(T, ModelFingerprint), StructuredOutputError>
where
    T: serde::de::DeserializeOwned,
{
    let resp = client.complete(request).await?;
    let value: Value = extract_json(&resp.content)?;
    let typed: T = schema.validate_and_deserialize(value)?;
    Ok((typed, resp.fingerprint))
}
