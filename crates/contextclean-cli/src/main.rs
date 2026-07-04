mod args;
mod exit;

use std::fs;
use std::io::{self, Write};

use args::Cli;
use clap::Parser;
use contextclean_core::{clean_text, read_source, render_result, CleanOptions, ContextCleanError};
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

    if cli.no_redact_secrets && !cli.quiet {
        eprintln!("warning: secret redaction disabled by --no-redact-secrets");
    }

    let source = read_source(cli.input.as_deref()).map_err(AppError::Core)?;
    let options = CleanOptions {
        mode: cli.mode.into(),
        format: cli.format.into(),
        max_tokens: cli.max_tokens,
        strip_comments: cli.strip_comments,
        redact_secrets: !cli.no_redact_secrets,
        source_name: source.name.clone(),
    };

    let mut result = clean_text(&source.content, &options);
    result.warnings.extend(source.warnings);

    if cli.verbose && !cli.quiet {
        eprintln!(
            "source: {}; input_tokens: {}; output_tokens: {}; elapsed_ms: {}",
            result.source.as_deref().unwrap_or("unknown"),
            result.metrics.input_tokens,
            result.metrics.output_tokens,
            result.metadata.elapsed_ms
        );
    }

    let rendered = render_result(&result, options.format).map_err(AppError::Core)?;

    if cli.dry_run {
        if let Some(output) = &cli.output {
            if !cli.quiet {
                eprintln!("dry run: not writing output to {}", output.display());
            }
        }
        let mut stdout = io::stdout().lock();
        writeln!(stdout, "{rendered}").map_err(|error| AppError::Render(error.to_string()))?;
        return Ok(());
    }

    if let Some(output) = cli.output {
        if output.exists() && !cli.force {
            return Err(AppError::Core(ContextCleanError::OutputExists(output)));
        }
        fs::write(&output, rendered)
            .map_err(|error| AppError::Core(ContextCleanError::WriteOutput(error.to_string())))?;
        if !cli.quiet {
            eprintln!("wrote cleaned output to {}", output.display());
        }
    } else {
        let mut stdout = io::stdout().lock();
        writeln!(stdout, "{rendered}").map_err(|error| AppError::Render(error.to_string()))?;
    }

    Ok(())
}

#[derive(Debug)]
enum AppError {
    Core(ContextCleanError),
    Usage(String),
    Render(String),
}
