use std::sync::OnceLock;
use std::time::Instant;

use regex::Regex;

use crate::config::{CleanMode, CleanOptions};
use crate::models::{
    CleanResult, InputStats, Metadata, Metrics, NoiseSource, NoiseSourceKind, OutputBlock,
    RemovedSection, RemovedSectionKind, Truncation, Warning, WarningSeverity,
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
        content = drop_boilerplate_lines(&content, options.mode, &mut removed_sections);
        content = html_to_readable_text(&content);
        content =
            collapse_adjacent_repeated_lines(&content, &mut removed_sections, &mut noise_sources);
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
        content = truncate_to_budget(
            &content,
            max_tokens,
            &mut truncation,
            &mut removed_sections,
            &mut noise_sources,
        );
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
        tokens: estimate_tokens(input),
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
) -> String {
    let mut kept = Vec::new();
    let mut removed = 0usize;
    let mut removed_tokens = 0usize;

    for line in input.lines() {
        let lower = line.to_ascii_lowercase();
        let standard_noise = lower.contains("cookie")
            || lower.contains("newsletter")
            || lower.contains("subscribe")
            || lower.contains("advertisement")
            || lower.contains("privacy choices")
            || lower.contains("accept all");
        let aggressive_noise = standard_noise
            || lower.contains("share this")
            || lower.contains("skip to content")
            || lower.contains("all rights reserved");

        let should_remove = match mode {
            CleanMode::Light => false,
            CleanMode::Standard => standard_noise,
            CleanMode::Aggressive => aggressive_noise,
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
    }

    kept.join("\n")
}

fn html_to_readable_text(input: &str) -> String {
    let with_breaks = block_tag_regex().replace_all(input, "\n");
    let without_tags = tag_regex().replace_all(&with_breaks, "");
    whitespace_regex()
        .replace_all(&without_tags, " ")
        .replace(" \n", "\n")
        .replace("\n ", "\n")
}

fn collapse_adjacent_repeated_lines(
    input: &str,
    removed_sections: &mut Vec<RemovedSection>,
    noise_sources: &mut Vec<NoiseSource>,
) -> String {
    let mut output = Vec::new();
    let mut previous: Option<&str> = None;
    let mut repeat_count = 0usize;
    let mut removed_tokens = 0usize;
    let mut group_count = 0usize;

    let flush = |output: &mut Vec<String>,
                 previous: &mut Option<&str>,
                 repeat_count: &mut usize,
                 removed_tokens: &mut usize,
                 group_count: &mut usize| {
        if let Some(line) = previous.take() {
            if *repeat_count > 1 && !line.trim().is_empty() {
                *group_count += 1;
                *removed_tokens += estimate_tokens(line) * (*repeat_count - 1);
                output.push(format!("[Repeated {} times] {}", *repeat_count, line));
            } else {
                output.push(line.to_string());
            }
        }
        *repeat_count = 0;
    };

    for line in input.lines() {
        let trimmed = line.trim();
        match previous {
            Some(prev) if prev.trim() == trimmed && !trimmed.is_empty() => {
                repeat_count += 1;
            }
            _ => {
                flush(
                    &mut output,
                    &mut previous,
                    &mut repeat_count,
                    &mut removed_tokens,
                    &mut group_count,
                );
                previous = Some(line);
                repeat_count = 1;
            }
        }
    }
    flush(
        &mut output,
        &mut previous,
        &mut repeat_count,
        &mut removed_tokens,
        &mut group_count,
    );

    if group_count > 0 {
        removed_sections.push(RemovedSection {
            kind: RemovedSectionKind::DuplicateLine,
            label: "adjacent repeated lines".to_string(),
            tokens_removed: removed_tokens,
            count: group_count,
        });
        noise_sources.push(NoiseSource {
            kind: NoiseSourceKind::Repetition,
            label: "adjacent repeated lines".to_string(),
            tokens_removed: removed_tokens,
        });
    }

    output.join("\n")
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
        bearer_token_regex(),
        jwt_regex(),
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

fn truncate_to_budget(
    input: &str,
    max_tokens: usize,
    truncation: &mut Truncation,
    removed_sections: &mut Vec<RemovedSection>,
    noise_sources: &mut Vec<NoiseSource>,
) -> String {
    let current_tokens = estimate_tokens(input);
    if max_tokens == 0 {
        truncation.applied = true;
        truncation.tokens_removed = current_tokens;
        truncation.reason = Some("max_tokens budget".to_string());
        removed_sections.push(RemovedSection {
            kind: RemovedSectionKind::Truncated,
            label: "token budget truncation".to_string(),
            tokens_removed: current_tokens,
            count: 1,
        });
        noise_sources.push(NoiseSource {
            kind: NoiseSourceKind::Truncation,
            label: "token budget truncation".to_string(),
            tokens_removed: current_tokens,
        });
        return String::new();
    }

    if current_tokens <= max_tokens {
        return input.to_string();
    }

    let full_footer = format!(
        "\n\n[Context Truncated: Removed {} estimated tokens to fit {} token budget]",
        current_tokens.saturating_sub(max_tokens),
        max_tokens
    );
    let short_footer = "\n\n[Context Truncated]".to_string();
    let footer = if estimate_tokens(&full_footer) < max_tokens {
        full_footer
    } else {
        short_footer
    };
    let footer_tokens = estimate_tokens(&footer);
    let content_budget_tokens = max_tokens.saturating_sub(footer_tokens).max(1);
    let mut kept: String = input.chars().take(content_budget_tokens * 4).collect();

    if let Some(paragraph_boundary) = kept.rfind("\n\n") {
        kept.truncate(paragraph_boundary);
    } else if let Some(line_boundary) = kept.rfind('\n') {
        kept.truncate(line_boundary);
    }

    let mut candidate = format!("{}{}", kept.trim_end(), footer);
    while estimate_tokens(&candidate) > max_tokens && !kept.is_empty() {
        kept.pop();
        candidate = format!("{}{}", kept.trim_end(), footer);
    }

    if estimate_tokens(&candidate) > max_tokens {
        candidate = "[Context Truncated]".to_string();
    }

    let removed_tokens = current_tokens.saturating_sub(estimate_tokens(&kept));

    truncation.applied = true;
    truncation.tokens_removed = removed_tokens;
    truncation.reason = Some("max_tokens budget".to_string());
    removed_sections.push(RemovedSection {
        kind: RemovedSectionKind::Truncated,
        label: "token budget truncation".to_string(),
        tokens_removed: removed_tokens,
        count: 1,
    });
    noise_sources.push(NoiseSource {
        kind: NoiseSourceKind::Truncation,
        label: "token budget truncation".to_string(),
        tokens_removed: removed_tokens,
    });

    candidate
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

fn block_tag_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r"(?is)</?(p|div|section|article|main|li|ul|ol|h[1-6]|tr|td|th|br)\b[^>]*>")
            .expect("valid regex")
    })
}

fn tag_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?is)<[^>]+>").expect("valid regex"))
}

fn blank_lines_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"\n{3,}").expect("valid regex"))
}

fn whitespace_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"[ \t]{2,}").expect("valid regex"))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{CleanMode, OutputFormat};

    fn options(mode: CleanMode) -> CleanOptions {
        CleanOptions {
            mode,
            format: OutputFormat::Markdown,
            max_tokens: None,
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
    fn redacts_secret_like_assignments() {
        let result = clean_text(
            "OPENAI_API_KEY=sk-this-secret-value\nsafe=value",
            &options(CleanMode::Standard),
        );

        assert!(result.output.content.contains("[REDACTED_SECRET]"));
        assert!(!result.output.content.contains("sk-this-secret-value"));
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
    }
}
