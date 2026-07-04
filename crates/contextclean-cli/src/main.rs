mod args;
mod exit;

use std::fs;
use std::io::{self, Write};
use std::path::Path;

use args::{CleanArgs, Cli, CliCommand, ReportArgs, SharedArgs};
use clap::Parser;
use contextclean_core::{
    build_report, clean_text, read_source_with_options, render_report, render_result, CleanOptions,
    ContextCleanError, ReadOptions, ReportOptions,
};
use exit::{EXIT_OK, EXIT_PROCESSING, EXIT_USAGE};

fn main() {
    std::process::exit(match run() {
        Ok(()) => EXIT_OK,
        Err(AppError::Core(error)) => {
            eprintln!("error: {error}");
            error.exit_code()
        }
        Err(AppError::Usage(message)) => {
            eprintln!("error: {message}");
            EXIT_USAGE
        }
        Err(AppError::Render(message)) => {
            eprintln!("error: {message}");
            EXIT_PROCESSING
        }
    });
}

fn run() -> Result<(), AppError> {
    let cli = Cli::parse();
    cli.validate().map_err(AppError::Usage)?;

    match cli.command {
        Some(CliCommand::Report(report)) => run_report(report),
        None => run_clean(cli.clean),
    }
}

fn run_clean(args: CleanArgs) -> Result<(), AppError> {
    warn_redaction_disabled(&args.options);
    let source = read_source_with_options(
        args.input.as_deref(),
        &ReadOptions {
            include_sensitive: args.options.include_sensitive,
        },
    )
    .map_err(AppError::Core)?;
    let options = clean_options(&source.name, &args.options);
    let mut result = clean_text(&source.content, &options);
    result.warnings.extend(source.warnings);
    print_verbose(&result, &args.options);
    let rendered = render_result(&result, options.format).map_err(AppError::Core)?;
    write_rendered(rendered, &args.options, "cleaned output")
}

fn run_report(args: ReportArgs) -> Result<(), AppError> {
    warn_redaction_disabled(&args.options);
    let source = read_source_with_options(
        Some(&args.input),
        &ReadOptions {
            include_sensitive: args.options.include_sensitive,
        },
    )
    .map_err(AppError::Core)?;
    let options = clean_options(&source.name, &args.options);
    let mut result = clean_text(&source.content, &options);
    result.warnings.extend(source.warnings);
    print_verbose(&result, &args.options);
    let report = build_report(
        &result,
        &ReportOptions {
            source_arg: Some(display_path(&args.input)),
            mode: options.mode,
            format: options.format,
            max_tokens: args.options.max_tokens,
            strip_comments: options.strip_comments,
            include_sensitive: args.options.include_sensitive,
        },
    );
    let rendered = render_report(&report, options.format).map_err(AppError::Core)?;
    write_rendered(rendered, &args.options, "report")
}

fn clean_options(source_name: &Option<String>, args: &SharedArgs) -> CleanOptions {
    CleanOptions {
        mode: args.mode.into(),
        format: args.format.into(),
        max_tokens: args.effective_max_tokens(),
        fit: args.fit.map(Into::into),
        strip_comments: args.strip_comments,
        redact_secrets: !args.no_redact_secrets,
        source_name: source_name.clone(),
    }
}

fn warn_redaction_disabled(args: &SharedArgs) {
    if args.no_redact_secrets && !args.quiet {
        eprintln!("warning: secret redaction disabled by --no-redact-secrets");
    }
}

fn print_verbose(result: &contextclean_core::CleanResult, args: &SharedArgs) {
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

fn write_rendered(rendered: String, args: &SharedArgs, artifact: &str) -> Result<(), AppError> {
    if args.dry_run {
        if let Some(output) = &args.output {
            if !args.quiet {
                eprintln!("dry run: not writing output to {}", output.display());
            }
        }
        let mut stdout = io::stdout().lock();
        writeln!(stdout, "{rendered}").map_err(|error| AppError::Render(error.to_string()))?;
        return Ok(());
    }

    if let Some(output) = &args.output {
        if output.exists() && !args.force {
            return Err(AppError::Core(ContextCleanError::OutputExists(
                output.clone(),
            )));
        }
        fs::write(output, rendered)
            .map_err(|error| AppError::Core(ContextCleanError::WriteOutput(error.to_string())))?;
        if !args.quiet {
            eprintln!("wrote {artifact} to {}", output.display());
        }
    } else {
        let mut stdout = io::stdout().lock();
        writeln!(stdout, "{rendered}").map_err(|error| AppError::Render(error.to_string()))?;
    }

    Ok(())
}

fn display_path(path: &Path) -> String {
    path.display().to_string()
}

#[derive(Debug)]
enum AppError {
    Core(ContextCleanError),
    Usage(String),
    Render(String),
}
