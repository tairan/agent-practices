//! End-to-end pipeline tests using only `MockClient` — no network, no env vars.
//!
//! Each mock fixture exercises a distinct branch of the pipeline:
//! ok / fenced / chatty → success
//! truncated / empty → extraction failure
//! multi_json → ambiguity
//! missing_field / type_mismatch → schema validation failure

use serde_json::Value;
use structured_output::model::mock::MockClient;
use structured_output::{
    AccessDecision, CompletionRequest, ContextBuilder, ContextItem, ContextRole,
    MEETING_SCHEMA_SRC, MeetingMinutes, SchemaValidator, StructuredOutputError, TrustLevel,
    extract_structured,
};

fn validator() -> SchemaValidator {
    let schema: Value = serde_json::from_str(MEETING_SCHEMA_SRC).unwrap();
    SchemaValidator::compile(&schema).unwrap()
}

fn request() -> CompletionRequest {
    CompletionRequest {
        model_family: "gemini-3.5-flash".into(),
        context: ContextBuilder::new("test", 1, 100)
            .add(ContextItem::new(
                ContextRole::System,
                "You return JSON only.",
                "test://system",
                "test",
                TrustLevel::TrustedInstruction,
                "test",
                AccessDecision::allowed("test"),
                1,
                "v1",
                None,
                "test",
            ))
            .add(ContextItem::new(
                ContextRole::User,
                "Summarize the meeting.",
                "test://user",
                "test",
                TrustLevel::UntrustedData,
                "test",
                AccessDecision::allowed("test"),
                1,
                "v1",
                None,
                "test",
            ))
            .build()
            .unwrap(),
        temperature: Some(0.0),
        max_tokens: Some(1024),
    }
}

#[tokio::test]
async fn ok_response_yields_typed_value() {
    let raw = include_str!("../fixtures/mock_responses/ok.json");
    let client = MockClient::with_scenario("ok", raw);
    let v = validator();
    let (parsed, metadata): (MeetingMinutes, _) =
        extract_structured(&client, &v, request()).await.unwrap();

    assert_eq!(parsed.title, "Q3 Planning");
    assert_eq!(parsed.attendees, vec!["alice", "bob"]);
    assert_eq!(parsed.action_items.len(), 2);
    assert_eq!(metadata.fingerprint.provider, "mock");
    assert_eq!(metadata.fingerprint.capability_tier, "flash");
    assert_eq!(metadata.fingerprint.response_model, "mock-fixture::ok");
    assert_eq!(
        metadata.finish_reason,
        structured_output::FinishReason::Stop
    );
}

#[tokio::test]
async fn fenced_markdown_is_unwrapped() {
    let raw = include_str!("../fixtures/mock_responses/fenced.txt");
    let client = MockClient::with_scenario("fenced", raw);
    let v = validator();
    let (parsed, _fp): (MeetingMinutes, _) =
        extract_structured(&client, &v, request()).await.unwrap();
    assert_eq!(parsed.title, "Q3 Planning");
}

#[tokio::test]
async fn chatty_prose_is_stripped() {
    let raw = include_str!("../fixtures/mock_responses/chatty.txt");
    let client = MockClient::with_scenario("chatty", raw);
    let v = validator();
    let (parsed, _fp): (MeetingMinutes, _) =
        extract_structured(&client, &v, request()).await.unwrap();
    assert_eq!(parsed.title, "Q3 Planning");
}

#[tokio::test]
async fn empty_response_is_classified_as_empty() {
    let raw = include_str!("../fixtures/mock_responses/empty.txt");
    let client = MockClient::with_scenario("empty", raw);
    let v = validator();
    let err = extract_structured::<MeetingMinutes>(&client, &v, request())
        .await
        .unwrap_err();
    assert!(matches!(err, StructuredOutputError::EmptyResponse));
}

#[tokio::test]
async fn truncated_response_fails_extraction() {
    let raw = include_str!("../fixtures/mock_responses/truncated.txt");
    let client = MockClient::with_scenario("truncated", raw);
    let v = validator();
    let err = extract_structured::<MeetingMinutes>(&client, &v, request())
        .await
        .unwrap_err();
    assert!(matches!(err, StructuredOutputError::JsonExtractionFailed));
}

#[tokio::test]
async fn multi_json_response_is_ambiguous() {
    let raw = include_str!("../fixtures/mock_responses/multi_json.txt");
    let client = MockClient::with_scenario("multi_json", raw);
    let v = validator();
    let err = extract_structured::<MeetingMinutes>(&client, &v, request())
        .await
        .unwrap_err();
    match err {
        StructuredOutputError::MultipleJsonCandidates { count, .. } => assert_eq!(count, 2),
        other => panic!("expected MultipleJsonCandidates, got {other:?}"),
    }
}

#[tokio::test]
async fn missing_required_field_fails_validation() {
    let raw = include_str!("../fixtures/mock_responses/missing_field.json");
    let client = MockClient::with_scenario("missing_field", raw);
    let v = validator();
    let err = extract_structured::<MeetingMinutes>(&client, &v, request())
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        StructuredOutputError::SchemaValidationFailed { .. }
    ));
}

#[tokio::test]
async fn type_mismatch_fails_validation() {
    let raw = include_str!("../fixtures/mock_responses/type_mismatch.json");
    let client = MockClient::with_scenario("type_mismatch", raw);
    let v = validator();
    let err = extract_structured::<MeetingMinutes>(&client, &v, request())
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        StructuredOutputError::SchemaValidationFailed { .. }
    ));
}
