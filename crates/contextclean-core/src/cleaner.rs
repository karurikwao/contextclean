use std::sync::OnceLock;
use std::time::Instant;

use regex::Regex;

use crate::budget::{count_tokens, pack_to_budget, tokenizer_name};
use crate::config::{CleanMode, CleanOptions};
use crate::html::{html_to_readable_content, remove_html_noise_blocks};
use crate::logs::crush_logs;
use crate::models::{
    Budget, BudgetLimitSource, CleanResult, InputStats, Metadata, Metrics, NoiseSource,
    NoiseSourceKind, OutputBlock, RemovedSection, RemovedSectionKind, Truncation, Warning,
    WarningSeverity,
};

pub fn clean_text(input: &str, options: &CleanOptions) -> CleanResult {
    let started = Instant::now();
    let input_stats = stats_for(input);
    let mut removed_sections = Vec::new();
    let mut noise_sources = Vec::new();
    let mut warnings = Vec::new();

    let mut content = normalize_newlines(input);
    content = remove_control_chars(&content);
    content = decode_basic_entities(&content);
    content = collapse_blank_lines(&content);

    content = remove_regex_group(
        &content,
        &[
            ("script", script_regex()),
            ("style", style_regex()),
            ("noscript", noscript_regex()),
        ],
        RemovedSectionKind::HtmlExecutionBlock,
        "HTML execution/style block",
        &mut removed_sections,
        &mut noise_sources,
        NoiseSourceKind::HtmlBoilerplate,
    );

    if matches!(options.mode, CleanMode::Standard | CleanMode::Aggressive) {
        content = remove_regex_group(
            &content,
            &[
                ("nav", nav_regex()),
                ("footer", footer_regex()),
                ("aside", aside_regex()),
                ("svg", svg_regex()),
            ],
            RemovedSectionKind::HtmlBoilerplate,
            "HTML boilerplate block",
            &mut removed_sections,
            &mut noise_sources,
            NoiseSourceKind::HtmlBoilerplate,
        );
        content = remove_html_comments(&content, &mut removed_sections, &mut noise_sources);
        content = remove_html_noise_blocks(
            &content,
            options.mode,
            &mut removed_sections,
            &mut noise_sources,
        );
        content = drop_boilerplate_lines(
            &content,
            options.mode,
            &mut removed_sections,
            &mut noise_sources,
        );
        content = html_to_readable_content(&content, options.mode);
        content = crush_logs(
            &content,
            options.mode,
            &mut removed_sections,
            &mut noise_sources,
        );
    }

    if options.strip_comments {
        content = strip_code_comment_lines(&content, &mut removed_sections, &mut noise_sources);
    }

    if options.redact_secrets {
        content = redact_secrets(
            &content,
            &mut removed_sections,
            &mut noise_sources,
            &mut warnings,
        );
    }

    if matches!(options.mode, CleanMode::Aggressive) {
        content = drop_aggressive_noise(&content, &mut removed_sections);
        content = collapse_blank_lines(&content);
    }

    let mut truncation = Truncation {
        applied: false,
        limit_tokens: options.max_tokens,
        tokens_removed: 0,
        reason: None,
    };

    if let Some(max_tokens) = options.max_tokens {
        let packed = pack_to_budget(&content, max_tokens);
        if packed.applied {
            truncation.applied = true;
            truncation.tokens_removed = packed.tokens_removed;
            truncation.reason = Some(budget_reason(options));
            removed_sections.push(RemovedSection {
                kind: RemovedSectionKind::Truncated,
                label: "token budget truncation".to_string(),
                tokens_removed: packed.tokens_removed,
                count: 1,
            });
            noise_sources.push(NoiseSource {
                kind: NoiseSourceKind::Truncation,
                label: "token budget truncation".to_string(),
                tokens_removed: packed.tokens_removed,
            });
            content = packed.content;
        }
    }

    content = collapse_blank_lines(&content).trim().to_string();

    let output_stats = stats_for(&content);
    let metrics = metrics_for(input_stats.tokens, output_stats.tokens);

    CleanResult {
        version: env!("CARGO_PKG_VERSION").to_string(),
        mode: options.mode,
        format: options.format,
        source: options.source_name.clone(),
        input: input_stats,
        output: OutputBlock {
            bytes: content.len(),
            chars: content.chars().count(),
            tokens: output_stats.tokens,
            content,
        },
        metrics,
        budget: budget_for(options),
        truncation,
        removed_sections,
        noise_sources,
        warnings,
        metadata: Metadata {
            elapsed_ms: started.elapsed().as_millis(),
            engine: "contextclean-core".to_string(),
        },
    }
}

fn stats_for(input: &str) -> InputStats {
    InputStats {
        bytes: input.len(),
        chars: input.chars().count(),
        tokens: count_tokens(input),
    }
}

fn budget_for(options: &CleanOptions) -> Budget {
    let limit_source = match (options.fit, options.max_tokens) {
        (None, None) => BudgetLimitSource::None,
        (None, Some(_)) => BudgetLimitSource::MaxTokens,
        (Some(_), None) => BudgetLimitSource::Fit,
        (Some(fit), Some(limit)) if limit == fit.max_tokens() => BudgetLimitSource::Fit,
        (Some(_), Some(_)) => BudgetLimitSource::FitAndMaxTokens,
    };

    Budget {
        fit: options.fit,
        model_id: options.fit.map(|fit| fit.model_id().to_string()),
        tokenizer: tokenizer_name().to_string(),
        token_count_is_exact: true,
        preset_limit_tokens: options.fit.map(|fit| fit.max_tokens()),
        effective_limit_tokens: options.max_tokens,
        model_max_output_tokens: options.fit.map(|fit| fit.max_output_tokens()),
        limit_source,
    }
}

fn budget_reason(options: &CleanOptions) -> String {
    match (options.fit, options.max_tokens) {
        (Some(fit), Some(limit)) if limit == fit.max_tokens() => {
            format!("fit {} budget", fit.label())
        }
        (Some(fit), Some(_)) => format!("fit {} and max_tokens budget", fit.label()),
        (Some(fit), None) => format!("fit {} budget", fit.label()),
        _ => "max_tokens budget".to_string(),
    }
}

fn estimate_tokens(input: &str) -> usize {
    if input.is_empty() {
        0
    } else {
        input.chars().count().div_ceil(4)
    }
}

fn metrics_for(input_tokens: usize, output_tokens: usize) -> Metrics {
    let tokens_saved = input_tokens as isize - output_tokens as isize;
    let compression_ratio = if input_tokens == 0 {
        0.0
    } else {
        output_tokens as f64 / input_tokens as f64
    };
    let reduction_percent = if input_tokens == 0 {
        0.0
    } else {
        (1.0 - compression_ratio).max(0.0) * 100.0
    };

    Metrics {
        input_tokens,
        output_tokens,
        tokens_saved,
        compression_ratio,
        reduction_percent,
    }
}

fn normalize_newlines(input: &str) -> String {
    input.replace("\r\n", "\n").replace('\r', "\n")
}

fn remove_control_chars(input: &str) -> String {
    input
        .chars()
        .filter(|ch| *ch == '\n' || *ch == '\t' || !ch.is_control())
        .collect()
}

fn decode_basic_entities(input: &str) -> String {
    input
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}

fn collapse_blank_lines(input: &str) -> String {
    blank_lines_regex().replace_all(input, "\n\n").to_string()
}

fn remove_regex_group(
    input: &str,
    patterns: &[(&str, &Regex)],
    section_kind: RemovedSectionKind,
    label: &str,
    removed_sections: &mut Vec<RemovedSection>,
    noise_sources: &mut Vec<NoiseSource>,
    noise_kind: NoiseSourceKind,
) -> String {
    let mut content = input.to_string();
    for (name, regex) in patterns {
        let matches: Vec<_> = regex
            .find_iter(&content)
            .map(|m| m.as_str().to_string())
            .collect();
        if matches.is_empty() {
            continue;
        }

        let removed_tokens = matches.iter().map(|value| estimate_tokens(value)).sum();
        removed_sections.push(RemovedSection {
            kind: section_kind.clone(),
            label: format!("{label}: {name}"),
            tokens_removed: removed_tokens,
            count: matches.len(),
        });
        noise_sources.push(NoiseSource {
            kind: noise_kind.clone(),
            label: format!("{label}: {name}"),
            tokens_removed: removed_tokens,
        });
        content = regex.replace_all(&content, "\n").to_string();
    }
    content
}

fn remove_html_comments(
    input: &str,
    removed_sections: &mut Vec<RemovedSection>,
    noise_sources: &mut Vec<NoiseSource>,
) -> String {
    let matches: Vec<_> = html_comment_regex()
        .find_iter(input)
        .map(|m| m.as_str().to_string())
        .collect();

    if matches.is_empty() {
        return input.to_string();
    }

    let removed_tokens = matches.iter().map(|value| estimate_tokens(value)).sum();
    removed_sections.push(RemovedSection {
        kind: RemovedSectionKind::HtmlComment,
        label: "HTML comments".to_string(),
        tokens_removed: removed_tokens,
        count: matches.len(),
    });
    noise_sources.push(NoiseSource {
        kind: NoiseSourceKind::HtmlBoilerplate,
        label: "HTML comments".to_string(),
        tokens_removed: removed_tokens,
    });

    html_comment_regex().replace_all(input, "\n").to_string()
}

fn drop_boilerplate_lines(
    input: &str,
    mode: CleanMode,
    removed_sections: &mut Vec<RemovedSection>,
    noise_sources: &mut Vec<NoiseSource>,
) -> String {
    let mut kept = Vec::new();
    let mut removed = 0usize;
    let mut removed_tokens = 0usize;

    for line in input.lines() {
        let should_remove = match mode {
            CleanMode::Light => false,
            CleanMode::Standard => is_short_web_boilerplate_line(line, false),
            CleanMode::Aggressive => is_short_web_boilerplate_line(line, true),
        };

        if should_remove {
            removed += 1;
            removed_tokens += estimate_tokens(line);
        } else {
            kept.push(line);
        }
    }

    if removed > 0 {
        removed_sections.push(RemovedSection {
            kind: RemovedSectionKind::HtmlBoilerplate,
            label: "boilerplate lines".to_string(),
            tokens_removed: removed_tokens,
            count: removed,
        });
        noise_sources.push(NoiseSource {
            kind: NoiseSourceKind::HtmlBoilerplate,
            label: "boilerplate lines".to_string(),
            tokens_removed: removed_tokens,
        });
    }

    kept.join("\n")
}

fn is_short_web_boilerplate_line(line: &str, aggressive: bool) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.len() > 220 || looks_like_signal_line(trimmed) {
        return false;
    }
    if trimmed.contains('<') && trimmed.contains('>') && !looks_like_standalone_noise_html(trimmed)
    {
        return false;
    }

    let lower = trimmed.to_ascii_lowercase();
    let standard_noise = lower == "cookie preferences"
        || lower == "privacy choices"
        || lower == "accept all"
        || lower == "reject all"
        || lower == "manage preferences"
        || lower.contains("accept all cookies")
        || lower.contains("manage cookie preferences")
        || lower.contains("subscribe to our newsletter")
        || lower.contains("newsletter signup")
        || lower.contains("advertisement")
        || lower.contains("sponsored content");
    let aggressive_noise = standard_noise
        || lower == "share this"
        || lower == "skip to content"
        || lower == "skip to main content"
        || lower.contains("all rights reserved");

    if aggressive {
        aggressive_noise
    } else {
        standard_noise
    }
}

fn looks_like_signal_line(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    lower.contains("error")
        || lower.contains("warning")
        || lower.contains("failed")
        || lower.contains("failure")
        || lower.contains("typeerror")
        || lower.contains("exception")
        || lower.contains("src/")
        || lower.contains(".rs:")
        || lower.contains(".ts:")
        || lower.contains(".js:")
}

fn looks_like_standalone_noise_html(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    (lower.contains("cookie") || lower.contains("newsletter") || lower.contains("advertisement"))
        && !(lower.contains("<main") || lower.contains("<article"))
}

fn strip_code_comment_lines(
    input: &str,
    removed_sections: &mut Vec<RemovedSection>,
    noise_sources: &mut Vec<NoiseSource>,
) -> String {
    let mut kept = Vec::new();
    let mut removed = 0usize;
    let mut removed_tokens = 0usize;

    for line in input.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
            removed += 1;
            removed_tokens += estimate_tokens(line);
        } else {
            kept.push(line);
        }
    }

    if removed > 0 {
        removed_sections.push(RemovedSection {
            kind: RemovedSectionKind::CodeComment,
            label: "code comment lines".to_string(),
            tokens_removed: removed_tokens,
            count: removed,
        });
        noise_sources.push(NoiseSource {
            kind: NoiseSourceKind::CodeComments,
            label: "code comment lines".to_string(),
            tokens_removed: removed_tokens,
        });
    }

    kept.join("\n")
}

fn redact_secrets(
    input: &str,
    removed_sections: &mut Vec<RemovedSection>,
    noise_sources: &mut Vec<NoiseSource>,
    warnings: &mut Vec<Warning>,
) -> String {
    let mut content = input.to_string();
    let mut redaction_count = 0usize;

    for regex in [
        private_key_regex(),
        assignment_secret_regex(),
        whitespace_secret_regex(),
        url_query_secret_regex(),
        bearer_token_regex(),
        jwt_regex(),
        provider_token_regex(),
    ] {
        let matches = regex.find_iter(&content).count();
        if matches > 0 {
            redaction_count += matches;
            content = regex.replace_all(&content, "[REDACTED_SECRET]").to_string();
        }
    }

    if redaction_count > 0 {
        removed_sections.push(RemovedSection {
            kind: RemovedSectionKind::Secret,
            label: "secret-like values".to_string(),
            tokens_removed: 0,
            count: redaction_count,
        });
        noise_sources.push(NoiseSource {
            kind: NoiseSourceKind::Secret,
            label: "secret-like values redacted".to_string(),
            tokens_removed: 0,
        });
        warnings.push(Warning {
            code: "secrets_redacted".to_string(),
            message: format!("redacted {redaction_count} secret-like value(s)"),
            severity: WarningSeverity::Warning,
        });
    }

    content
}

fn drop_aggressive_noise(input: &str, removed_sections: &mut Vec<RemovedSection>) -> String {
    let mut kept = Vec::new();
    let mut removed = 0usize;
    let mut removed_tokens = 0usize;

    for line in input.lines() {
        let trimmed = line.trim();
        let looks_like_badge = trimmed.starts_with("[![") || trimmed.contains("shields.io");
        let decorative = !trimmed.is_empty()
            && trimmed.chars().all(|ch| {
                ch == '-' || ch == '=' || ch == '*' || ch == '_' || ch == ' ' || ch == '#'
            })
            && trimmed.len() > 8;

        if looks_like_badge || decorative {
            removed += 1;
            removed_tokens += estimate_tokens(line);
        } else {
            kept.push(line);
        }
    }

    if removed > 0 {
        removed_sections.push(RemovedSection {
            kind: RemovedSectionKind::Other,
            label: "aggressive low-signal formatting".to_string(),
            tokens_removed: removed_tokens,
            count: removed,
        });
    }

    kept.join("\n")
}

fn script_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?is)<script\b[^>]*>.*?</script>").expect("valid regex"))
}

fn style_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?is)<style\b[^>]*>.*?</style>").expect("valid regex"))
}

fn noscript_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?is)<noscript\b[^>]*>.*?</noscript>").expect("valid regex"))
}

fn nav_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?is)<nav\b[^>]*>.*?</nav>").expect("valid regex"))
}

fn footer_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?is)<footer\b[^>]*>.*?</footer>").expect("valid regex"))
}

fn aside_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?is)<aside\b[^>]*>.*?</aside>").expect("valid regex"))
}

fn svg_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?is)<svg\b[^>]*>.*?</svg>").expect("valid regex"))
}

fn html_comment_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?is)<!--.*?-->").expect("valid regex"))
}

fn blank_lines_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"\n{3,}").expect("valid regex"))
}

fn private_key_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r"(?is)-----BEGIN [A-Z ]*PRIVATE KEY-----.*?-----END [A-Z ]*PRIVATE KEY-----")
            .expect("valid regex")
    })
}

fn assignment_secret_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(
            r#"(?i)\b[A-Z0-9_]*(api[_-]?key|token|secret|password|database_url|access_key_id|secret_access_key)\b\s*[:=]\s*["']?[^\s"']{8,}["']?"#,
        )
        .expect("valid regex")
    })
}

fn whitespace_secret_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(
            r#"(?i)\b(api[_-]?key|token|secret|password|passwd|pwd|access[_-]?key|secret[_-]?access[_-]?key)\b\s+["']?[^\s"']{8,}["']?"#,
        )
        .expect("valid regex")
    })
}

fn url_query_secret_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(
            r#"(?i)\b(?:x-amz-signature|x-amz-credential|x-amz-security-token|access_token|refresh_token|api[_-]?key|token|key|signature|sig|session|sessionid|auth|authorization|code)=["']?[^&\s)"']{8,}["']?"#,
        )
        .expect("valid regex")
    })
}

fn bearer_token_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX
        .get_or_init(|| Regex::new(r"(?i)\bbearer\s+[A-Za-z0-9_\-./+=]{16,}").expect("valid regex"))
}

fn jwt_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r"\beyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\b")
            .expect("valid regex")
    })
}

fn provider_token_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(
            r"\b(?:sk-[A-Za-z0-9_-]{20,}|ghp_[A-Za-z0-9_]{20,}|github_pat_[A-Za-z0-9_]{20,}|xox[baprs]-[A-Za-z0-9-]{20,}|AKIA[0-9A-Z]{16}|npm_[A-Za-z0-9]{20,})\b",
        )
        .expect("valid regex")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{CleanMode, OutputFormat};

    fn options(mode: CleanMode) -> CleanOptions {
        CleanOptions {
            mode,
            format: OutputFormat::Markdown,
            max_tokens: None,
            fit: None,
            strip_comments: false,
            redact_secrets: true,
            source_name: None,
        }
    }

    #[test]
    fn light_removes_execution_blocks_but_preserves_body() {
        let result = clean_text(
            "<html><script>alert(1)</script><main>Keep this</main></html>",
            &options(CleanMode::Light),
        );

        assert!(!result.output.content.contains("alert"));
        assert!(result.output.content.contains("Keep this"));
    }

    #[test]
    fn standard_collapses_adjacent_repeated_lines() {
        let result = clean_text(
            "warning: retry\nwarning: retry\nwarning: retry\nunique error",
            &options(CleanMode::Standard),
        );

        assert!(result
            .output
            .content
            .contains("[Repeated 3 times] warning: retry"));
        assert!(result.output.content.contains("unique error"));
    }

    #[test]
    fn keyword_boilerplate_lines_create_noise_sources() {
        let result = clean_text(
            "Keep this article fact.\nCookie preferences\nNewsletter signup",
            &options(CleanMode::Standard),
        );

        assert!(result.output.content.contains("Keep this article fact."));
        assert!(!result.output.content.contains("Cookie preferences"));
        assert!(result
            .removed_sections
            .iter()
            .any(|section| section.label == "boilerplate lines"));
        assert!(result
            .noise_sources
            .iter()
            .any(|source| source.label == "boilerplate lines"));
    }

    #[test]
    fn redacts_sensitive_url_query_values_with_warning() {
        let result = clean_text(
            r#"<main><a href="https://s3.example.com/report.csv?X-Amz-Signature=abc123secret&safe=1">signed report</a></main>"#,
            &options(CleanMode::Standard),
        );

        assert!(result.output.content.contains("[REDACTED_SECRET]"));
        assert!(result.output.content.contains("safe=1"));
        assert!(!result.output.content.contains("abc123secret"));
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.code == "secrets_redacted"));
        assert!(result
            .removed_sections
            .iter()
            .any(|section| matches!(section.kind, RemovedSectionKind::Secret)));
        assert!(result
            .noise_sources
            .iter()
            .any(|source| matches!(source.kind, NoiseSourceKind::Secret)));
    }

    #[test]
    fn boilerplate_line_cleanup_does_not_drop_minified_main_content() {
        let result = clean_text(
            r#"<main><h1>KEEP_MINIFIED_MAIN</h1><p>Important body.</p></main><div id="cookie-banner">Accept all cookies</div>"#,
            &options(CleanMode::Standard),
        );

        assert!(result.output.content.contains("KEEP_MINIFIED_MAIN"));
        assert!(result.output.content.contains("Important body."));
        assert!(!result.output.content.contains("Accept all cookies"));
    }

    #[test]
    fn redacts_secret_like_assignments() {
        let result = clean_text(
            "OPENAI_API_KEY=sk-this-secret-value\nsafe=value",
            &options(CleanMode::Standard),
        );

        assert!(result.output.content.contains("[REDACTED_SECRET]"));
        assert!(!result.output.content.contains("sk-this-secret-value"));
    }

    #[test]
    fn redacts_whitespace_delimited_secrets() {
        let result = clean_text(
            "machine api.example.com login alice password netrcSecretValue123\n",
            &options(CleanMode::Standard),
        );

        assert!(result.output.content.contains("[REDACTED_SECRET]"));
        assert!(!result.output.content.contains("netrcSecretValue123"));
    }

    #[test]
    fn redacts_standalone_provider_tokens() {
        let input = "\
OpenAI sk-abcdefghijklmnopqrstuvwx
GitHub ghp_abcdefghijklmnopqrstuvwx
GitHubPat github_pat_abcdefghijklmnopqrstuvwx
Slack xoxb-abcdefghijklmnopqrstuvwx
AWS AKIAABCDEFGHIJKLMNOP
npm npm_abcdefghijklmnopqrstuvwx
";

        let result = clean_text(input, &options(CleanMode::Standard));

        assert_eq!(
            result.output.content.matches("[REDACTED_SECRET]").count(),
            6
        );
        assert!(!result
            .output
            .content
            .contains("sk-abcdefghijklmnopqrstuvwx"));
        assert!(!result
            .output
            .content
            .contains("ghp_abcdefghijklmnopqrstuvwx"));
        assert!(!result
            .output
            .content
            .contains("github_pat_abcdefghijklmnopqrstuvwx"));
        assert!(!result
            .output
            .content
            .contains("xoxb-abcdefghijklmnopqrstuvwx"));
        assert!(!result.output.content.contains("AKIAABCDEFGHIJKLMNOP"));
        assert!(!result
            .output
            .content
            .contains("npm_abcdefghijklmnopqrstuvwx"));
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.code == "secrets_redacted"));
    }

    #[test]
    fn max_tokens_applies_truncation_footer() {
        let input = "paragraph one\n\nparagraph two with lots and lots and lots and lots of words\n\nparagraph three";
        let mut opts = options(CleanMode::Standard);
        opts.max_tokens = Some(12);

        let result = clean_text(input, &opts);

        assert!(result.truncation.applied);
        assert!(result.output.content.contains("Context Truncated"));
        assert!(result.output.tokens <= 12);
        if result.output.content.contains("Removed ") {
            assert!(result.output.content.contains(&format!(
                "Removed {} tokens",
                result.truncation.tokens_removed
            )));
        }
    }
}
