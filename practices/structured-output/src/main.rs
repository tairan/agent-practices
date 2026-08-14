//! Demo entry. Picks a `ModelClient` from env vars (see the README mode
//! resolution table), runs the pipeline once against the meeting-minutes
//! schema, and prints the typed result plus model-call metadata.

use std::process::ExitCode;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::Value;
use tracing::{error, info, warn};

use structured_output::{
    AccessDecision, CompletionRequest, ContextBuilder, ContextItem, ContextRole,
    MAX_MODEL_TIMEOUT_SECS, MEETING_SCHEMA_SRC, MIN_MODEL_TIMEOUT_SECS, MeetingMinutes,
    ModelCallMetadata, PROMPT_ID, PROMPT_VERSION, SYSTEM_PROMPT, SchemaValidator,
    StructuredOutputError, TrustLevel, VERIFIED_MODEL_FAMILY, extract_structured,
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
    SafeDefault,
}

fn resolve_mode() -> Result<Mode, String> {
    let mode = std::env::var("STRUCTURED_OUTPUT_MODE").ok();
    let key = std::env::var("GEMINI_API_KEY")
        .ok()
        .filter(|s| !s.trim().is_empty());
    let base_url_is_set = std::env::var_os("GEMINI_BASE_URL").is_some();
    let family = std::env::var("GEMINI_MODEL_FAMILY").ok();
    let timeout_value = std::env::var("GEMINI_TIMEOUT_SECS").ok();
    resolve_mode_values(
        mode.as_deref(),
        key,
        base_url_is_set,
        family,
        timeout_value.as_deref(),
    )
}

fn resolve_mode_values(
    mode: Option<&str>,
    key: Option<String>,
    base_url_is_set: bool,
    family: Option<String>,
    timeout_value: Option<&str>,
) -> Result<Mode, String> {
    match mode {
        Some("mock") => Ok(Mode::Mock {
            reason: MockReason::ExplicitMock,
        }),
        Some("real") => match key {
            Some(api_key) => Ok(Mode::Real {
                cfg: build_real_config(api_key, base_url_is_set, timeout_value)?,
                family: family_from_value(family)?,
            }),
            None => Err(
                "STRUCTURED_OUTPUT_MODE=real but GEMINI_API_KEY is unset; refusing to call real provider"
                    .into(),
            ),
        },
        Some(_) => Err("invalid STRUCTURED_OUTPUT_MODE; expected 'mock' or 'real'".into()),
        None => Ok(Mode::Mock {
            reason: MockReason::SafeDefault,
        }),
    }
}

fn build_real_config(
    api_key: String,
    base_url_is_set: bool,
    timeout_value: Option<&str>,
) -> Result<GeminiConfig, String> {
    if base_url_is_set {
        return Err("GEMINI_BASE_URL is not configurable when using a Gemini API key".into());
    }
    let timeout_secs = match timeout_value {
        Some(value) => value
            .parse::<u64>()
            .map_err(|_| "GEMINI_TIMEOUT_SECS must be an integer from 1 through 120")?,
        None => 30,
    };
    if !(MIN_MODEL_TIMEOUT_SECS..=MAX_MODEL_TIMEOUT_SECS).contains(&timeout_secs) {
        return Err("GEMINI_TIMEOUT_SECS must be an integer from 1 through 120".into());
    }
    Ok(GeminiConfig {
        api_key,
        timeout: Duration::from_secs(timeout_secs),
    })
}

fn family_from_value(family: Option<String>) -> Result<String, String> {
    let family = family.unwrap_or_else(|| VERIFIED_MODEL_FAMILY.into());
    if family == VERIFIED_MODEL_FAMILY {
        Ok(family)
    } else {
        Err("GEMINI_MODEL_FAMILY is outside the verified capability class".into())
    }
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
) -> Result<(MeetingMinutes, ModelCallMetadata), StructuredOutputError> {
    let client = MockClient::with_scenario(scenario, content);
    extract_structured::<MeetingMinutes>(&client, validator, request).await
}

async fn run_with_real(
    cfg: GeminiConfig,
    validator: &SchemaValidator,
    request: CompletionRequest,
) -> Result<(MeetingMinutes, ModelCallMetadata), Box<dyn std::error::Error>> {
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
    let (parsed, metadata) = match mode {
        Mode::Mock { reason } => {
            match reason {
                MockReason::ExplicitMock => {
                    info!(
                        mode = "mock",
                        source = "explicit",
                        "running with MockClient"
                    );
                }
                MockReason::SafeDefault => {
                    eprintln!(
                        "structured-output: defaulting to MockClient; set STRUCTURED_OUTPUT_MODE=real and GEMINI_API_KEY to opt in to the fixed Gemini endpoint."
                    );
                    info!(mode = "mock", source = "safe-default", "using safe default");
                }
            }
            let request = build_request("gemini-3.5-flash")?;
            run_with_mock("ok", MOCK_OK_FIXTURE, &validator, request).await?
        }
        Mode::Real { cfg, family: f } => {
            info!(
                mode = "real",
                family = %f,
                "running with GeminiOpenAiClient"
            );
            warn!("real mode hits the network; core tests remain fixture-only by contract");
            let request = build_request(&f)?;
            run_with_real(cfg, &validator, request).await?
        }
    };

    info!(
        provider = metadata.fingerprint.provider,
        requested_family = %metadata.fingerprint.requested_family,
        response_model = %metadata.fingerprint.response_model,
        response_model_missing = metadata.fingerprint.response_model_missing,
        api_version = ?metadata.fingerprint.api_version,
        capability_tier = metadata.fingerprint.capability_tier,
        usage = ?metadata.usage,
        usage_source = if metadata.usage.is_some() { "reported" } else { "unknown" },
        finish_reason = ?metadata.finish_reason,
        content_filter = ?metadata.content_filter,
        latency_ms = metadata.latency.as_millis(),
        prompt_id = PROMPT_ID,
        prompt_version = PROMPT_VERSION,
        "model call metadata"
    );

    println!("--- parsed MeetingMinutes ---");
    println!("{}", serde_json::to_string_pretty(&parsed)?);
    println!("--- fingerprint ---");
    println!("provider         = {}", metadata.fingerprint.provider);
    println!(
        "requested_family = {}",
        metadata.fingerprint.requested_family
    );
    println!("response_model   = {}", metadata.fingerprint.response_model);
    println!(
        "model_id_missing  = {}",
        metadata.fingerprint.response_model_missing
    );
    println!("api_version      = {:?}", metadata.fingerprint.api_version);
    println!(
        "capability_tier  = {}",
        metadata.fingerprint.capability_tier
    );
    println!("usage            = {:?}", metadata.usage);
    println!("finish_reason    = {:?}", metadata.finish_reason);
    println!("content_filter   = {:?}", metadata.content_filter);
    println!("latency_ms       = {}", metadata.latency.as_millis());

    Ok(())
}

fn build_request(family: &str) -> Result<CompletionRequest, structured_output::ContextBuildError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let context = ContextBuilder::new("structured-output-teaching", now, 4096)
        .add(ContextItem::new(
            ContextRole::System,
            SYSTEM_PROMPT,
            "repo://structured-output/prompt/system",
            format!("{PROMPT_ID}@{PROMPT_VERSION}"),
            TrustLevel::TrustedInstruction,
            "structured-output-teaching",
            AccessDecision::allowed("project-local-static-instruction"),
            now,
            PROMPT_VERSION,
            None,
            "required output contract",
        ))
        .add(ContextItem::new(
            ContextRole::User,
            format!("Meeting transcript:\n{MEETING_INPUT}"),
            "repo://structured-output/fixtures/meeting_input.txt",
            "fixtures/meeting_input.txt",
            TrustLevel::UntrustedData,
            "structured-output-teaching",
            AccessDecision::allowed("public-synthetic-fixture"),
            now,
            "fixture-v1",
            None,
            "single task input",
        ))
        .build()?;
    Ok(CompletionRequest {
        model_family: family.to_string(),
        context,
        temperature: Some(0.0),
        max_tokens: Some(1024),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};
    use structured_output::PROMPT_SHA256;

    #[test]
    fn prompt_contract_is_versioned_and_user_data_stays_untrusted() {
        assert_eq!(PROMPT_ID, "structured-output.system");
        assert_eq!(PROMPT_VERSION, "1");
        assert_eq!(
            format!("{:x}", Sha256::digest(SYSTEM_PROMPT)),
            PROMPT_SHA256
        );
        let request = build_request("gemini-3.5-flash").unwrap();
        assert_eq!(request.context.items().len(), 2);
        assert_eq!(
            request.context.items()[0].trust_level(),
            TrustLevel::TrustedInstruction
        );
        assert_eq!(
            request.context.items()[1].trust_level(),
            TrustLevel::UntrustedData
        );
        assert!(request.context.user().starts_with("Meeting transcript:\n"));
        assert!(!request.context.system().contains(MEETING_INPUT));
    }

    #[test]
    fn real_mode_requires_explicit_opt_in_and_bound_configuration() {
        assert!(matches!(
            resolve_mode_values(None, Some("key".into()), false, None, None).unwrap(),
            Mode::Mock {
                reason: MockReason::SafeDefault
            }
        ));
        assert!(resolve_mode_values(Some("real"), Some("key".into()), true, None, None).is_err());
        assert!(
            resolve_mode_values(
                Some("real"),
                Some("key".into()),
                false,
                Some("unknown-model".into()),
                None,
            )
            .is_err()
        );
        for timeout in ["not-a-number", "0", "121"] {
            assert!(
                resolve_mode_values(Some("real"), Some("key".into()), false, None, Some(timeout),)
                    .is_err()
            );
        }
    }
}
