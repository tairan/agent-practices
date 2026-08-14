//! Deterministic baseline from `acceptance.toml`: direct serde deserialization.

use serde::Deserialize;
use structured_output::{
    EVALUATION_CASES, EVALUATION_MANIFEST_SRC, baseline_accepts, baseline_task_matches,
};

#[derive(Deserialize)]
struct Manifest {
    cases: Vec<ManifestCase>,
}

#[derive(Deserialize)]
struct ManifestCase {
    name: String,
    baseline_accepts: bool,
}

#[test]
fn direct_serde_only_accepts_the_bare_json_fixture() {
    let manifest: Manifest = serde_json::from_str(EVALUATION_MANIFEST_SRC).unwrap();
    assert_eq!(manifest.cases.len(), EVALUATION_CASES.len());
    let mut task_outcome_passes = 0usize;
    for (case, registered) in EVALUATION_CASES.iter().zip(&manifest.cases) {
        assert_eq!(registered.name, case.name);
        let actual_success = baseline_accepts(case.raw);
        assert_eq!(
            actual_success, registered.baseline_accepts,
            "case {}",
            case.name
        );
        if baseline_task_matches(actual_success, case.expected) {
            task_outcome_passes += 1;
        }
    }

    // Direct serde misses fenced/chatty and incorrectly accepts missing_due_date
    // and extra_field. It matches the target contract on 6/10 cases.
    assert_eq!(task_outcome_passes, 6);
}
