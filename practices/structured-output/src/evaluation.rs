//! Shared deterministic dataset and scorers used by tests and the evaluator.

use crate::{MeetingMinutes, SchemaValidator, StructuredOutputError, extract_json};

pub const EVALUATION_MANIFEST_SRC: &str = include_str!("../fixtures/evaluation_manifest.json");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpectedOutcome {
    Success,
    Empty,
    Extraction,
    Ambiguous,
    Schema,
}

impl ExpectedOutcome {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Empty => "empty",
            Self::Extraction => "extraction",
            Self::Ambiguous => "ambiguous",
            Self::Schema => "schema",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActualOutcome {
    Success,
    Empty,
    Extraction,
    Ambiguous,
    Schema,
    OtherFailure,
}

impl ActualOutcome {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Empty => "empty",
            Self::Extraction => "extraction",
            Self::Ambiguous => "ambiguous",
            Self::Schema => "schema",
            Self::OtherFailure => "other_failure",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EvaluationCase {
    pub name: &'static str,
    pub path: &'static str,
    pub raw: &'static str,
    pub expected: ExpectedOutcome,
}

pub const EVALUATION_CASES: &[EvaluationCase] = &[
    EvaluationCase {
        name: "ok",
        path: "fixtures/mock_responses/ok.json",
        raw: include_str!("../fixtures/mock_responses/ok.json"),
        expected: ExpectedOutcome::Success,
    },
    EvaluationCase {
        name: "fenced",
        path: "fixtures/mock_responses/fenced.txt",
        raw: include_str!("../fixtures/mock_responses/fenced.txt"),
        expected: ExpectedOutcome::Success,
    },
    EvaluationCase {
        name: "chatty",
        path: "fixtures/mock_responses/chatty.txt",
        raw: include_str!("../fixtures/mock_responses/chatty.txt"),
        expected: ExpectedOutcome::Success,
    },
    EvaluationCase {
        name: "empty",
        path: "fixtures/mock_responses/empty.txt",
        raw: include_str!("../fixtures/mock_responses/empty.txt"),
        expected: ExpectedOutcome::Empty,
    },
    EvaluationCase {
        name: "truncated",
        path: "fixtures/mock_responses/truncated.txt",
        raw: include_str!("../fixtures/mock_responses/truncated.txt"),
        expected: ExpectedOutcome::Extraction,
    },
    EvaluationCase {
        name: "multi_json",
        path: "fixtures/mock_responses/multi_json.txt",
        raw: include_str!("../fixtures/mock_responses/multi_json.txt"),
        expected: ExpectedOutcome::Ambiguous,
    },
    EvaluationCase {
        name: "missing_field",
        path: "fixtures/mock_responses/missing_field.json",
        raw: include_str!("../fixtures/mock_responses/missing_field.json"),
        expected: ExpectedOutcome::Schema,
    },
    EvaluationCase {
        name: "type_mismatch",
        path: "fixtures/mock_responses/type_mismatch.json",
        raw: include_str!("../fixtures/mock_responses/type_mismatch.json"),
        expected: ExpectedOutcome::Schema,
    },
    EvaluationCase {
        name: "missing_due_date",
        path: "fixtures/mock_responses/missing_due_date.json",
        raw: include_str!("../fixtures/mock_responses/missing_due_date.json"),
        expected: ExpectedOutcome::Schema,
    },
    EvaluationCase {
        name: "extra_field",
        path: "fixtures/mock_responses/extra_field.json",
        raw: include_str!("../fixtures/mock_responses/extra_field.json"),
        expected: ExpectedOutcome::Schema,
    },
];

pub fn classify_target(raw: &str, validator: &SchemaValidator) -> ActualOutcome {
    let result = extract_json(raw)
        .and_then(|value| validator.validate_and_deserialize::<MeetingMinutes>(value));
    match result {
        Ok(_) => ActualOutcome::Success,
        Err(StructuredOutputError::EmptyResponse) => ActualOutcome::Empty,
        Err(StructuredOutputError::JsonExtractionFailed) => ActualOutcome::Extraction,
        Err(StructuredOutputError::MultipleJsonCandidates { .. }) => ActualOutcome::Ambiguous,
        Err(StructuredOutputError::SchemaValidationFailed { .. }) => ActualOutcome::Schema,
        Err(_) => ActualOutcome::OtherFailure,
    }
}

pub const fn target_matches(actual: ActualOutcome, expected: ExpectedOutcome) -> bool {
    matches!(
        (actual, expected),
        (ActualOutcome::Success, ExpectedOutcome::Success)
            | (ActualOutcome::Empty, ExpectedOutcome::Empty)
            | (ActualOutcome::Extraction, ExpectedOutcome::Extraction)
            | (ActualOutcome::Ambiguous, ExpectedOutcome::Ambiguous)
            | (ActualOutcome::Schema, ExpectedOutcome::Schema)
    )
}

pub fn baseline_accepts(raw: &str) -> bool {
    serde_json::from_str::<MeetingMinutes>(raw).is_ok()
}

pub const fn baseline_task_matches(accepted: bool, expected: ExpectedOutcome) -> bool {
    accepted == matches!(expected, ExpectedOutcome::Success)
}

pub const fn is_illegal_acceptance(actual: ActualOutcome, expected: ExpectedOutcome) -> bool {
    !matches!(expected, ExpectedOutcome::Success) && matches!(actual, ActualOutcome::Success)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MEETING_SCHEMA_SRC;
    use serde_json::Value;

    #[test]
    fn every_registered_case_matches_and_invalid_cases_are_rejected() {
        let schema: Value = serde_json::from_str(MEETING_SCHEMA_SRC).unwrap();
        let validator = SchemaValidator::compile(&schema).unwrap();
        assert_eq!(EVALUATION_CASES.len(), 10);
        for case in EVALUATION_CASES {
            let actual = classify_target(case.raw, &validator);
            assert!(target_matches(actual, case.expected), "case {}", case.name);
            assert!(
                !is_illegal_acceptance(actual, case.expected),
                "case {}",
                case.name
            );
        }
    }
}
