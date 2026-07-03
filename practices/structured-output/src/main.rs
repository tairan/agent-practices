//! Demo entry. Picks a `ModelClient` from env vars (see README §9 mode
//! resolution table), runs the pipeline once against the meeting-minutes
//! schema, and prints the typed result plus the §4.4 model fingerprint.

use std::process::ExitCode;
use std::time::Duration;

use serde_json::Value;
use tracing::{error, info, warn};

use structured_output::{
    CompletionRequest, MEETING_SCHEMA_SRC, MeetingMinutes, ModelFingerprint, SchemaValidator,
    StructuredOutputError, extract_structured,
    model::gemini_openai::{GeminiConfig, GeminiOpenAiClient},
    model::mock::MockClient,
};

const MEETING_INPUT: &str = include_str!("../fixtures/meeting_input.txt");
const MOCK_OK_FIXTURE: &str = include_str!("../fixtures/mock_responses/ok.json");

fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            error!(error = %e, "failed to build tokio runtime");
            return ExitCode::from(1);
        }
    };

    rt.block_on(async {
        match run().await {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                error!(error = %e, "demo failed");
                ExitCode::from(1)
            }
        }
    })
}

/// Resolved demo mode after env-var inspection.
enum Mode {
    Mock { reason: MockReason },
    Real { cfg: GeminiConfig, family: String },
}

enum MockReason {
    ExplicitMock,
    AutoFallbackNoKey,
}

fn resolve_mode() -> Result<Mode, String> {
    let mode = std::env::var("STRUCTURED_OUTPUT_MODE").ok();
    let key = std::env::var("GEMINI_API_KEY")
        .ok()
        .filter(|s| !s.trim().is_empty());

    match mode.as_deref() {
        Some("mock") => Ok(Mode::Mock {
            reason: MockReason::ExplicitMock,
        }),
        Some("real") => match key {
            Some(api_key) => Ok(Mode::Real {
                cfg: build_real_config(api_key)?,
                family: family_from_env(),
            }),
            None => Err(
                "STRUCTURED_OUTPUT_MODE=real but GEMINI_API_KEY is unset; refusing to call real provider"
                    .into(),
            ),
        },
        Some(other) => Err(format!(
            "invalid STRUCTURED_OUTPUT_MODE={other:?}; expected 'mock' or 'real'"
        )),
        None => match key {
            Some(api_key) => Ok(Mode::Real {
                cfg: build_real_config(api_key)?,
                family: family_from_env(),
            }),
            None => Ok(Mode::Mock {
                reason: MockReason::AutoFallbackNoKey,
            }),
        },
    }
}

fn build_real_config(api_key: String) -> Result<GeminiConfig, String> {
    let base_url = std::env::var("GEMINI_BASE_URL")
        .unwrap_or_else(|_| "https://generativelanguage.googleapis.com/v1beta/openai/".into());
    let timeout_secs: u64 = std::env::var("GEMINI_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(30);
    Ok(GeminiConfig {
        base_url,
        api_key,
        timeout: Duration::from_secs(timeout_secs),
    })
}

fn family_from_env() -> String {
    std::env::var("GEMINI_MODEL_FAMILY").unwrap_or_else(|_| "gemini-3.5-flash".into())
}

/// Build a boxed `MockClient` typed as `Box<dyn ModelClient>`.
///
/// Wrapping the `Box::new` in a helper whose return type is the trait object
/// puts the unsized coercion on a `return` expression — a position both rustc
/// and rust-analyzer handle uniformly. Inline `Box::new(...)` inside a `match`
/// arm or `let` binding works for rustc but trips rust-analyzer's type
/// inference (see commit history for the long version).
async fn run_with_mock(
    scenario: &'static str,
    content: &'static str,
    validator: &SchemaValidator,
    request: CompletionRequest,
) -> Result<(MeetingMinutes, ModelFingerprint), StructuredOutputError> {
    let client = MockClient::with_scenario(scenario, content);
    extract_structured::<MeetingMinutes>(&client, validator, request).await
}

async fn run_with_real(
    cfg: GeminiConfig,
    validator: &SchemaValidator,
    request: CompletionRequest,
) -> Result<(MeetingMinutes, ModelFingerprint), Box<dyn std::error::Error>> {
    let client = GeminiOpenAiClient::new(cfg)?;
    Ok(extract_structured::<MeetingMinutes>(&client, validator, request).await?)
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mode = resolve_mode().map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;

    let schema: Value = serde_json::from_str(MEETING_SCHEMA_SRC)?;
    let validator = SchemaValidator::compile(&schema)?;

    // Each match arm holds the concrete client type and runs the pipeline
    // itself. `extract_structured` takes `&dyn ModelClient` and the deref
    // coercion happens at the call site — rust-analyzer handles this form
    // correctly, unlike `Box<dyn ModelClient>` produced cross-arm.
    let (parsed, fingerprint) = match mode {
        Mode::Mock { reason } => {
            match reason {
                MockReason::ExplicitMock => {
                    info!(
                        mode = "mock",
                        source = "explicit",
                        "running with MockClient"
                    );
                }
                MockReason::AutoFallbackNoKey => {
                    eprintln!(
                        "structured-output: GEMINI_API_KEY not set; falling back to MockClient.\n\
                         set GEMINI_API_KEY to run against the real Gemini OpenAI-compatible endpoint."
                    );
                    info!(mode = "mock", source = "fallback", "no api key → mock");
                }
            }
            let request = build_request("gemini-3.5-flash");
            run_with_mock("ok", MOCK_OK_FIXTURE, &validator, request).await?
        }
        Mode::Real { cfg, family: f } => {
            info!(
                mode = "real",
                family = %f,
                "running with GeminiOpenAiClient"
            );
            warn!("real mode hits the network; the test suite never does — see AGENTS.md §4 / §9");
            let request = build_request(&f);
            run_with_real(cfg, &validator, request).await?
        }
    };

    info!(
        provider = fingerprint.provider,
        requested_family = %fingerprint.requested_family,
        response_model = %fingerprint.response_model,
        api_version = ?fingerprint.api_version,
        capability_tier = fingerprint.capability_tier,
        "model fingerprint (AGENTS.md §4.4 invariant #1)"
    );

    println!("--- parsed MeetingMinutes ---");
    println!("{}", serde_json::to_string_pretty(&parsed)?);
    println!("--- fingerprint ---");
    println!("provider         = {}", fingerprint.provider);
    println!("requested_family = {}", fingerprint.requested_family);
    println!("response_model   = {}", fingerprint.response_model);
    println!("api_version      = {:?}", fingerprint.api_version);
    println!("capability_tier  = {}", fingerprint.capability_tier);

    Ok(())
}

fn build_request(family: &str) -> CompletionRequest {
    CompletionRequest {
        model_family: family.to_string(),
        system: Some(SYSTEM_PROMPT.into()),
        user: format!("Meeting transcript:\n{MEETING_INPUT}"),
        temperature: Some(0.0),
        max_tokens: Some(1024),
    }
}

const SYSTEM_PROMPT: &str = "\
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
