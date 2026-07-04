use std::collections::HashSet;
use std::sync::OnceLock;

use regex::Regex;

use crate::config::CleanMode;
use crate::models::{NoiseSource, NoiseSourceKind, RemovedSection, RemovedSectionKind};

pub(crate) fn crush_logs(
    input: &str,
    mode: CleanMode,
    removed_sections: &mut Vec<RemovedSection>,
    noise_sources: &mut Vec<NoiseSource>,
) -> String {
    let without_noise = remove_install_noise(input, mode, removed_sections, noise_sources);
    let with_collapsed_stacks =
        collapse_duplicate_stack_frames(&without_noise, removed_sections, noise_sources);
    collapse_repeated_lines(&with_collapsed_stacks, removed_sections, noise_sources)
}

fn remove_install_noise(
    input: &str,
    mode: CleanMode,
    removed_sections: &mut Vec<RemovedSection>,
    noise_sources: &mut Vec<NoiseSource>,
) -> String {
    let mut kept = Vec::new();
    let mut removed = 0usize;
    let mut removed_tokens = 0usize;

    for line in input.lines() {
        if is_install_noise(line, mode) && !is_signal_line(line) {
            removed += 1;
            removed_tokens += estimate_tokens(line);
        } else {
            kept.push(line);
        }
    }

    if removed > 0 {
        removed_sections.push(RemovedSection {
            kind: RemovedSectionKind::LogNoise,
            label: "install/build noise".to_string(),
            tokens_removed: removed_tokens,
            count: removed,
        });
        noise_sources.push(NoiseSource {
            kind: NoiseSourceKind::LogNoise,
            label: "install/build noise".to_string(),
            tokens_removed: removed_tokens,
        });
    }

    kept.join("\n")
}

fn collapse_duplicate_stack_frames(
    input: &str,
    removed_sections: &mut Vec<RemovedSection>,
    noise_sources: &mut Vec<NoiseSource>,
) -> String {
    let mut output = Vec::new();
    let mut seen_frames = HashSet::new();
    let mut duplicate_count = 0usize;
    let mut removed_tokens = 0usize;

    let flush = |output: &mut Vec<String>,
                 seen_frames: &mut HashSet<String>,
                 duplicate_count: &mut usize,
                 removed_tokens: &mut usize| {
        if *duplicate_count > 0 {
            output.push(format!(
                "[Collapsed stack frames: {} duplicate frames removed]",
                *duplicate_count
            ));
        }
        seen_frames.clear();
        *duplicate_count = 0;
        *removed_tokens = 0;
    };

    let mut total_duplicates = 0usize;
    let mut total_removed_tokens = 0usize;
    for line in input.lines() {
        if is_stack_frame(line) {
            let normalized = normalize_stack_frame(line);
            if seen_frames.contains(&normalized) {
                duplicate_count += 1;
                total_duplicates += 1;
                removed_tokens += estimate_tokens(line);
                total_removed_tokens += estimate_tokens(line);
            } else {
                seen_frames.insert(normalized);
                output.push(line.to_string());
            }
        } else {
            flush(
                &mut output,
                &mut seen_frames,
                &mut duplicate_count,
                &mut removed_tokens,
            );
            output.push(line.to_string());
        }
    }
    flush(
        &mut output,
        &mut seen_frames,
        &mut duplicate_count,
        &mut removed_tokens,
    );

    if total_duplicates > 0 {
        removed_sections.push(RemovedSection {
            kind: RemovedSectionKind::StackFrame,
            label: "duplicate stack frames".to_string(),
            tokens_removed: total_removed_tokens,
            count: total_duplicates,
        });
        noise_sources.push(NoiseSource {
            kind: NoiseSourceKind::StackTrace,
            label: "duplicate stack frames".to_string(),
            tokens_removed: total_removed_tokens,
        });
    }

    output.join("\n")
}

fn collapse_repeated_lines(
    input: &str,
    removed_sections: &mut Vec<RemovedSection>,
    noise_sources: &mut Vec<NoiseSource>,
) -> String {
    let mut output = Vec::new();
    let mut current_line: Option<String> = None;
    let mut last_repeated_line: Option<String> = None;
    let mut current_key: Option<String> = None;
    let mut repeat_count = 0usize;
    let mut removed_tokens = 0usize;
    let mut group_count = 0usize;

    let flush = |output: &mut Vec<String>,
                 current_line: &mut Option<String>,
                 last_repeated_line: &mut Option<String>,
                 current_key: &mut Option<String>,
                 repeat_count: &mut usize,
                 removed_tokens: &mut usize,
                 group_count: &mut usize| {
        if let Some(line) = current_line.take() {
            if *repeat_count > 1 && !line.trim().is_empty() {
                *group_count += 1;
                let last = last_repeated_line.as_deref().unwrap_or(&line);
                output.push(format_repeated_summary(*repeat_count, &line, last));
            } else {
                output.push(line);
            }
        }
        *last_repeated_line = None;
        *current_key = None;
        *repeat_count = 0;
        *removed_tokens = 0;
    };

    let mut total_removed_tokens = 0usize;
    for line in input.lines() {
        let key = repetition_key(line);
        match (&current_key, &key) {
            (Some(previous), Some(next)) if previous == next => {
                repeat_count += 1;
                last_repeated_line = Some(line.to_string());
                removed_tokens += estimate_tokens(line);
                total_removed_tokens += estimate_tokens(line);
            }
            _ => {
                flush(
                    &mut output,
                    &mut current_line,
                    &mut last_repeated_line,
                    &mut current_key,
                    &mut repeat_count,
                    &mut removed_tokens,
                    &mut group_count,
                );
                current_line = Some(line.to_string());
                last_repeated_line = None;
                current_key = key;
                repeat_count = 1;
            }
        }
    }
    flush(
        &mut output,
        &mut current_line,
        &mut last_repeated_line,
        &mut current_key,
        &mut repeat_count,
        &mut removed_tokens,
        &mut group_count,
    );

    if group_count > 0 {
        removed_sections.push(RemovedSection {
            kind: RemovedSectionKind::DuplicateLine,
            label: "repeated log lines".to_string(),
            tokens_removed: total_removed_tokens,
            count: group_count,
        });
        noise_sources.push(NoiseSource {
            kind: NoiseSourceKind::Repetition,
            label: "repeated log lines".to_string(),
            tokens_removed: total_removed_tokens,
        });
    }

    output.join("\n")
}

fn repetition_key(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with("[Collapsed stack frames:") {
        return None;
    }

    let without_timestamp = timestamp_prefix_regex().replace(trimmed, "");
    let normalized = whitespace_regex()
        .replace_all(without_timestamp.trim(), " ")
        .to_ascii_lowercase();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

fn format_repeated_summary(count: usize, first_line: &str, last_line: &str) -> String {
    let summary = summarize_repeated_line(first_line);
    match (
        timestamp_for_line(first_line),
        timestamp_for_line(last_line),
    ) {
        (Some(first), Some(last)) if first != last => {
            format!("[Repeated {count} times from {first} to {last}] {summary}")
        }
        _ => format!("[Repeated {count} times] {summary}"),
    }
}

fn summarize_repeated_line(line: &str) -> String {
    let without_timestamp = timestamp_prefix_regex().replace(line.trim(), "");
    let without_level = log_level_prefix_regex().replace(without_timestamp.trim(), "");
    cleanup_summary(without_level.trim())
}

fn timestamp_for_line(line: &str) -> Option<String> {
    timestamp_capture_regex()
        .captures(line)
        .and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
}

fn cleanup_summary(input: &str) -> String {
    whitespace_regex()
        .replace_all(input, " ")
        .trim()
        .to_string()
}

fn is_stack_frame(line: &str) -> bool {
    js_stack_regex().is_match(line)
        || python_stack_regex().is_match(line)
        || java_stack_regex().is_match(line)
        || rust_stack_regex().is_match(line)
}

fn normalize_stack_frame(line: &str) -> String {
    whitespace_regex()
        .replace_all(line.trim(), " ")
        .to_ascii_lowercase()
}

fn is_install_noise(line: &str, mode: CleanMode) -> bool {
    let lower = normalize_noise_candidate(line);
    if lower.is_empty() {
        return false;
    }

    let standard_noise = npm_summary_regex().is_match(&lower)
        || lower.starts_with("npm notice")
        || cargo_download_regex().is_match(&lower)
        || lower.starts_with("installing component ")
        || lower.starts_with("installing collected packages:")
        || lower.starts_with("resolving deltas:")
        || lower.starts_with("receiving objects:")
        || lower.starts_with("remote: counting objects:");

    let aggressive_noise = standard_noise
        || cargo_compile_regex().is_match(&lower)
        || cargo_check_regex().is_match(&lower)
        || lower.starts_with("finished `dev` profile")
        || lower.starts_with("finished `test` profile")
        || lower.starts_with("finished `release` profile")
        || lower.starts_with("running unittests ")
        || lower.starts_with("running tests/");

    match mode {
        CleanMode::Light => false,
        CleanMode::Standard => standard_noise,
        CleanMode::Aggressive => aggressive_noise,
    }
}

fn normalize_noise_candidate(line: &str) -> String {
    let lower = line.trim().to_ascii_lowercase();
    let without_timestamp = timestamp_prefix_regex().replace(&lower, "");
    let without_bracket_prefix = bracket_prefix_regex().replace(without_timestamp.trim(), "");
    let without_level = log_level_prefix_regex().replace(without_bracket_prefix.trim(), "");
    without_level.trim().to_string()
}

fn is_signal_line(line: &str) -> bool {
    let lower = line.trim().to_ascii_lowercase();
    lower.contains("error")
        || lower.contains("failed")
        || lower.contains("failure")
        || lower.contains("panic")
        || lower.contains("exception")
        || lower.contains("typeerror")
        || lower.contains("assertion")
        || lower.contains("test result:")
        || lower.contains("tests failed")
        || lower.contains("failures:")
}

fn estimate_tokens(input: &str) -> usize {
    if input.is_empty() {
        0
    } else {
        input.chars().count().div_ceil(4)
    }
}

fn timestamp_prefix_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(
            r"(?i)^\s*\[?(?:\d{4}-\d{2}-\d{2}[T ][0-9:.+-]+z?|\d{2}:\d{2}:\d{2}(?:\.\d+)?|[A-Z][a-z]{2}\s+\d{1,2}\s+\d{2}:\d{2}:\d{2})\]?\s+",
        )
        .expect("valid regex")
    })
}

fn timestamp_capture_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(
            r"(?i)^\s*\[?(\d{4}-\d{2}-\d{2}[T ][0-9:.+-]+z?|\d{2}:\d{2}:\d{2}(?:\.\d+)?|[A-Z][a-z]{2}\s+\d{1,2}\s+\d{2}:\d{2}:\d{2})\]?\s+",
        )
        .expect("valid regex")
    })
}

fn log_level_prefix_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r"(?i)^(trace|debug|info|warn|warning|error|fatal)\s+").expect("valid regex")
    })
}

fn whitespace_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"\s+").expect("valid regex"))
}

fn js_stack_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"^\s*at\s+.+(:\d+:\d+|\(.+:\d+:\d+\))").expect("valid regex"))
}

fn python_stack_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r#"^\s*File ".+", line \d+, in .+"#).expect("valid regex"))
}

fn java_stack_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"^\s*at\s+[A-Za-z0-9_.$]+\(.*:\d+\)").expect("valid regex"))
}

fn rust_stack_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"^\s*\d+:\s+.+::.+").expect("valid regex"))
}

fn npm_summary_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r"^(added|removed|changed|audited)\s+\d+\s+packages?(?:\s+in\s+.+)?$|^found\s+0\s+vulnerabilities$|^up to date in\s+")
            .expect("valid regex")
    })
}

fn cargo_download_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r"^download(?:ed|ing)\s+[a-z0-9_.-]+\s+v?\d").expect("valid regex")
    })
}

fn cargo_compile_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"^compiling\s+[a-z0-9_.-]+\s+v?\d").expect("valid regex"))
}

fn cargo_check_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"^checking\s+[a-z0-9_.-]+\s+v?\d").expect("valid regex"))
}

fn bracket_prefix_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"^\[[^\]]{1,32}\]\s+").expect("valid regex"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crushes_repeated_timestamped_lines() {
        let mut removed = Vec::new();
        let mut noise = Vec::new();
        let input = "2026-07-04T10:00:01Z warn Connection timeout to database\n2026-07-04T10:00:02Z warn Connection timeout to database\n2026-07-04T10:00:03Z warn Connection timeout to database\n2026-07-04T10:00:04Z error TypeError: Cannot read properties of undefined";

        let output = crush_logs(input, CleanMode::Standard, &mut removed, &mut noise);

        assert!(output.contains(
            "[Repeated 3 times from 2026-07-04T10:00:01Z to 2026-07-04T10:00:03Z] Connection timeout to database"
        ));
        assert!(output.contains("2026-07-04T10:00:04Z error TypeError"));
        assert!(!removed.is_empty());
    }

    #[test]
    fn collapses_duplicate_stack_frames_but_keeps_unique_failure() {
        let mut removed = Vec::new();
        let mut noise = Vec::new();
        let input = "TypeError: Cannot read properties of undefined\n    at loadUser (/app/src/user.ts:42:13)\n    at main (/app/src/main.ts:8:1)\n    at loadUser (/app/src/user.ts:42:13)\n    at main (/app/src/main.ts:8:1)";

        let output = crush_logs(input, CleanMode::Standard, &mut removed, &mut noise);

        assert!(output.contains("TypeError: Cannot read properties of undefined"));
        assert!(output.contains("at loadUser"));
        assert!(output.contains("[Collapsed stack frames: 2 duplicate frames removed]"));
    }

    #[test]
    fn stack_frame_collapse_preserves_distinct_locations() {
        let mut removed = Vec::new();
        let mut noise = Vec::new();
        let input = "TypeError: bad user\n    at validate (/app/src/user.ts:10:1)\n    at validate (/app/src/user.ts:20:1)\n    at validate (/app/src/user.ts:10:1)";

        let output = crush_logs(input, CleanMode::Standard, &mut removed, &mut noise);

        assert!(output.contains("at validate (/app/src/user.ts:10:1)"));
        assert!(output.contains("at validate (/app/src/user.ts:20:1)"));
        assert!(output.contains("[Collapsed stack frames: 1 duplicate frames removed]"));
    }

    #[test]
    fn collapses_bracketed_timestamp_repeats() {
        let mut removed = Vec::new();
        let mut noise = Vec::new();
        let input = "[2026-07-04T10:00:01Z] warn queue reconnect\n[2026-07-04T10:00:02Z] warn queue reconnect\n[2026-07-04T10:00:03Z] warn queue reconnect\n[2026-07-04T10:00:04Z] error final failure";

        let output = crush_logs(input, CleanMode::Standard, &mut removed, &mut noise);

        assert!(output.contains(
            "[Repeated 3 times from 2026-07-04T10:00:01Z to 2026-07-04T10:00:03Z] queue reconnect"
        ));
        assert!(output.contains("[2026-07-04T10:00:04Z] error final failure"));
    }

    #[test]
    fn removes_safe_install_noise_without_dropping_failed_test_names() {
        let mut removed = Vec::new();
        let mut noise = Vec::new();
        let input = "added 481 packages\nfound 0 vulnerabilities\nFAIL packages/api/user.test.ts\nExpected user id to exist";

        let output = crush_logs(input, CleanMode::Standard, &mut removed, &mut noise);

        assert!(!output.contains("added 481 packages"));
        assert!(output.contains("FAIL packages/api/user.test.ts"));
        assert!(output.contains("Expected user id to exist"));
    }

    #[test]
    fn removes_prefixed_install_summaries() {
        let mut removed = Vec::new();
        let mut noise = Vec::new();
        let input =
            "2026-07-04T10:00:00Z info added 481 packages\n[build] Installing collected packages: attrs, click\nINFO found 0 vulnerabilities\nFAIL packages/api/user.test.ts";

        let output = crush_logs(input, CleanMode::Standard, &mut removed, &mut noise);

        assert!(!output.contains("added 481 packages"));
        assert!(!output.contains("Installing collected packages"));
        assert!(!output.contains("found 0 vulnerabilities"));
        assert!(output.contains("FAIL packages/api/user.test.ts"));
    }

    #[test]
    fn install_noise_rules_preserve_provenance_lines() {
        let mut removed = Vec::new();
        let mut noise = Vec::new();
        let input = "running `cargo test -p api user_contract_test`\nRunning packages/api/user.test.ts\nChecking database migration 20260704_add_user_indexes\nFinished writing junit report to target/junit.xml\nDownloaded artifact: playwright-report.zip to /tmp/artifacts/playwright-report.zip\nInstalling pgvector 0.7.4 extension in database app";

        let output = crush_logs(input, CleanMode::Aggressive, &mut removed, &mut noise);

        assert!(output.contains("running `cargo test -p api user_contract_test`"));
        assert!(output.contains("Running packages/api/user.test.ts"));
        assert!(output.contains("Checking database migration"));
        assert!(output.contains("Finished writing junit report"));
        assert!(output.contains("Downloaded artifact: playwright-report.zip"));
        assert!(output.contains("Installing pgvector 0.7.4 extension"));
    }

    #[test]
    fn cache_domain_events_are_not_install_noise() {
        let mut removed = Vec::new();
        let mut noise = Vec::new();
        let input = "INFO cache hit for: user permissions\nINFO cache restored user profile\nrestore cache consistency marker";

        let output = crush_logs(input, CleanMode::Standard, &mut removed, &mut noise);

        assert!(output.contains("cache hit for: user permissions"));
        assert!(output.contains("cache restored user profile"));
        assert!(output.contains("restore cache consistency marker"));
    }
}
