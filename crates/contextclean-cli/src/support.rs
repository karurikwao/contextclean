use std::fs;
use std::io::{self, Write};
use std::path::Path;

use contextclean_core::{
    build_report, clean_text, read_source_with_options, render_report, render_result, CleanMode,
    CleanOptions, ContextCleanError, OutputFormat, ReadOptions, ReportOptions,
};

use crate::args::{CliMode, SharedArgs};

#[derive(Debug, Clone, Copy)]
pub enum ArtifactKind {
    CleanedOutput,
    Report,
}

impl ArtifactKind {
    fn label(self) -> &'static str {
        match self {
            Self::CleanedOutput => "cleaned output",
            Self::Report => "report",
        }
    }
}

pub fn run_clean_path(
    input: Option<&Path>,
    args: &SharedArgs,
    default_mode: CliMode,
    artifact: ArtifactKind,
) -> Result<(), CliSupportError> {
    warn_redaction_disabled(args);
    let source = read_source_with_options(
        input,
        &ReadOptions {
            include_sensitive: args.include_sensitive,
        },
    )
    .map_err(CliSupportError::Core)?;
    let options = clean_options(&source.name, args, default_mode);
    let mut result = clean_text(&source.content, &options);
    result.warnings.extend(source.warnings);
    print_verbose(&result, args);
    let rendered = render_result(&result, options.format).map_err(CliSupportError::Core)?;
    write_rendered(rendered, args, artifact.label())
}

pub fn run_report_path(
    input: &Path,
    args: &SharedArgs,
    default_mode: CliMode,
) -> Result<(), CliSupportError> {
    warn_redaction_disabled(args);
    let source = read_source_with_options(
        Some(input),
        &ReadOptions {
            include_sensitive: args.include_sensitive,
        },
    )
    .map_err(CliSupportError::Core)?;
    let options = clean_options(&source.name, args, default_mode);
    let mut result = clean_text(&source.content, &options);
    result.warnings.extend(source.warnings);
    print_verbose(&result, args);
    let report = build_report(
        &result,
        &ReportOptions {
            source_arg: Some(display_path(input)),
            mode: options.mode,
            format: options.format,
            max_tokens: args.max_tokens,
            strip_comments: options.strip_comments,
            include_sensitive: args.include_sensitive,
        },
    );
    let rendered = render_report(&report, options.format).map_err(CliSupportError::Core)?;
    write_rendered(rendered, args, ArtifactKind::Report.label())
}

pub fn clean_options(
    source_name: &Option<String>,
    args: &SharedArgs,
    default_mode: CliMode,
) -> CleanOptions {
    CleanOptions {
        mode: CleanMode::from(args.effective_mode(default_mode)),
        format: OutputFormat::from(args.format),
        max_tokens: args.effective_max_tokens(),
        fit: args.fit.map(Into::into),
        strip_comments: args.strip_comments,
        redact_secrets: !args.no_redact_secrets,
        source_name: source_name.clone(),
    }
}

pub fn write_rendered(
    rendered: String,
    args: &SharedArgs,
    artifact: &str,
) -> Result<(), CliSupportError> {
    if args.dry_run {
        if let Some(output) = &args.output {
            if !args.quiet {
                eprintln!("dry run: not writing output to {}", output.display());
            }
        }
        let mut stdout = io::stdout().lock();
        writeln!(stdout, "{rendered}")
            .map_err(|error| CliSupportError::Render(error.to_string()))?;
        return Ok(());
    }

    if let Some(output) = &args.output {
        if output.exists() && !args.force {
            return Err(CliSupportError::Core(ContextCleanError::OutputExists(
                output.clone(),
            )));
        }
        fs::write(output, rendered).map_err(|error| {
            CliSupportError::Core(ContextCleanError::WriteOutput(error.to_string()))
        })?;
        if !args.quiet {
            eprintln!("wrote {artifact} to {}", output.display());
        }
    } else {
        let mut stdout = io::stdout().lock();
        writeln!(stdout, "{rendered}")
            .map_err(|error| CliSupportError::Render(error.to_string()))?;
    }

    Ok(())
}

pub fn warn_redaction_disabled(args: &SharedArgs) {
    if args.no_redact_secrets && !args.quiet {
        eprintln!("warning: secret redaction disabled by --no-redact-secrets");
    }
}

pub fn print_verbose(result: &contextclean_core::CleanResult, args: &SharedArgs) {
    if args.verbose && !args.quiet {
        eprintln!(
            "source: {}; input_tokens: {}; output_tokens: {}; elapsed_ms: {}",
            result.source.as_deref().unwrap_or("unknown"),
            result.metrics.input_tokens,
            result.metrics.output_tokens,
            result.metadata.elapsed_ms
        );
    }
}

fn display_path(path: &Path) -> String {
    path.display().to_string()
}

#[derive(Debug)]
pub enum CliSupportError {
    Core(ContextCleanError),
    Render(String),
}

impl CliSupportError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Core(error) => error.exit_code(),
            Self::Render(_) => crate::exit::EXIT_PROCESSING,
        }
    }
}

impl std::fmt::Display for CliSupportError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Core(error) => write!(formatter, "{error}"),
            Self::Render(message) => write!(formatter, "{message}"),
        }
    }
}
