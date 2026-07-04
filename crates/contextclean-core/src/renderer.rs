use crate::config::OutputFormat;
use crate::error::ContextCleanError;
use crate::models::CleanResult;

pub fn render_result(
    result: &CleanResult,
    format: OutputFormat,
) -> Result<String, ContextCleanError> {
    match format {
        OutputFormat::Text => Ok(render_text(result)),
        OutputFormat::Markdown => Ok(render_markdown(result)),
        OutputFormat::Json => serde_json::to_string_pretty(result)
            .map_err(|error| ContextCleanError::Serialize(error.to_string())),
    }
}

fn render_text(result: &CleanResult) -> String {
    format!(
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
    )
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

#[cfg(test)]
mod tests {
    use crate::cleaner::clean_text;
    use crate::config::{CleanOptions, OutputFormat};

    use super::render_result;

    #[test]
    fn json_renderer_emits_parseable_json() {
        let options = CleanOptions::default();
        let result = clean_text("hello", &options);
        let rendered = render_result(&result, OutputFormat::Json).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&rendered).unwrap();

        assert_eq!(parsed["version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(parsed["metrics"]["input_tokens"], 2);
    }
}
