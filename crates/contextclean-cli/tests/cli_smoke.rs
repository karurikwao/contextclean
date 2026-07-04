use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("fixtures")
        .join(name)
}

fn estimated_tokens(bytes: &[u8]) -> usize {
    let text = String::from_utf8_lossy(bytes);
    if text.is_empty() {
        0
    } else {
        text.chars().count().div_ceil(4)
    }
}

fn json_array_has_kind(parsed: &Value, array_name: &str, kind: &str) -> bool {
    parsed[array_name]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["kind"] == kind)
}

fn json_array_has_warning(parsed: &Value, code: &str) -> bool {
    parsed["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["code"] == code)
}

#[test]
fn help_starts_successfully() {
    let mut command = Command::cargo_bin("ctxclean").unwrap();
    command
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("ContextClean strips"))
        .stdout(predicate::str::contains("--format"))
        .stdout(predicate::str::contains("--max-tokens"))
        .stdout(predicate::str::contains("--no-redact-secrets"));
}

#[test]
fn cleans_file_to_markdown() {
    let temp = tempdir().unwrap();
    let input = temp.path().join("dirty.html");
    fs::write(
        &input,
        "<html><script>noise()</script><main><h1>Keep Me</h1><p>Hello</p></main></html>",
    )
    .unwrap();

    let mut command = Command::cargo_bin("ctxclean").unwrap();
    command
        .arg(&input)
        .assert()
        .success()
        .stdout(predicate::str::contains("# Cleaned Context"))
        .stdout(predicate::str::contains("Keep Me"))
        .stdout(predicate::str::contains("noise").not());
}

#[test]
fn emits_parseable_json() {
    let temp = tempdir().unwrap();
    let input = temp.path().join("log.txt");
    fs::write(&input, "error one\nerror one\nfinal failure").unwrap();

    let mut command = Command::cargo_bin("ctxclean").unwrap();
    let output = command
        .arg(&input)
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let parsed: Value = serde_json::from_slice(&output).unwrap();

    assert_eq!(parsed["mode"], "standard");
    assert_eq!(parsed["format"], "json");
    assert!(parsed["metrics"]["input_tokens"].as_u64().unwrap() > 0);
    assert!(parsed["removed_sections"].is_array());
    assert!(parsed["warnings"].is_array());
    assert!(parsed["output"]["content"]
        .as_str()
        .unwrap()
        .contains("final failure"));
}

#[test]
fn refuses_to_overwrite_without_force() {
    let temp = tempdir().unwrap();
    let input = temp.path().join("input.txt");
    let output = temp.path().join("output.md");
    fs::write(&input, "hello").unwrap();
    fs::write(&output, "existing").unwrap();

    let mut command = Command::cargo_bin("ctxclean").unwrap();
    command
        .arg(&input)
        .arg("--output")
        .arg(&output)
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("output file already exists"));

    assert_eq!(fs::read_to_string(output).unwrap(), "existing");
}

#[test]
fn writes_output_with_force() {
    let temp = tempdir().unwrap();
    let input = temp.path().join("input.txt");
    let output = temp.path().join("output.md");
    fs::write(&input, "hello").unwrap();
    fs::write(&output, "existing").unwrap();

    let mut command = Command::cargo_bin("ctxclean").unwrap();
    command
        .arg(&input)
        .arg("--output")
        .arg(&output)
        .arg("--force")
        .assert()
        .success();

    assert!(fs::read_to_string(output).unwrap().contains("hello"));
}

#[test]
fn writes_output_with_out_alias() {
    let temp = tempdir().unwrap();
    let input = temp.path().join("input.txt");
    let output = temp.path().join("output.md");
    fs::write(&input, "hello alias").unwrap();

    let mut command = Command::cargo_bin("ctxclean").unwrap();
    command
        .arg(&input)
        .arg("--out")
        .arg(&output)
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("wrote cleaned output"));

    assert!(fs::read_to_string(output).unwrap().contains("hello alias"));
}

#[test]
fn committed_html_fixture_smokes() {
    let mut command = Command::cargo_bin("ctxclean").unwrap();
    command
        .arg(fixture_path("dirty_html_small.html"))
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "ContextClean keeps the main article",
        ))
        .stdout(predicate::str::contains("window.analytics").not())
        .stdout(predicate::str::contains("Newsletter signup").not());
}

#[test]
fn dirty_html_article_preserves_visible_structure() {
    let mut command = Command::cargo_bin("ctxclean").unwrap();
    let output = command
        .arg(fixture_path("dirty_html_article.html"))
        .arg("--mode")
        .arg("standard")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let parsed: Value = serde_json::from_slice(&output).unwrap();
    let content = parsed["output"]["content"].as_str().unwrap();

    assert!(content.contains("# API Setup & Troubleshooting"));
    assert!(content.contains("[setup guide](/docs/setup?lang=rust&step=install)"));
    assert!(content.contains("| Mode | Removes | Keeps |"));
    assert!(content.contains("| standard | cookie banners | src/app.rs:42 |"));
    assert!(content.contains("```"));
    assert!(content.contains("    println!(\"KEEP_CODE_INDENT\");"));
    assert!(content.contains("Unique visible conclusion"));
    assert!(!content.contains("window.analytics"));
    assert!(!content.contains("Accept all cookies"));
    assert!(!content.contains("Advertisement: buy a distraction"));
    assert!(parsed["metrics"]["tokens_saved"].as_i64().unwrap() > 0);
    assert!(json_array_has_kind(
        &parsed,
        "removed_sections",
        "html_execution_block"
    ));
    assert!(json_array_has_kind(
        &parsed,
        "removed_sections",
        "html_boilerplate"
    ));
    assert!(json_array_has_kind(
        &parsed,
        "noise_sources",
        "html_boilerplate"
    ));
}

#[test]
fn committed_log_fixture_smokes_with_truncation_json() {
    let mut command = Command::cargo_bin("ctxclean").unwrap();
    let output = command
        .arg(fixture_path("repeated_log.txt"))
        .arg("--max-tokens")
        .arg("40")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let parsed: Value = serde_json::from_slice(&output).unwrap();

    assert_eq!(parsed["truncation"]["applied"], true);
    assert!(parsed["output"]["tokens"].as_u64().unwrap() <= 40);
    let removed = parsed["truncation"]["tokens_removed"].as_u64().unwrap();
    let content = parsed["output"]["content"].as_str().unwrap();
    if content.contains("Removed ") {
        assert!(content.contains(&format!("Removed {removed} estimated tokens")));
    }
}

#[test]
fn committed_log_fixture_smokes_unbudgeted_json() {
    let mut command = Command::cargo_bin("ctxclean").unwrap();
    let output = command
        .arg(fixture_path("repeated_log.txt"))
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let parsed: Value = serde_json::from_slice(&output).unwrap();
    let content = parsed["output"]["content"].as_str().unwrap();

    assert!(content.contains("[Repeated 3 times] retrying database connection"));
    assert!(content.contains("TypeError"));
    assert!(content.contains("src/user.ts:42"));
    assert!(content.contains("src/main.ts:8"));
    assert_eq!(parsed["truncation"]["applied"], false);
}

#[test]
fn ci_failure_log_crushes_noise_but_preserves_failure() {
    let mut command = Command::cargo_bin("ctxclean").unwrap();
    let output = command
        .arg(fixture_path("ci_failure_log.txt"))
        .arg("--mode")
        .arg("standard")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let parsed: Value = serde_json::from_slice(&output).unwrap();
    let content = parsed["output"]["content"].as_str().unwrap();

    assert!(content.contains("[Repeated 3 times from 2026-07-04T10:00:01Z to 2026-07-04T10:00:03Z] Connection timeout to database"));
    assert!(content.contains("[Collapsed stack frames: 2 duplicate frames removed]"));
    assert!(content.contains("FAIL packages/api/user.test.ts"));
    assert!(content.contains("Unique failure:"));
    assert!(content.contains("TypeError: Cannot read properties of undefined"));
    assert!(content.contains("at loadUser"));
    assert!(content.contains("at main"));
    assert!(content.contains("Final error summary: request failed after retries"));
    assert!(!content.contains("added 481 packages"));
    assert!(!content.contains("found 0 vulnerabilities"));
    assert!(parsed["metrics"]["tokens_saved"].as_i64().unwrap() > 0);
    assert!(json_array_has_kind(
        &parsed,
        "removed_sections",
        "stack_frame"
    ));
    assert!(json_array_has_kind(
        &parsed,
        "removed_sections",
        "log_noise"
    ));
    assert!(json_array_has_kind(&parsed, "noise_sources", "stack_trace"));
    assert!(json_array_has_kind(&parsed, "noise_sources", "log_noise"));
}

#[test]
fn timestamped_repeats_do_not_merge_distinct_errors() {
    let input = "\
2026-07-04T10:00:01Z warn Connection timeout to database
2026-07-04T10:00:02Z warn Connection timeout to database
2026-07-04T10:00:03Z warn Connection timeout to database
2026-07-04T10:00:04Z error TypeError: Cannot read properties of undefined
2026-07-04T10:00:05Z error ReferenceError: Cannot read properties of undefined
";

    let mut command = Command::cargo_bin("ctxclean").unwrap();
    command
        .arg("--format")
        .arg("text")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "[Repeated 3 times from 2026-07-04T10:00:01Z to 2026-07-04T10:00:03Z] Connection timeout to database",
        ))
        .stdout(predicate::str::contains(
            "2026-07-04T10:00:04Z error TypeError",
        ))
        .stdout(predicate::str::contains(
            "2026-07-04T10:00:05Z error ReferenceError",
        ));
}

#[test]
fn markdown_max_tokens_caps_rendered_output() {
    let mut command = Command::cargo_bin("ctxclean").unwrap();
    let output = command
        .arg(fixture_path("repeated_log.txt"))
        .arg("--max-tokens")
        .arg("40")
        .arg("--format")
        .arg("markdown")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    assert!(estimated_tokens(&output) <= 40);
}

#[test]
fn directory_fixture_skips_sensitive_files() {
    let mut command = Command::cargo_bin("ctxclean").unwrap();
    command
        .arg(fixture_path("simple_project"))
        .arg("--format")
        .arg("text")
        .assert()
        .success()
        .stdout(predicate::str::contains("src/app.rs"))
        .stdout(predicate::str::contains("hello from fixture"))
        .stdout(predicate::str::contains(".env.example").not())
        .stdout(predicate::str::contains("fixture-fake-secret-value").not());
}

#[test]
fn signed_url_query_secrets_are_redacted_and_reported() {
    let input = r#"<main><a href="https://s3.example.com/report.csv?X-Amz-Signature=abc123secret&safe=1">signed report</a></main>"#;

    let mut command = Command::cargo_bin("ctxclean").unwrap();
    let output = command
        .arg("--format")
        .arg("json")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let parsed: Value = serde_json::from_slice(&output).unwrap();
    let content = parsed["output"]["content"].as_str().unwrap();

    assert!(content.contains("[REDACTED_SECRET]"));
    assert!(content.contains("safe=1"));
    assert!(!content.contains("abc123secret"));
    assert!(json_array_has_warning(&parsed, "secrets_redacted"));
    assert!(json_array_has_kind(&parsed, "removed_sections", "secret"));
    assert!(json_array_has_kind(&parsed, "noise_sources", "secret"));
}

#[test]
fn oversized_stdin_is_rejected() {
    let mut command = Command::cargo_bin("ctxclean").unwrap();
    command
        .arg("-")
        .write_stdin("x".repeat(4 * 1_048_576 + 1))
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("stdin input exceeds"));
}

#[test]
fn reads_explicit_stdin_dash() {
    let mut command = Command::cargo_bin("ctxclean").unwrap();
    command
        .arg("-")
        .write_stdin("hello\nhello\nkept\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("kept"));
}

#[test]
fn reads_piped_stdin_when_input_is_omitted() {
    let mut command = Command::cargo_bin("ctxclean").unwrap();
    command
        .write_stdin("omitted stdin works\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("omitted stdin works"));
}

#[test]
fn dry_run_does_not_write_output_file() {
    let temp = tempdir().unwrap();
    let input = temp.path().join("input.txt");
    let output = temp.path().join("output.md");
    fs::write(&input, "hello").unwrap();

    let mut command = Command::cargo_bin("ctxclean").unwrap();
    command
        .arg(&input)
        .arg("--dry-run")
        .arg("--output")
        .arg(&output)
        .assert()
        .success()
        .stdout(predicate::str::contains("hello"))
        .stderr(predicate::str::contains("dry run: not writing output"));

    assert!(!output.exists());
}

#[test]
fn json_verbose_keeps_stdout_parseable_and_diagnostics_on_stderr() {
    let temp = tempdir().unwrap();
    let input = temp.path().join("input.txt");
    fs::write(&input, "json verbose").unwrap();

    let mut command = Command::cargo_bin("ctxclean").unwrap();
    let assert = command
        .arg(&input)
        .arg("--format")
        .arg("json")
        .arg("--verbose")
        .assert()
        .success()
        .stderr(predicate::str::contains("input_tokens"));
    let output = assert.get_output().stdout.clone();
    let parsed: Value = serde_json::from_slice(&output).unwrap();

    assert_eq!(parsed["format"], "json");
}

#[test]
fn quiet_and_verbose_conflict() {
    let mut command = Command::cargo_bin("ctxclean").unwrap();
    command
        .arg("--quiet")
        .arg("--verbose")
        .write_stdin("hello")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(
            "--quiet and --verbose cannot be used together",
        ));
}

#[test]
fn max_tokens_zero_is_rejected() {
    let mut command = Command::cargo_bin("ctxclean").unwrap();
    command
        .arg("--max-tokens")
        .arg("0")
        .write_stdin("hello")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn cli_redacts_by_default() {
    let temp = tempdir().unwrap();
    let input = temp.path().join("input.txt");
    fs::write(
        &input,
        "DATABASE_URL=postgres://user:secret@localhost/app\n",
    )
    .unwrap();

    let mut command = Command::cargo_bin("ctxclean").unwrap();
    command
        .arg(&input)
        .arg("--format")
        .arg("markdown")
        .assert()
        .success()
        .stdout(predicate::str::contains("[REDACTED_SECRET]"))
        .stdout(predicate::str::contains("user:secret").not())
        .stdout(predicate::str::contains("secrets_redacted"));
}

#[test]
fn committed_mixed_markdown_fixture_smokes() {
    let mut command = Command::cargo_bin("ctxclean").unwrap();
    let output = command
        .arg(fixture_path("mixed_markdown.md"))
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let parsed: Value = serde_json::from_slice(&output).unwrap();
    let content = parsed["output"]["content"].as_str().unwrap();

    assert!(content.contains("This action item must be preserved."));
    assert!(content.contains("TypeError"));
    assert!(content.contains("[Repeated 3 times] warning: retrying fetch"));
    assert_eq!(content.matches("```").count() % 2, 0);
    assert!(!content.contains("Cookie preferences"));
    assert!(!content.contains("Newsletter signup"));
}

#[test]
fn directory_scan_respects_gitignore_and_ctxcleanignore() {
    let temp = tempdir().unwrap();
    fs::create_dir_all(temp.path().join("src")).unwrap();
    fs::create_dir_all(temp.path().join("ignored-by-git")).unwrap();
    fs::create_dir_all(temp.path().join("ignored-by-ctx")).unwrap();
    fs::write(temp.path().join("src/keep.txt"), "KEEP_SENTINEL").unwrap();
    fs::write(temp.path().join(".gitignore"), "ignored-by-git/\n").unwrap();
    fs::write(temp.path().join(".ctxcleanignore"), "ignored-by-ctx/\n").unwrap();
    fs::write(
        temp.path().join("ignored-by-git/file.txt"),
        "GITIGNORE_SENTINEL",
    )
    .unwrap();
    fs::write(
        temp.path().join("ignored-by-ctx/file.txt"),
        "CTXCLEANIGNORE_SENTINEL",
    )
    .unwrap();

    let mut command = Command::cargo_bin("ctxclean").unwrap();
    command
        .arg(temp.path())
        .arg("--format")
        .arg("text")
        .assert()
        .success()
        .stdout(predicate::str::contains("KEEP_SENTINEL"))
        .stdout(predicate::str::contains("GITIGNORE_SENTINEL").not())
        .stdout(predicate::str::contains("CTXCLEANIGNORE_SENTINEL").not());
}

#[test]
fn directory_scan_respects_gitignore_negation() {
    let temp = tempdir().unwrap();
    fs::write(temp.path().join(".gitignore"), "drop.txt\n!drop.txt\n").unwrap();
    fs::write(temp.path().join("drop.txt"), "NEGATION_SENTINEL").unwrap();

    let mut command = Command::cargo_bin("ctxclean").unwrap();
    command
        .arg(temp.path())
        .arg("--format")
        .arg("text")
        .assert()
        .success()
        .stdout(predicate::str::contains("NEGATION_SENTINEL"));
}

#[test]
fn directory_scan_skips_hidden_credential_files_by_default() {
    let temp = tempdir().unwrap();
    fs::create_dir_all(temp.path().join("src")).unwrap();
    fs::create_dir_all(temp.path().join(".aws")).unwrap();
    fs::write(temp.path().join("src/keep.txt"), "KEEP_SENTINEL").unwrap();
    fs::write(
        temp.path().join(".netrc"),
        "machine api.example.com login alice password netrcSecretValue123",
    )
    .unwrap();
    fs::write(
        temp.path().join(".aws/credentials"),
        "aws_secret_access_key = awsSecretValue123456",
    )
    .unwrap();

    let mut command = Command::cargo_bin("ctxclean").unwrap();
    command
        .arg(temp.path())
        .arg("--format")
        .arg("text")
        .assert()
        .success()
        .stdout(predicate::str::contains("KEEP_SENTINEL"))
        .stdout(predicate::str::contains("netrcSecretValue123").not())
        .stdout(predicate::str::contains("awsSecretValue123456").not());
}

#[test]
fn mode_selection_changes_boilerplate_behavior() {
    let temp = tempdir().unwrap();
    let input = temp.path().join("input.html");
    fs::write(
        &input,
        "<nav>Keep in light</nav><main>Unique error at src/app.rs:9</main>\n----------",
    )
    .unwrap();

    let mut light = Command::cargo_bin("ctxclean").unwrap();
    light
        .arg(&input)
        .arg("--mode")
        .arg("light")
        .arg("--format")
        .arg("text")
        .assert()
        .success()
        .stdout(predicate::str::contains("Keep in light"));

    let mut aggressive = Command::cargo_bin("ctxclean").unwrap();
    aggressive
        .arg(&input)
        .arg("--mode")
        .arg("aggressive")
        .arg("--format")
        .arg("text")
        .assert()
        .success()
        .stdout(predicate::str::contains("Unique error at src/app.rs:9"))
        .stdout(predicate::str::contains("Keep in light").not())
        .stdout(predicate::str::contains("----------").not());
}
