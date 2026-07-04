use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::config::{CleanMode, OutputFormat};
use crate::models::{Budget, CleanResult, NoiseSourceKind, RemovedSectionKind, Warning};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextReport {
    pub version: String,
    pub source: Option<String>,
    pub mode: CleanMode,
    pub format: OutputFormat,
    pub budget: Budget,
    pub tokens: ReportTokens,
    pub biggest_noise_sources: Vec<ReportNoiseSource>,
    pub removed_section_summary: Vec<RemovedSectionSummary>,
    pub recommended_command: String,
    pub warnings: Vec<Warning>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ReportTokens {
    pub input: usize,
    pub output: usize,
    pub saved: isize,
    pub compression_ratio: f64,
    pub reduction_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportNoiseSource {
    pub kind: String,
    pub label: String,
    pub tokens_removed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemovedSectionSummary {
    pub kind: String,
    pub count: usize,
    pub tokens_removed: usize,
}

#[derive(Debug, Clone)]
pub struct ReportOptions {
    pub source_arg: Option<String>,
    pub mode: CleanMode,
    pub format: OutputFormat,
    pub max_tokens: Option<usize>,
    pub strip_comments: bool,
    pub include_sensitive: bool,
}

pub fn build_report(result: &CleanResult, options: &ReportOptions) -> ContextReport {
    ContextReport {
        version: result.version.clone(),
        source: result.source.clone(),
        mode: result.mode,
        format: options.format,
        budget: result.budget.clone(),
        tokens: ReportTokens {
            input: result.metrics.input_tokens,
            output: result.metrics.output_tokens,
            saved: result.metrics.tokens_saved,
            compression_ratio: result.metrics.compression_ratio,
            reduction_percent: result.metrics.reduction_percent,
        },
        biggest_noise_sources: biggest_noise_sources(result),
        removed_section_summary: removed_section_summary(result),
        recommended_command: recommended_command(result, options),
        warnings: result.warnings.clone(),
    }
}

pub fn render_report(
    report: &ContextReport,
    format: OutputFormat,
) -> Result<String, crate::ContextCleanError> {
    match format {
        OutputFormat::Json => serde_json::to_string_pretty(report)
            .map_err(|error| crate::ContextCleanError::Serialize(error.to_string())),
        OutputFormat::Text => Ok(render_text_report(report)),
        OutputFormat::Markdown => Ok(render_markdown_report(report)),
    }
}

fn biggest_noise_sources(result: &CleanResult) -> Vec<ReportNoiseSource> {
    let mut sources: Vec<_> = result
        .noise_sources
        .iter()
        .map(|source| ReportNoiseSource {
            kind: noise_kind_label(&source.kind).to_string(),
            label: source.label.clone(),
            tokens_removed: source.tokens_removed,
        })
        .collect();
    sources.sort_by(|a, b| b.tokens_removed.cmp(&a.tokens_removed));
    sources.truncate(5);
    sources
}

fn removed_section_summary(result: &CleanResult) -> Vec<RemovedSectionSummary> {
    let mut grouped: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    for section in &result.removed_sections {
        let key = kind_label(&section.kind).to_string();
        let entry = grouped.entry(key).or_default();
        entry.0 += section.count;
        entry.1 += section.tokens_removed;
    }
    grouped
        .into_iter()
        .map(|(kind, (count, tokens_removed))| RemovedSectionSummary {
            kind,
            count,
            tokens_removed,
        })
        .collect()
}

fn recommended_command(result: &CleanResult, options: &ReportOptions) -> String {
    let source = options
        .source_arg
        .as_deref()
        .or(result.source.as_deref())
        .unwrap_or("-");
    let mut parts = vec!["ctxclean".to_string(), source.to_string()];
    if options.mode != CleanMode::Standard {
        parts.push("--mode".to_string());
        parts.push(format!("{:?}", options.mode).to_ascii_lowercase());
    }
    if let Some(fit) = result.budget.fit {
        parts.push("--fit".to_string());
        parts.push(fit.label().to_string());
    }
    if let Some(max_tokens) = options.max_tokens {
        parts.push("--max-tokens".to_string());
        parts.push(max_tokens.to_string());
    }
    if options.strip_comments {
        parts.push("--strip-comments".to_string());
    }
    if options.include_sensitive {
        parts.push("--include-sensitive".to_string());
    }
    parts.push("--format".to_string());
    parts.push("markdown".to_string());
    parts.join(" ")
}

fn render_text_report(report: &ContextReport) -> String {
    let mut rendered = String::new();
    rendered.push_str("ContextClean Report\n");
    rendered.push_str(&format!(
        "source: {}\n",
        report.source.as_deref().unwrap_or("stdin")
    ));
    rendered.push_str(&format!("input_tokens: {}\n", report.tokens.input));
    rendered.push_str(&format!("output_tokens: {}\n", report.tokens.output));
    rendered.push_str(&format!("tokens_saved: {}\n", report.tokens.saved));
    rendered.push_str(&format!(
        "compression_ratio: {:.3}\n",
        report.tokens.compression_ratio
    ));
    rendered.push_str(&format!(
        "reduction_percent: {:.1}%\n",
        report.tokens.reduction_percent
    ));
    rendered.push_str("biggest_noise_sources:\n");
    for source in &report.biggest_noise_sources {
        rendered.push_str(&format!(
            "- {}: {} tokens ({})\n",
            source.kind, source.tokens_removed, source.label
        ));
    }
    rendered.push_str("removed_section_summary:\n");
    for section in &report.removed_section_summary {
        rendered.push_str(&format!(
            "- {}: {} tokens removed across {} item(s)\n",
            section.kind, section.tokens_removed, section.count
        ));
    }
    if !report.warnings.is_empty() {
        rendered.push_str("warnings:\n");
        for warning in &report.warnings {
            rendered.push_str(&format!("- {}: {}\n", warning.code, warning.message));
        }
    }
    rendered.push_str(&format!(
        "recommended_command: {}\n",
        report.recommended_command
    ));
    rendered
}

fn render_markdown_report(report: &ContextReport) -> String {
    let mut rendered = String::new();
    rendered.push_str("# ContextClean Report\n\n");
    rendered.push_str("## Token Summary\n\n");
    rendered.push_str("| Field | Value |\n|---|---:|\n");
    rendered.push_str(&format!("| Input tokens | {} |\n", report.tokens.input));
    rendered.push_str(&format!("| Output tokens | {} |\n", report.tokens.output));
    rendered.push_str(&format!("| Tokens saved | {} |\n", report.tokens.saved));
    rendered.push_str(&format!(
        "| Compression ratio | {:.3} |\n",
        report.tokens.compression_ratio
    ));
    rendered.push_str(&format!(
        "| Reduction | {:.1}% |\n",
        report.tokens.reduction_percent
    ));

    rendered.push_str("\n## Biggest Noise Sources\n\n");
    rendered.push_str("| Kind | Label | Tokens removed |\n|---|---|---:|\n");
    for source in &report.biggest_noise_sources {
        rendered.push_str(&format!(
            "| {} | {} | {} |\n",
            source.kind, source.label, source.tokens_removed
        ));
    }

    rendered.push_str("\n## Removed Section Summary\n\n");
    rendered.push_str("| Kind | Count | Tokens removed |\n|---|---:|---:|\n");
    for section in &report.removed_section_summary {
        rendered.push_str(&format!(
            "| {} | {} | {} |\n",
            section.kind, section.count, section.tokens_removed
        ));
    }

    rendered.push_str("\n## Recommended Command\n\n```bash\n");
    rendered.push_str(&report.recommended_command);
    rendered.push_str("\n```\n");

    if !report.warnings.is_empty() {
        rendered.push_str("\n## Warnings\n\n");
        rendered.push_str("| Code | Message |\n|---|---|\n");
        for warning in &report.warnings {
            rendered.push_str(&format!("| {} | {} |\n", warning.code, warning.message));
        }
    }
    rendered
}

fn noise_kind_label(kind: &NoiseSourceKind) -> &'static str {
    match kind {
        NoiseSourceKind::HtmlBoilerplate => "html_boilerplate",
        NoiseSourceKind::Repetition => "repetition",
        NoiseSourceKind::StackTrace => "stack_trace",
        NoiseSourceKind::LogNoise => "log_noise",
        NoiseSourceKind::CodeComments => "code_comments",
        NoiseSourceKind::Secret => "secret",
        NoiseSourceKind::Truncation => "truncation",
        NoiseSourceKind::Other => "other",
    }
}

fn kind_label(kind: &RemovedSectionKind) -> &'static str {
    match kind {
        RemovedSectionKind::HtmlExecutionBlock => "html_execution_block",
        RemovedSectionKind::HtmlBoilerplate => "html_boilerplate",
        RemovedSectionKind::HtmlComment => "html_comment",
        RemovedSectionKind::DuplicateLine => "duplicate_line",
        RemovedSectionKind::StackFrame => "stack_frame",
        RemovedSectionKind::LogNoise => "log_noise",
        RemovedSectionKind::CodeComment => "code_comment",
        RemovedSectionKind::Secret => "secret",
        RemovedSectionKind::Truncated => "truncated",
        RemovedSectionKind::Other => "other",
    }
}

#[cfg(test)]
mod tests {
    use crate::{clean_text, CleanOptions, OutputFormat};

    use super::*;

    #[test]
    fn report_projects_clean_result_metrics() {
        let options = CleanOptions {
            format: OutputFormat::Json,
            ..Default::default()
        };
        let result = clean_text("warning\nwarning\nerror", &options);
        let report = build_report(
            &result,
            &ReportOptions {
                source_arg: Some("build.log".to_string()),
                mode: options.mode,
                format: OutputFormat::Json,
                max_tokens: None,
                strip_comments: false,
                include_sensitive: false,
            },
        );

        assert_eq!(report.tokens.input, result.metrics.input_tokens);
        assert!(report.recommended_command.contains("ctxclean build.log"));
        assert!(report
            .biggest_noise_sources
            .iter()
            .all(|source| source.kind == "repetition"));
    }
}
