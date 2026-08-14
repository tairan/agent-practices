//! JSON Schema (Draft 2020-12) validation, then typed deserialize.
//!
//! Two-stage by design, following the explicit model-output type contract:
//!
//! 1. Validate the JSON value against the schema — collect *all* issues, not
//!    just the first, so callers get actionable feedback in one pass.
//! 2. Only on success, `serde_json::from_value::<T>` into the target type.
//!    A failure here is a *bug*: schema and Rust type are out of sync. We
//!    flag it as a distinct error variant so diagnosis is unambiguous.

use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::error::{SchemaIssue, StructuredOutputError};

pub struct SchemaValidator {
    inner: jsonschema::Validator,
}

#[derive(Debug, thiserror::Error)]
#[error("schema compile error: {0}")]
pub struct SchemaCompileError(String);

impl SchemaValidator {
    /// Compile a schema once; reuse the validator across calls.
    pub fn compile(schema: &Value) -> Result<Self, SchemaCompileError> {
        let inner =
            jsonschema::validator_for(schema).map_err(|e| SchemaCompileError(e.to_string()))?;
        Ok(Self { inner })
    }

    /// Validate-then-deserialize. Returns the typed value on success;
    /// otherwise a [`StructuredOutputError`] describing exactly which stage
    /// failed and where.
    pub fn validate_and_deserialize<T: DeserializeOwned>(
        &self,
        value: Value,
    ) -> Result<T, StructuredOutputError> {
        let issues: Vec<SchemaIssue> = self
            .inner
            .iter_errors(&value)
            .map(|error| {
                let instance_path = error.instance_path().to_string();
                let schema_path = error.schema_path().to_string();
                let code = schema_path
                    .rsplit('/')
                    .find(|segment| !segment.is_empty())
                    .unwrap_or("validation")
                    .to_string();
                SchemaIssue {
                    instance_depth: instance_path
                        .split('/')
                        .filter(|segment| !segment.is_empty())
                        .count(),
                    schema_path,
                    code,
                }
            })
            .collect();

        if !issues.is_empty() {
            return Err(StructuredOutputError::SchemaValidationFailed { issues });
        }

        serde_json::from_value::<T>(value)
            .map_err(|_| StructuredOutputError::TypedDeserializeFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use serde_json::json;

    const SCHEMA_SRC: &str = include_str!("../fixtures/meeting_schema.json");

    #[derive(Debug, Deserialize, PartialEq)]
    struct ActionItem {
        owner: String,
        task: String,
        due_date: Option<String>,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct MeetingMinutes {
        title: String,
        date: String,
        attendees: Vec<String>,
        decisions: Vec<String>,
        action_items: Vec<ActionItem>,
    }

    fn validator() -> SchemaValidator {
        let schema: Value = serde_json::from_str(SCHEMA_SRC).unwrap();
        SchemaValidator::compile(&schema).unwrap()
    }

    #[test]
    fn schema_ok_full_payload() {
        let v = validator();
        let payload = json!({
            "title": "Q3 Planning",
            "date": "2026-06-24",
            "attendees": ["alice", "bob"],
            "decisions": ["adopt feature X"],
            "action_items": [
                { "owner": "alice", "task": "draft spec", "due_date": "2026-07-01" },
                { "owner": "bob",   "task": "review",     "due_date": null }
            ]
        });
        let parsed: MeetingMinutes = v.validate_and_deserialize(payload).unwrap();
        assert_eq!(parsed.title, "Q3 Planning");
        assert_eq!(parsed.action_items.len(), 2);
    }

    #[test]
    fn schema_missing_required_field() {
        let v = validator();
        let payload = json!({
            "title": "M",
            "date": "2026-06-24",
            "attendees": ["a"],
            "decisions": []
            // action_items missing
        });
        let err = v
            .validate_and_deserialize::<MeetingMinutes>(payload)
            .unwrap_err();
        match err {
            StructuredOutputError::SchemaValidationFailed { issues } => {
                assert!(
                    issues.iter().any(|i| i.code == "required"),
                    "expected a required-field issue, got: {issues:?}"
                );
            }
            other => panic!("expected SchemaValidationFailed, got {other:?}"),
        }
    }

    #[test]
    fn schema_type_mismatch() {
        let v = validator();
        let payload = json!({
            "title": "M",
            "date": "2026-06-24",
            "attendees": "alice",          // should be array
            "decisions": [],
            "action_items": []
        });
        let err = v
            .validate_and_deserialize::<MeetingMinutes>(payload)
            .unwrap_err();
        assert!(matches!(
            err,
            StructuredOutputError::SchemaValidationFailed { .. }
        ));
    }

    #[test]
    fn validation_issues_never_retain_model_values() {
        let canary = "SECRET_MODEL_VALUE_CANARY_9f31";
        for schema in [
            json!({"type":"object","properties":{"value":{"type":"integer"}},"required":["value"]}),
            json!({"type":"object","properties":{"value":{"type":"string","pattern":"^allowed$"}},"required":["value"]}),
            json!({"type":"object","properties":{"value":{"enum":["allowed"]}},"required":["value"]}),
        ] {
            let validator = SchemaValidator::compile(&schema).unwrap();
            let error = validator
                .validate_and_deserialize::<Value>(json!({"value": canary}))
                .unwrap_err();
            let StructuredOutputError::SchemaValidationFailed { issues } = &error else {
                panic!("expected schema validation failure");
            };
            assert!(!error.to_string().contains(canary));
            assert!(!format!("{error:?}").contains(canary));
            assert!(!serde_json::to_string(issues).unwrap().contains(canary));
        }
    }

    #[test]
    fn schema_rejects_additional_properties() {
        let v = validator();
        let payload = json!({
            "title": "M",
            "date": "2026-06-24",
            "attendees": ["a"],
            "decisions": [],
            "action_items": [],
            "secret_field": "not allowed"
        });
        let err = v
            .validate_and_deserialize::<MeetingMinutes>(payload)
            .unwrap_err();
        assert!(matches!(
            err,
            StructuredOutputError::SchemaValidationFailed { .. }
        ));
    }

    #[test]
    fn schema_array_item_invalid() {
        let v = validator();
        let payload = json!({
            "title": "M",
            "date": "2026-06-24",
            "attendees": ["a"],
            "decisions": [],
            "action_items": [
                { "owner": "alice" }  // missing required "task"
            ]
        });
        let err = v
            .validate_and_deserialize::<MeetingMinutes>(payload)
            .unwrap_err();
        match err {
            StructuredOutputError::SchemaValidationFailed { issues } => {
                assert!(
                    issues
                        .iter()
                        .any(|i| i.instance_depth == 2 && i.code == "required")
                );
            }
            other => panic!("expected SchemaValidationFailed, got {other:?}"),
        }
    }

    #[test]
    fn schema_date_pattern_violation() {
        let v = validator();
        let payload = json!({
            "title": "M",
            "date": "yesterday",
            "attendees": ["a"],
            "decisions": [],
            "action_items": []
        });
        let err = v
            .validate_and_deserialize::<MeetingMinutes>(payload)
            .unwrap_err();
        assert!(matches!(
            err,
            StructuredOutputError::SchemaValidationFailed { .. }
        ));
    }

    #[test]
    fn schema_collects_all_issues_at_once() {
        let v = validator();
        let payload = json!({
            "title": "",                   // minLength violation
            "date": "bad",                 // pattern violation
            "attendees": [],               // minItems violation
            "decisions": [],
            "action_items": []
        });
        let err = v
            .validate_and_deserialize::<MeetingMinutes>(payload)
            .unwrap_err();
        match err {
            StructuredOutputError::SchemaValidationFailed { issues } => {
                assert!(
                    issues.len() >= 2,
                    "expected ≥2 issues, got {}: {issues:?}",
                    issues.len()
                );
            }
            other => panic!("expected SchemaValidationFailed, got {other:?}"),
        }
    }

    #[test]
    fn schema_and_rust_type_mismatch_is_classified() {
        #[derive(Debug, Deserialize)]
        #[allow(dead_code)]
        struct WrongRustType {
            field_not_in_schema: String,
        }

        let v = validator();
        let payload = json!({
            "title": "M",
            "date": "2026-06-24",
            "attendees": ["a"],
            "decisions": [],
            "action_items": []
        });
        assert!(matches!(
            v.validate_and_deserialize::<WrongRustType>(payload),
            Err(StructuredOutputError::TypedDeserializeFailed)
        ));
    }
}
