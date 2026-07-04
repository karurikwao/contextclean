use std::io::{self, ErrorKind, Write};
use std::process::{Command as ProcessCommand, ExitStatus};

use clap::Parser;
use contextclean_cli::args::{parse_positive_usize, CliFit, CliFormat, CliMode};
use contextclean_cli::exit::{EXIT_OK, EXIT_PROCESSING, EXIT_USAGE};
use contextclean_core::{clean_text, render_result, CleanMode, CleanOptions, OutputFormat};

#[derive(Debug, Parser)]
#[command(
    name = "ctxrun",
    version,
    about = "Run a command and clean failed output for AI agents.",
    long_about = "ctxrun executes a local command. Successful commands pass stdout/stderr through unchanged. Failed commands are compressed with ContextClean while preserving the original exit code."
)]
struct CtxrunCli {
    /// Hard ceiling for cleaned failed-output tokens. Defaults to unlimited unless --fit is provided.
    #[arg(short = 't', long = "max-tokens", value_parser = parse_positive_usize)]
    max_tokens: Option<usize>,

    /// Fit cleaned failed output for a known model budget.
    #[arg(long = "fit", value_enum)]
    fit: Option<CliFit>,

    /// Optimization depth for failed output.
    #[arg(short = 'm', long = "mode", value_enum, default_value_t = CliMode::Aggressive)]
    mode: CliMode,

    /// Output structural layout for failed output.
    #[arg(short = 'f', long = "format", value_enum, default_value_t = CliFormat::Markdown)]
    format: CliFormat,

    /// Remove obvious code comment lines in failed output.
    #[arg(short = 'c', long = "strip-comments")]
    strip_comments: bool,

    /// Keep default secret redaction enabled explicitly.
    #[arg(long)]
    redact_secrets: bool,

    /// Disable defensive redaction of secret-like values.
    #[arg(long)]
    no_redact_secrets: bool,

    /// Suppress ctxrun diagnostics.
    #[arg(short = 'q', long)]
    quiet: bool,

    /// Print extra ctxrun diagnostics.
    #[arg(short = 'v', long)]
    verbose: bool,

    /// Command and arguments to execute.
    #[arg(required = true, trailing_var_arg = true, allow_hyphen_values = true)]
    command: Vec<String>,
}

fn main() {
    std::process::exit(match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("error: {}", error.message);
            error.exit_code
        }
    });
}

fn run() -> Result<i32, CtxrunError> {
    let cli = CtxrunCli::parse();
    validate(&cli).map_err(|message| CtxrunError::new(message, EXIT_USAGE))?;

    let output = ProcessCommand::new(&cli.command[0])
        .args(&cli.command[1..])
        .output()
        .map_err(|error| {
            let exit_code = if error.kind() == ErrorKind::NotFound {
                127
            } else {
                EXIT_PROCESSING
            };
            CtxrunError::new(
                format!("failed to run {}: {error}", cli.command[0]),
                exit_code,
            )
        })?;

    if output.status.success() {
        io::stdout()
            .write_all(&output.stdout)
            .map_err(|error| CtxrunError::new(error.to_string(), EXIT_PROCESSING))?;
        io::stderr()
            .write_all(&output.stderr)
            .map_err(|error| CtxrunError::new(error.to_string(), EXIT_PROCESSING))?;
        return Ok(EXIT_OK);
    }

    if !cli.quiet {
        eprintln!(
            "ctxrun: command failed with {}; cleaned output follows",
            status_label(output.status)
        );
    }
    if cli.verbose && !cli.quiet {
        eprintln!("ctxrun: command: {}", cli.command.join(" "));
    }

    let captured = captured_output(&cli, &output.stdout, &output.stderr);
    let options = CleanOptions {
        mode: CleanMode::from(cli.mode),
        format: OutputFormat::from(cli.format),
        max_tokens: cli.max_tokens.or_else(|| {
            cli.fit
                .map(|fit| contextclean_core::FitModel::from(fit).max_tokens())
        }),
        fit: cli.fit.map(Into::into),
        strip_comments: cli.strip_comments,
        redact_secrets: !cli.no_redact_secrets,
        source_name: Some(format!("ctxrun {}", cli.command.join(" "))),
    };
    let result = clean_text(&captured, &options);
    let rendered = render_result(&result, options.format)
        .map_err(|error| CtxrunError::new(error.to_string(), EXIT_PROCESSING))?;
    writeln!(io::stdout(), "{rendered}")
        .map_err(|error| CtxrunError::new(error.to_string(), EXIT_PROCESSING))?;

    Ok(output.status.code().unwrap_or(EXIT_PROCESSING))
}

fn validate(cli: &CtxrunCli) -> Result<(), String> {
    if cli.quiet && cli.verbose {
        return Err("--quiet and --verbose cannot be used together".to_string());
    }
    if cli.redact_secrets && cli.no_redact_secrets {
        return Err("--redact-secrets and --no-redact-secrets cannot be used together".to_string());
    }
    if let (Some(fit), Some(max_tokens)) = (cli.fit, cli.max_tokens) {
        let preset = contextclean_core::FitModel::from(fit).max_tokens();
        if max_tokens > preset {
            return Err(format!(
                "--max-tokens {max_tokens} exceeds --fit {} preset limit of {preset}",
                contextclean_core::FitModel::from(fit).label()
            ));
        }
    }
    Ok(())
}

fn captured_output(cli: &CtxrunCli, stdout: &[u8], stderr: &[u8]) -> String {
    let mut captured = format!("$ {}\n", cli.command.join(" "));
    if !stdout.is_empty() {
        captured.push_str("\n## stdout\n\n");
        captured.push_str(&String::from_utf8_lossy(stdout));
    }
    if !stderr.is_empty() {
        captured.push_str("\n## stderr\n\n");
        captured.push_str(&String::from_utf8_lossy(stderr));
    }
    captured
}

fn status_label(status: ExitStatus) -> String {
    status
        .code()
        .map(|code| format!("exit code {code}"))
        .unwrap_or_else(|| "terminated by signal".to_string())
}

#[derive(Debug)]
struct CtxrunError {
    message: String,
    exit_code: i32,
}

impl CtxrunError {
    fn new(message: String, exit_code: i32) -> Self {
        Self { message, exit_code }
    }
}
