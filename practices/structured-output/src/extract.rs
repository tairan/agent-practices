//! Locate one JSON object inside arbitrary model output.
//!
//! Conservative and explicit by design — no auto-repair, no LLM second-pass.
//! Algorithm:
//!
//! 1. Trim ASCII whitespace; reject empty content.
//! 2. Strip a single ```fenced``` code block if present (with or without a
//!    language tag like `json`).
//! 3. Try the resulting string as a top-level JSON value first.
//! 4. If that fails, scan for balanced `{ ... }` blocks at the top level,
//!    skipping over JSON string literals (so `{` inside `"…"` does not count).
//!    Each balanced block is a candidate.
//! 5. Exactly one candidate → return it parsed. Zero candidates → extraction
//!    failed. Two or more candidates → ambiguity error (intentional: see
//!    the structured-validation rule: do not depend on the model
//!    'guaranteeing' format").
//!
//! Top-level JSON arrays are out of scope for this concept (the demo schema is
//! an object). Adding array support is straightforward but deferred to keep
//! the failure surface tight.

use serde_json::Value;

use crate::error::StructuredOutputError;

/// Extract a single JSON object from a model output and parse it to a
/// `serde_json::Value`. See module docs for the algorithm.
pub fn extract_json(content: &str) -> Result<Value, StructuredOutputError> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Err(StructuredOutputError::EmptyResponse);
    }

    let body = strip_fence(trimmed).trim();
    if body.is_empty() {
        return Err(StructuredOutputError::EmptyResponse);
    }

    // Fast path: the whole body parses as JSON. A valid non-object is a stable
    // type error, not prose to scan for nested object candidates.
    match serde_json::from_str::<Value>(body) {
        Ok(v @ Value::Object(_)) => return Ok(v),
        Ok(other) => {
            let actual = match other {
                Value::Array(_) => "array",
                Value::String(_) => "string",
                Value::Number(_) => "number",
                Value::Bool(_) => "boolean",
                Value::Null => "null",
                Value::Object(_) => unreachable!(),
            };
            return Err(StructuredOutputError::UnexpectedTopLevelType { actual });
        }
        Err(_) => {}
    }

    // Slow path: find balanced top-level { ... } blocks.
    let candidates = find_object_candidates(body);
    match candidates.len() {
        0 => Err(StructuredOutputError::JsonExtractionFailed),
        1 => {
            let slice = candidates.into_iter().next().unwrap();
            serde_json::from_str::<Value>(slice).map_err(StructuredOutputError::JsonParseError)
        }
        n => Err(StructuredOutputError::MultipleJsonCandidates { count: n }),
    }
}

/// Strip a single surrounding ```…``` fence if the string is fenced.
/// Returns the body without the fence markers; otherwise returns the input.
fn strip_fence(s: &str) -> &str {
    let Some(rest) = s.strip_prefix("```") else {
        return s;
    };
    // Drop an optional language tag and the newline after the opening fence.
    let after_lang = match rest.find('\n') {
        Some(i) => &rest[i + 1..],
        None => return s, // malformed fence: no newline → leave original
    };
    // Strip the closing fence if present at the end (allow trailing whitespace).
    let trimmed_end = after_lang.trim_end();
    if let Some(inner) = trimmed_end.strip_suffix("```") {
        inner
    } else {
        s
    }
}

/// Find every balanced top-level `{ ... }` block in `s`, returning slices.
/// Skips brace characters that appear inside JSON string literals.
fn find_object_candidates(s: &str) -> Vec<&str> {
    let bytes = s.as_bytes();
    let mut out: Vec<&str> = Vec::new();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] != b'{' {
            i += 1;
            continue;
        }

        let start = i;
        let mut depth: u32 = 0;
        let mut in_string = false;
        let mut escape = false;
        let mut closed_at: Option<usize> = None;

        let mut j = i;
        while j < bytes.len() {
            let c = bytes[j];
            if in_string {
                if escape {
                    escape = false;
                } else if c == b'\\' {
                    escape = true;
                } else if c == b'"' {
                    in_string = false;
                }
            } else {
                match c {
                    b'"' => in_string = true,
                    b'{' => depth += 1,
                    b'}' => {
                        depth -= 1;
                        if depth == 0 {
                            closed_at = Some(j);
                            break;
                        }
                    }
                    _ => {}
                }
            }
            j += 1;
        }

        if let Some(end) = closed_at {
            out.push(&s[start..=end]);
            i = end + 1;
        } else {
            // Unbalanced from this `{` to EOF: stop scanning entirely.
            break;
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_obj_keys(v: &Value, keys: &[&str]) {
        let obj = v.as_object().expect("expected object");
        for k in keys {
            assert!(obj.contains_key(*k), "missing key {k}");
        }
    }

    #[test]
    fn extract_plain_json_object() {
        let v = extract_json(r#"{"a": 1, "b": "two"}"#).unwrap();
        assert_obj_keys(&v, &["a", "b"]);
    }

    #[test]
    fn extract_fenced_with_json_tag() {
        let raw = "```json\n{\"a\": 1}\n```";
        let v = extract_json(raw).unwrap();
        assert_obj_keys(&v, &["a"]);
    }

    #[test]
    fn extract_fenced_without_tag() {
        let raw = "```\n{\"a\": 1}\n```";
        let v = extract_json(raw).unwrap();
        assert_obj_keys(&v, &["a"]);
    }

    #[test]
    fn extract_chatty_prose_around_json() {
        let raw = "Sure, here is the result:\n{\"a\": 1}\nLet me know if you want changes.";
        let v = extract_json(raw).unwrap();
        assert_obj_keys(&v, &["a"]);
    }

    #[test]
    fn extract_empty_returns_empty_response() {
        let err = extract_json("   ").unwrap_err();
        assert!(matches!(err, StructuredOutputError::EmptyResponse));
    }

    #[test]
    fn extract_truncated_returns_extraction_failed() {
        // Open brace with no matching close → no candidates found.
        let err = extract_json(r#"{"a": 1, "b": [1, 2"#).unwrap_err();
        assert!(matches!(err, StructuredOutputError::JsonExtractionFailed));
    }

    #[test]
    fn extract_two_independent_objects_is_ambiguous() {
        let raw = r#"{"a": 1} and also {"b": 2}"#;
        let err = extract_json(raw).unwrap_err();
        match err {
            StructuredOutputError::MultipleJsonCandidates { count, .. } => assert_eq!(count, 2),
            e => panic!("expected MultipleJsonCandidates, got {e:?}"),
        }
    }

    #[test]
    fn extract_no_braces_at_all() {
        let err = extract_json("the answer is 42").unwrap_err();
        assert!(matches!(err, StructuredOutputError::JsonExtractionFailed));
    }

    #[test]
    fn extract_nested_object_kept_as_one_candidate() {
        let raw = r#"{"outer": {"inner": 1}}"#;
        let v = extract_json(raw).unwrap();
        assert_obj_keys(&v, &["outer"]);
        assert!(v["outer"].is_object());
    }

    #[test]
    fn extract_braces_inside_string_ignored() {
        let raw = r#"{"msg": "this has { and } in it", "ok": true}"#;
        let v = extract_json(raw).unwrap();
        assert_eq!(v["msg"], "this has { and } in it");
        assert_eq!(v["ok"], true);
    }

    #[test]
    fn extract_escaped_quote_inside_string() {
        let raw = r#"{"msg": "she said \"hi\""}"#;
        let v = extract_json(raw).unwrap();
        assert_eq!(v["msg"], "she said \"hi\"");
    }

    #[test]
    fn extract_unicode_keys_and_values() {
        let raw = r#"{"标题": "会议", "出席": ["张三"]}"#;
        let v = extract_json(raw).unwrap();
        assert_eq!(v["标题"], "会议");
    }

    #[test]
    fn extract_fenced_with_prose_outside_is_handled() {
        let raw = "Here you go:\n```json\n{\"a\": 1}\n```\nDone.";
        // The fence stripper only fires when the trimmed body starts with ```,
        // so this falls through to the candidate scan, which finds exactly one.
        let v = extract_json(raw).unwrap();
        assert_obj_keys(&v, &["a"]);
    }

    #[test]
    fn rejects_top_level_arrays_without_extracting_nested_objects() {
        for raw in [r#"[{"a":1}]"#, r#"[{"a":1},{"b":2}]"#] {
            assert!(matches!(
                extract_json(raw),
                Err(StructuredOutputError::UnexpectedTopLevelType { actual: "array" })
            ));
        }
    }

    #[test]
    fn errors_do_not_echo_sensitive_model_content() {
        let canary = "SECRET_CANARY_9f31";
        let error = extract_json(canary).unwrap_err();
        assert!(!error.to_string().contains(canary));
        assert!(!format!("{error:?}").contains(canary));
    }
}
