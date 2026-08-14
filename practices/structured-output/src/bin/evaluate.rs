use std::{
    env, fs,
    path::PathBuf,
    process::Command,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use structured_output::{
    ActualOutcome, CONTEXT_POLICY_VERSION, EVALUATION_CASES, EVALUATION_MANIFEST_SRC,
    ExpectedOutcome, MEETING_SCHEMA_SRC, PROMPT_ID, PROMPT_SHA256, PROMPT_VERSION, SYSTEM_PROMPT,
    SchemaValidator, baseline_accepts, baseline_task_matches, classify_target,
    is_illegal_acceptance, target_matches,
};

const COMMAND: &str = "cargo run --locked --bin evaluate -- --output evidence/evaluation.json";

const SOURCE_FILES: &[(&str, &str)] = &[
    (".env.example", include_str!("../../.env.example")),
    (".gitignore", include_str!("../../.gitignore")),
    ("README.md", include_str!("../../README.md")),
    ("src/bin/evaluate.rs", include_str!("evaluate.rs")),
    ("src/context.rs", include_str!("../context.rs")),
    ("src/error.rs", include_str!("../error.rs")),
    ("src/evaluation.rs", include_str!("../evaluation.rs")),
    ("src/extract.rs", include_str!("../extract.rs")),
    ("src/lib.rs", include_str!("../lib.rs")),
    ("src/main.rs", include_str!("../main.rs")),
    ("src/schema.rs", include_str!("../schema.rs")),
    ("src/model/mod.rs", include_str!("../model/mod.rs")),
    ("src/model/mock.rs", include_str!("../model/mock.rs")),
    (
        "src/model/gemini_openai.rs",
        include_str!("../model/gemini_openai.rs"),
    ),
    ("tests/baseline.rs", include_str!("../../tests/baseline.rs")),
    (
        "tests/context_policy.rs",
        include_str!("../../tests/context_policy.rs"),
    ),
    (
        "tests/end_to_end_mock.rs",
        include_str!("../../tests/end_to_end_mock.rs"),
    ),
    ("Cargo.toml", include_str!("../../Cargo.toml")),
    ("Cargo.lock", include_str!("../../Cargo.lock")),
    ("practice.toml", include_str!("../../practice.toml")),
    ("acceptance.toml", include_str!("../../acceptance.toml")),
    (
        "rust-toolchain.toml",
        include_str!("../../rust-toolchain.toml"),
    ),
];

const DATASET_FILES: &[(&str, &str)] = &[
    (
        "fixtures/evaluation_manifest.json",
        include_str!("../../fixtures/evaluation_manifest.json"),
    ),
    (
        "fixtures/meeting_schema.json",
        include_str!("../../fixtures/meeting_schema.json"),
    ),
    (
        "fixtures/meeting_input.txt",
        include_str!("../../fixtures/meeting_input.txt"),
    ),
    (
        "fixtures/mock_responses/ok.json",
        include_str!("../../fixtures/mock_responses/ok.json"),
    ),
    (
        "fixtures/mock_responses/fenced.txt",
        include_str!("../../fixtures/mock_responses/fenced.txt"),
    ),
    (
        "fixtures/mock_responses/chatty.txt",
        include_str!("../../fixtures/mock_responses/chatty.txt"),
    ),
    (
        "fixtures/mock_responses/empty.txt",
        include_str!("../../fixtures/mock_responses/empty.txt"),
    ),
    (
        "fixtures/mock_responses/truncated.txt",
        include_str!("../../fixtures/mock_responses/truncated.txt"),
    ),
    (
        "fixtures/mock_responses/multi_json.txt",
        include_str!("../../fixtures/mock_responses/multi_json.txt"),
    ),
    (
        "fixtures/mock_responses/missing_field.json",
        include_str!("../../fixtures/mock_responses/missing_field.json"),
    ),
    (
        "fixtures/mock_responses/type_mismatch.json",
        include_str!("../../fixtures/mock_responses/type_mismatch.json"),
    ),
    (
        "fixtures/mock_responses/missing_due_date.json",
        include_str!("../../fixtures/mock_responses/missing_due_date.json"),
    ),
    (
        "fixtures/mock_responses/extra_field.json",
        include_str!("../../fixtures/mock_responses/extra_field.json"),
    ),
];

#[derive(Deserialize)]
struct RegisteredManifest {
    schema_version: u32,
    contract_version: u32,
    manifest_version: String,
    cases: Vec<RegisteredCase>,
}

#[derive(Deserialize)]
struct RegisteredCase {
    name: String,
    path: String,
    expected: String,
    illegal: bool,
    baseline_accepts: bool,
    sha256: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct AcceptanceContract {
    schema_version: u32,
    contract_version: u32,
    dataset: DatasetContract,
    prompt: PromptContract,
    baseline: BaselineContract,
    metric: Vec<MetricContract>,
    budgets: BudgetContract,
    command: Vec<CommandContract>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct DatasetContract {
    path: String,
    manifest: String,
    manifest_version: String,
    manifest_sha256: String,
    distinct_cases: usize,
    invalid_cases: usize,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct PromptContract {
    id: String,
    version: String,
    sha256: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct BaselineContract {
    name: String,
    description: String,
    command: String,
    dataset: String,
    scorer: String,
    artifact: String,
    sample_size: usize,
    expected_task_outcome_pass_rate: f64,
    comparison: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct MetricContract {
    name: String,
    direction: String,
    threshold: f64,
    sample_size: usize,
    dataset: String,
    scorer: String,
    command: String,
    artifact: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct BudgetContract {
    p95_latency_ms: f64,
    max_estimated_cost_usd: f64,
    measurement_command: String,
    artifact: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct CommandContract {
    name: String,
    run: String,
    required: bool,
}

fn percentile(sorted_ns: &[u128], percentile: f64) -> f64 {
    let index = ((sorted_ns.len() - 1) as f64 * percentile).ceil() as usize;
    sorted_ns[index] as f64 / 1_000_000.0
}

fn output_path() -> Result<PathBuf, String> {
    let mut args = env::args().skip(1);
    match (args.next().as_deref(), args.next(), args.next()) {
        (Some("--output"), Some(path), None) => Ok(path.into()),
        _ => Err("usage: evaluate --output <path>".into()),
    }
}

fn command_version(program: &str) -> String {
    Command::new(program)
        .args(["--version", "--verbose"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|output| output.trim().to_string())
        .unwrap_or_else(|| "unknown".into())
}

fn sha256_hex(content: &[u8]) -> String {
    format!("{:x}", Sha256::digest(content))
}

fn fingerprint(files: &[(&str, &str)]) -> (String, Vec<Value>) {
    let mut combined = Sha256::new();
    let manifest = files
        .iter()
        .map(|(path, content)| {
            combined.update((path.len() as u64).to_be_bytes());
            combined.update(path.as_bytes());
            combined.update((content.len() as u64).to_be_bytes());
            combined.update(content.as_bytes());
            json!({"path": path, "sha256": sha256_hex(content.as_bytes())})
        })
        .collect();
    (format!("{:x}", combined.finalize()), manifest)
}

fn collect_files(path: &std::path::Path, output: &mut Vec<String>) -> Result<(), std::io::Error> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if matches!(
                path.file_name().and_then(|name| name.to_str()),
                Some("target" | "evidence")
            ) {
                continue;
            }
            collect_files(&path, output)?;
        } else {
            output.push(
                path.to_string_lossy()
                    .replace('\\', "/")
                    .trim_start_matches("./")
                    .to_string(),
            );
        }
    }
    Ok(())
}

fn verify_inventory(
    roots: &[&str],
    expected: impl Iterator<Item = &'static str>,
) -> Result<(), String> {
    let mut actual = Vec::new();
    for root in roots {
        collect_files(std::path::Path::new(root), &mut actual)
            .map_err(|_| "inventory read failed")?;
    }
    actual.sort();
    let mut expected: Vec<String> = expected.map(str::to_owned).collect();
    expected.sort();
    if actual == expected {
        Ok(())
    } else {
        Err("fingerprint inventory does not match repository files".into())
    }
}

fn metric<'a>(contract: &'a AcceptanceContract, name: &str) -> Result<&'a MetricContract, String> {
    let matches: Vec<_> = contract
        .metric
        .iter()
        .filter(|metric| metric.name == name)
        .collect();
    if matches.len() == 1 {
        Ok(matches[0])
    } else {
        Err("acceptance metric missing or duplicated".into())
    }
}

fn metric_passes(value: f64, contract: &MetricContract) -> Result<bool, String> {
    match contract.direction.as_str() {
        "higher" => Ok(value >= contract.threshold),
        "lower" => Ok(value <= contract.threshold),
        _ => Err("acceptance metric has invalid direction".into()),
    }
}

fn load_acceptance_contract() -> Result<AcceptanceContract, String> {
    toml::from_str(include_str!("../../acceptance.toml"))
        .map_err(|_| "invalid acceptance contract".into())
}

fn verify_acceptance_contract(contract: &AcceptanceContract) -> Result<usize, String> {
    if contract.schema_version != 1
        || contract.contract_version != 4
        || contract.dataset.distinct_cases == 0
        || contract.dataset.invalid_cases == 0
        || contract.dataset.invalid_cases > contract.dataset.distinct_cases
        || contract.dataset.path != "fixtures/mock_responses"
        || contract.dataset.manifest != "fixtures/evaluation_manifest.json"
        || contract.prompt.id != PROMPT_ID
        || contract.prompt.version != PROMPT_VERSION
        || contract.prompt.sha256 != PROMPT_SHA256
        || contract.metric.len() != 3
        || contract.baseline.name != "direct-serde-json"
        || contract.baseline.description.trim().is_empty()
        || contract.baseline.command != "baseline-evaluation"
        || contract.baseline.dataset != contract.dataset.path
        || contract.baseline.scorer != "expected-success-or-rejection-v3"
        || contract.baseline.artifact != "process:exit-code"
        || contract.baseline.sample_size != contract.dataset.distinct_cases
        || !contract
            .baseline
            .expected_task_outcome_pass_rate
            .is_finite()
        || contract.baseline.comparison != "target task_outcome_pass_rate must exceed baseline"
        || !contract.budgets.p95_latency_ms.is_finite()
        || contract.budgets.p95_latency_ms < 0.0
        || !contract.budgets.max_estimated_cost_usd.is_finite()
        || contract.budgets.max_estimated_cost_usd < 0.0
        || contract.budgets.measurement_command != "offline-evaluation"
        || contract.budgets.artifact != "evidence/evaluation.json"
    {
        return Err("acceptance contract metadata mismatch".into());
    }

    let task = metric(contract, "task_outcome_pass_rate")?;
    let illegal = metric(contract, "invalid_or_ambiguous_acceptance_rate")?;
    let latency = metric(contract, "p95_pipeline_latency_ms")?;
    for metric in [task, illegal, latency] {
        if !metric.threshold.is_finite()
            || metric.sample_size == 0
            || metric.dataset != contract.dataset.path
            || metric.command != "offline-evaluation"
            || metric.artifact != "evidence/evaluation.json"
        {
            return Err("acceptance metric routing mismatch".into());
        }
    }
    if task.direction != "higher"
        || task.scorer != "expected-success-or-rejection-v3"
        || task.sample_size != contract.dataset.distinct_cases
        || illegal.direction != "lower"
        || illegal.scorer != "reject-invalid-or-ambiguous-v3"
        || illegal.sample_size != contract.dataset.invalid_cases
        || latency.direction != "lower"
        || latency.scorer != "full-pipeline-steady-clock-v1"
        || latency.threshold != contract.budgets.p95_latency_ms
        || latency.sample_size % contract.dataset.distinct_cases != 0
    {
        return Err("acceptance metric definition mismatch".into());
    }

    let required_commands: Vec<_> = contract
        .command
        .iter()
        .filter(|command| command.required)
        .map(|command| (command.name.as_str(), command.run.as_str()))
        .collect();
    if contract.command.len() != 3
        || required_commands
            != [
                ("baseline-evaluation", "cargo test --locked --test baseline"),
                ("offline-evaluation", COMMAND),
                ("offline-tests", "cargo test --locked"),
            ]
    {
        return Err("acceptance commands mismatch".into());
    }
    Ok(latency.sample_size / contract.dataset.distinct_cases)
}

fn verify_registered_manifest(contract: &AcceptanceContract) -> Result<(String, String), String> {
    let manifest_sha256 = sha256_hex(EVALUATION_MANIFEST_SRC.as_bytes());
    if manifest_sha256 != contract.dataset.manifest_sha256 {
        return Err("acceptance contract does not bind the current case manifest".into());
    }

    let manifest: RegisteredManifest =
        serde_json::from_str(EVALUATION_MANIFEST_SRC).map_err(|_| "invalid case manifest")?;
    if manifest.schema_version != 1
        || manifest.contract_version != contract.contract_version
        || manifest.manifest_version != contract.dataset.manifest_version
        || manifest.cases.len() != contract.dataset.distinct_cases
        || EVALUATION_CASES.len() != contract.dataset.distinct_cases
    {
        return Err("case manifest metadata mismatch".into());
    }

    for (registered, case) in manifest.cases.iter().zip(EVALUATION_CASES) {
        let illegal = !matches!(case.expected, ExpectedOutcome::Success);
        if registered.name != case.name
            || registered.path != case.path
            || registered.expected != case.expected.as_str()
            || registered.illegal != illegal
            || registered.baseline_accepts != baseline_accepts(case.raw)
            || registered.sha256 != sha256_hex(case.raw.as_bytes())
        {
            return Err("case manifest entry mismatch".into());
        }
    }
    if manifest.cases.iter().filter(|case| case.illegal).count() != contract.dataset.invalid_cases {
        return Err("case manifest invalid-case count mismatch".into());
    }
    Ok((manifest.manifest_version, manifest_sha256))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output =
        output_path().map_err(|message| -> Box<dyn std::error::Error> { message.into() })?;
    let contract = load_acceptance_contract()?;
    let iterations = verify_acceptance_contract(&contract)?;
    let (manifest_version, manifest_sha256) = verify_registered_manifest(&contract)?;
    verify_inventory(
        &["."],
        SOURCE_FILES
            .iter()
            .map(|(path, _)| *path)
            .chain(DATASET_FILES.iter().map(|(path, _)| *path)),
    )?;
    let schema: Value = serde_json::from_str(MEETING_SCHEMA_SRC)?;
    let validator = SchemaValidator::compile(&schema)?;

    let mut durations = Vec::with_capacity(EVALUATION_CASES.len() * iterations);
    let mut target_passes = vec![0usize; EVALUATION_CASES.len()];
    let mut illegal_accepts = vec![0usize; EVALUATION_CASES.len()];
    let mut observed = vec![ActualOutcome::OtherFailure; EVALUATION_CASES.len()];

    for _ in 0..iterations {
        for (index, case) in EVALUATION_CASES.iter().enumerate() {
            let started = Instant::now();
            let actual = classify_target(case.raw, &validator);
            durations.push(started.elapsed().as_nanos());
            observed[index] = actual;
            if target_matches(actual, case.expected) {
                target_passes[index] += 1;
            }
            if is_illegal_acceptance(actual, case.expected) {
                illegal_accepts[index] += 1;
            }
        }
    }
    durations.sort_unstable();

    let target_pass_count: usize = target_passes.iter().sum();
    let illegal_accept_count: usize = illegal_accepts.iter().sum();
    let invalid_distinct_cases = EVALUATION_CASES
        .iter()
        .filter(|case| !matches!(case.expected, ExpectedOutcome::Success))
        .count();
    let invalid_measurements = invalid_distinct_cases * iterations;
    let baseline_pass_count = EVALUATION_CASES
        .iter()
        .filter(|case| baseline_task_matches(baseline_accepts(case.raw), case.expected))
        .count();

    let task_outcome_pass_rate = target_pass_count as f64 / durations.len() as f64;
    let invalid_or_ambiguous_acceptance_rate =
        illegal_accept_count as f64 / invalid_measurements as f64;
    let baseline_task_outcome_pass_rate =
        baseline_pass_count as f64 / EVALUATION_CASES.len() as f64;
    let p50_latency_ms = percentile(&durations, 0.50);
    let p95_latency_ms = percentile(&durations, 0.95);
    let generated_at_utc_unix_seconds = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let (source_fingerprint_sha256, source_files) = fingerprint(SOURCE_FILES);
    let (dataset_fingerprint_sha256, dataset_files) = fingerprint(DATASET_FILES);
    let actual_prompt_sha256 = sha256_hex(SYSTEM_PROMPT.as_bytes());
    let estimated_cost_usd = 0.0;

    let case_results: Vec<Value> = EVALUATION_CASES
        .iter()
        .enumerate()
        .map(|(index, case)| {
            let baseline_accepted = baseline_accepts(case.raw);
            json!({
                "name": case.name,
                "path": case.path,
                "expected": case.expected.as_str(),
                "observed": observed[index].as_str(),
                "target_passes": target_passes[index],
                "target_measurements": iterations,
                "illegal_accepts": illegal_accepts[index],
                "baseline_accepted": baseline_accepted,
                "baseline_task_match": baseline_task_matches(baseline_accepted, case.expected),
            })
        })
        .collect();

    let result = json!({
        "schema_version": 2,
        "project": "structured-output",
        "project_version": env!("CARGO_PKG_VERSION"),
        "generated_at_utc_unix_seconds": generated_at_utc_unix_seconds,
        "source_fingerprint_sha256": source_fingerprint_sha256,
        "source_files": source_files,
        "dataset_fingerprint_sha256": dataset_fingerprint_sha256,
        "dataset_files": dataset_files,
        "acceptance_contract_version": contract.contract_version,
        "prompt": {
            "id": &contract.prompt.id,
            "version": &contract.prompt.version,
            "sha256": &actual_prompt_sha256,
        },
        "context_policy_version": CONTEXT_POLICY_VERSION,
        "dataset": &contract.dataset.path,
        "dataset_version": &contract.dataset.manifest_version,
        "manifest_version": manifest_version,
        "manifest_sha256": manifest_sha256,
        "distinct_cases": EVALUATION_CASES.len(),
        "invalid_distinct_cases": invalid_distinct_cases,
        "iterations": iterations,
        "measurements": durations.len(),
        "invalid_measurements": invalid_measurements,
        "task_outcome_pass_count": target_pass_count,
        "task_outcome_pass_rate": task_outcome_pass_rate,
        "illegal_accept_count": illegal_accept_count,
        "invalid_or_ambiguous_acceptance_rate": invalid_or_ambiguous_acceptance_rate,
        "baseline_task_outcome_pass_count": baseline_pass_count,
        "baseline_task_outcome_pass_rate": baseline_task_outcome_pass_rate,
        "p50_pipeline_latency_ms": p50_latency_ms,
        "p95_pipeline_latency_ms": p95_latency_ms,
        "estimated_cost_usd": estimated_cost_usd,
        "model_calls": 0,
        "case_results": case_results,
        "execution": {
            "command": COMMAND,
            "rustc": command_version("rustc"),
            "cargo": command_version("cargo"),
            "os": env::consts::OS,
            "arch": env::consts::ARCH,
            "declared_toolchain": include_str!("../../rust-toolchain.toml").trim(),
            "network_used": false,
            "real_provider_conformance": "not_executed",
        },
    });

    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&output, serde_json::to_vec_pretty(&result)?)?;

    let task_metric = metric(&contract, "task_outcome_pass_rate")?;
    let illegal_metric = metric(&contract, "invalid_or_ambiguous_acceptance_rate")?;
    let latency_metric = metric(&contract, "p95_pipeline_latency_ms")?;
    if actual_prompt_sha256 != contract.prompt.sha256
        || EVALUATION_CASES.len() != contract.dataset.distinct_cases
        || invalid_distinct_cases != contract.dataset.invalid_cases
        || durations.len() != latency_metric.sample_size
        || !metric_passes(task_outcome_pass_rate, task_metric)?
        || !metric_passes(invalid_or_ambiguous_acceptance_rate, illegal_metric)?
        || !metric_passes(p95_latency_ms, latency_metric)?
        || baseline_task_outcome_pass_rate != contract.baseline.expected_task_outcome_pass_rate
        || task_outcome_pass_rate <= baseline_task_outcome_pass_rate
        || p95_latency_ms > contract.budgets.p95_latency_ms
        || estimated_cost_usd > contract.budgets.max_estimated_cost_usd
    {
        return Err("evaluation threshold failed".into());
    }
    Ok(())
}
