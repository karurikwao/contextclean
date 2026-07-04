use crate::budget::count_tokens;
use crate::config::OutputFormat;
use crate::error::ContextCleanError;
use crate::models::CleanResult;

pub fn render_result(
    result: &CleanResult,
    format: OutputFormat,
) -> Result<String, ContextCleanError> {
    match format {
        OutputFormat::Text => Ok(enforce_render_budget(
            result,
            render_text(result),
            OutputFormat::Text,
        )),
        OutputFormat::Markdown => Ok(enforce_render_budget(
            result,
            render_markdown(result),
            OutputFormat::Markdown,
        )),
        OutputFormat::Json => serde_json::to_string_pretty(result)
            .map_err(|error| ContextCleanError::Serialize(error.to_string())),
    }
}

fn render_text(result: &CleanResult) -> String {
    let mut rendered = format!(
        "{}\n\n---\nctxclean v{}\nmode: {:?}\nformat: {:?}\ninput_tokens: {}\noutput_tokens: {}\ntokens_saved: {}\nreduction_percent: {:.1}%\ntruncation: {}\n",
        result.output.content,
        result.version,
        result.mode,
        result.format,
        result.metrics.input_tokens,
        result.metrics.output_tokens,
        result.metrics.tokens_saved,
        result.metrics.reduction_percent,
        result.truncation.applied
    );
    if !result.warnings.is_empty() {
        rendered.push_str("warnings:\n");
        for warning in &result.warnings {
            rendered.push_str(&format!("- {}: {}\n", warning.code, warning.message));
        }
    }
    rendered
}

fn render_markdown(result: &CleanResult) -> String {
    let mut rendered = String::new();
    rendered.push_str("# Cleaned Context\n\n");
    rendered.push_str(&result.output.content);
    rendered.push_str("\n\n---\n\n## Metrics\n\n");
    rendered.push_str("| Field | Value |\n|---|---:|\n");
    rendered.push_str(&format!("| Mode | {:?} |\n", result.mode));
    rendered.push_str(&format!("| Format | {:?} |\n", result.format));
    rendered.push_str(&format!(
        "| Input tokens | {} |\n",
        result.metrics.input_tokens
    ));
    rendered.push_str(&format!(
        "| Output tokens | {} |\n",
        result.metrics.output_tokens
    ));
    rendered.push_str(&format!(
        "| Tokens saved | {} |\n",
        result.metrics.tokens_saved
    ));
    rendered.push_str(&format!(
        "| Reduction | {:.1}% |\n",
        result.metrics.reduction_percent
    ));
    rendered.push_str(&format!(
        "| Truncation applied | {} |\n",
        result.truncation.applied
    ));

    if !result.removed_sections.is_empty() {
        rendered.push_str("\n## Removed Sections\n\n");
        rendered.push_str("| Kind | Label | Tokens removed | Count |\n|---|---|---:|---:|\n");
        for section in &result.removed_sections {
            rendered.push_str(&format!(
                "| {:?} | {} | {} | {} |\n",
                section.kind, section.label, section.tokens_removed, section.count
            ));
        }
    }

    if !result.warnings.is_empty() {
        rendered.push_str("\n## Warnings\n\n");
        rendered.push_str("| Severity | Code | Message |\n|---|---|---|\n");
        for warning in &result.warnings {
            rendered.push_str(&format!(
                "| {:?} | {} | {} |\n",
                warning.severity, warning.code, warning.message
            ));
        }
    }

    rendered
}

fn enforce_render_budget(result: &CleanResult, rendered: String, format: OutputFormat) -> String {
    let Some(max_tokens) = result.truncation.limit_tokens else {
        return rendered;
    };
    if estimate_tokens(&rendered) <= max_tokens {
        return rendered;
    }

    if result.truncation.applied && estimate_tokens(&result.output.content) <= max_tokens {
        return result.output.content.clone();
    }

    match format {
        OutputFormat::Text => render_budgeted_human(
            "",
            &result.output.content,
            &format!(
                "\n\n---\nctxclean truncation={} output_tokens={}\n",
                result.truncation.applied, result.metrics.output_tokens
            ),
            max_tokens,
        ),
        OutputFormat::Markdown => render_budgeted_human(
            "# Cleaned Context\n\n",
            &result.output.content,
            &format!(
                "\n\n---\n\n_ctxclean: truncation={}, output_tokens={}_\n",
                result.truncation.applied, result.metrics.output_tokens
            ),
            max_tokens,
        ),
        OutputFormat::Json => rendered,
    }
}

fn render_budgeted_human(prefix: &str, content: &str, footer: &str, max_tokens: usize) -> String {
    let overhead_tokens = estimate_tokens(prefix) + estimate_tokens(footer);
    let mut body = fit_to_estimated_tokens(content, max_tokens.saturating_sub(overhead_tokens));

    loop {
        let candidate = format!("{prefix}{}{footer}", body.trim_end());
        if estimate_tokens(&candidate) <= max_tokens {
            return candidate;
        }
        if body.is_empty() {
            return fit_to_estimated_tokens(&candidate, max_tokens);
        }
        body.pop();
    }
}

fn fit_to_estimated_tokens(input: &str, max_tokens: usize) -> String {
    input.chars().take(max_tokens.saturating_mul(4)).collect()
}

fn estimate_tokens(input: &str) -> usize {
    count_tokens(input)
}

#[cfg(test)]
mod tests {
    use crate::cleaner::clean_text;
    use crate::config::{CleanOptions, OutputFormat};

    use super::{estimate_tokens, render_result};

    #[test]
    fn json_renderer_emits_parseable_json() {
        let options = CleanOptions::default();
        let result = clean_text("hello", &options);
        let rendered = render_result(&result, OutputFormat::Json).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&rendered).unwrap();

        assert_eq!(parsed["version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(parsed["metrics"]["input_tokens"], estimate_tokens("hello"));
    }

    #[test]
    fn markdown_renderer_respects_requested_budget() {
        let options = CleanOptions {
            max_tokens: Some(24),
            ..Default::default()
        };
        let result = clean_text(
            "line one\nline two\nline three\nline four\nline five\nline six\nline seven",
            &options,
        );

        let rendered = render_result(&result, OutputFormat::Markdown).unwrap();

        assert!(estimate_tokens(&rendered) <= 24);
    }
}
