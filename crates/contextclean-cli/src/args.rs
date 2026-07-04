use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use contextclean_core::{CleanMode, OutputFormat};

#[derive(Debug, Parser)]
#[command(
    name = "ctxclean",
    version,
    about = "Local-first context cleaner for AI agents.",
    long_about = "ContextClean strips obvious context noise, redacts secret-like values, and emits token-budget-aware text, markdown, or JSON for AI agent workflows."
)]
pub struct Cli {
    /// File, directory, or '-' to read from stdin. If omitted, ctxclean reads piped stdin.
    pub input: Option<PathBuf>,

    /// Write cleaned output to a file instead of stdout.
    #[arg(short = 'o', long = "output", visible_alias = "out")]
    pub output: Option<PathBuf>,

    /// Hard ceiling for estimated output tokens. Defaults to unlimited.
    #[arg(short = 't', long = "max-tokens", value_parser = parse_positive_usize)]
    pub max_tokens: Option<usize>,

    /// Optimization depth.
    #[arg(short = 'm', long = "mode", value_enum, default_value_t = CliMode::Standard)]
    pub mode: CliMode,

    /// Output structural layout.
    #[arg(short = 'f', long = "format", value_enum, default_value_t = CliFormat::Markdown)]
    pub format: CliFormat,

    /// Remove obvious code comment lines.
    #[arg(short = 'c', long = "strip-comments")]
    pub strip_comments: bool,

    /// Analyze and print output without writing output files.
    #[arg(long)]
    pub dry_run: bool,

    /// Disable defensive redaction of secret-like values.
    #[arg(long)]
    pub no_redact_secrets: bool,

    /// Overwrite output file if it exists.
    #[arg(long)]
    pub force: bool,

    /// Suppress non-error diagnostics.
    #[arg(short = 'q', long)]
    pub quiet: bool,

    /// Print extra diagnostic details to stderr.
    #[arg(short = 'v', long)]
    pub verbose: bool,
}

impl Cli {
    pub fn validate(&self) -> Result<(), String> {
        if self.quiet && self.verbose {
            return Err("--quiet and --verbose cannot be used together".to_string());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CliMode {
    Light,
    Standard,
    Aggressive,
}

impl From<CliMode> for CleanMode {
    fn from(value: CliMode) -> Self {
        match value {
            CliMode::Light => Self::Light,
            CliMode::Standard => Self::Standard,
            CliMode::Aggressive => Self::Aggressive,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CliFormat {
    Text,
    Markdown,
    Json,
}

impl From<CliFormat> for OutputFormat {
    fn from(value: CliFormat) -> Self {
        match value {
            CliFormat::Text => Self::Text,
            CliFormat::Markdown => Self::Markdown,
            CliFormat::Json => Self::Json,
        }
    }
}

fn parse_positive_usize(value: &str) -> Result<usize, String> {
    let parsed = value
        .parse::<usize>()
        .map_err(|_| "must be a positive integer".to_string())?;
    if parsed == 0 {
        Err("must be greater than 0".to_string())
    } else {
        Ok(parsed)
    }
}
