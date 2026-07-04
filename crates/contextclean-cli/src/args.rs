use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use contextclean_core::{CleanMode, FitModel, OutputFormat, MIN_EXPLAINABLE_TRUNCATION_TOKENS};

#[derive(Debug, Parser)]
#[command(
    name = "ctxclean",
    version,
    about = "Local-first context cleaner for AI agents.",
    long_about = "ContextClean strips obvious context noise, redacts secret-like values, and emits token-budget-aware text, markdown, or JSON for AI agent workflows."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<CliCommand>,

    #[command(flatten)]
    pub clean: CleanArgs,
}

#[derive(Debug, Subcommand)]
pub enum CliCommand {
    /// Clean a GitHub Actions failure log for AI debugging workflows.
    Gha(GhaArgs),
    /// Build a safe context pack from a repository or project directory.
    Repo(RepoArgs),
    /// Run a minimal stdio MCP server for agent integrations.
    Mcp(McpArgs),
    /// Explain token savings, removed sections, and recommended cleanup command.
    Report(ReportArgs),
}

#[derive(Debug, Clone, Args)]
pub struct CleanArgs {
    /// File, directory, or '-' to read from stdin. If omitted, ctxclean reads piped stdin.
    pub input: Option<PathBuf>,

    #[command(flatten)]
    pub options: SharedArgs,
}

#[derive(Debug, Clone, Args)]
pub struct ReportArgs {
    /// File, directory, or '-' to analyze for a context report.
    pub input: PathBuf,

    #[command(flatten)]
    pub options: SharedArgs,
}

#[derive(Debug, Clone, Args)]
pub struct GhaArgs {
    /// GitHub Actions log file, or '-' to read from stdin.
    pub input: PathBuf,

    #[command(flatten)]
    pub options: SharedArgs,
}

#[derive(Debug, Clone, Args)]
pub struct RepoArgs {
    /// Repository or project directory to pack safely.
    pub input: PathBuf,

    #[command(flatten)]
    pub options: SharedArgs,
}

#[derive(Debug, Clone, Args)]
pub struct McpArgs {}

#[derive(Debug, Clone, Args)]
pub struct SharedArgs {
    /// Write output to a file instead of stdout.
    #[arg(short = 'o', long = "output", visible_alias = "out")]
    pub output: Option<PathBuf>,

    /// Hard ceiling for output content tokens. Defaults to unlimited unless --fit is provided.
    #[arg(short = 't', long = "max-tokens", value_parser = parse_positive_usize)]
    pub max_tokens: Option<usize>,

    /// Fit output for a known model budget.
    #[arg(long = "fit", value_enum)]
    pub fit: Option<CliFit>,

    /// Optimization depth.
    #[arg(short = 'm', long = "mode", value_enum)]
    pub mode: Option<CliMode>,

    /// Output structural layout.
    #[arg(short = 'f', long = "format", value_enum, default_value_t = CliFormat::Markdown)]
    pub format: CliFormat,

    /// Remove obvious code comment lines.
    #[arg(short = 'c', long = "strip-comments")]
    pub strip_comments: bool,

    /// Analyze and print output without writing output files.
    #[arg(long)]
    pub dry_run: bool,

    /// Keep default secret redaction enabled explicitly.
    #[arg(long)]
    pub redact_secrets: bool,

    /// Disable defensive redaction of secret-like values.
    #[arg(long)]
    pub no_redact_secrets: bool,

    /// Include sensitive paths such as .env files, private keys, and credential dirs.
    #[arg(long)]
    pub include_sensitive: bool,

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
        match &self.command {
            Some(CliCommand::Gha(gha)) => gha.options.validate(),
            Some(CliCommand::Repo(repo)) => repo.options.validate(),
            Some(CliCommand::Mcp(_)) => Ok(()),
            Some(CliCommand::Report(report)) => report.options.validate(),
            None => self.clean.options.validate(),
        }
    }
}

impl SharedArgs {
    pub fn validate(&self) -> Result<(), String> {
        if self.quiet && self.verbose {
            return Err("--quiet and --verbose cannot be used together".to_string());
        }
        if self.redact_secrets && self.no_redact_secrets {
            return Err(
                "--redact-secrets and --no-redact-secrets cannot be used together".to_string(),
            );
        }
        if let (Some(fit), Some(max_tokens)) = (self.fit, self.max_tokens) {
            let preset = FitModel::from(fit).max_tokens();
            if max_tokens > preset {
                return Err(format!(
                    "--max-tokens {max_tokens} exceeds --fit {} preset limit of {preset}",
                    FitModel::from(fit).label()
                ));
            }
        }
        Ok(())
    }

    pub fn effective_max_tokens(&self) -> Option<usize> {
        self.max_tokens
            .or_else(|| self.fit.map(|fit| FitModel::from(fit).max_tokens()))
    }

    pub fn effective_mode(&self, default: CliMode) -> CliMode {
        self.mode.unwrap_or(default)
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

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CliFit {
    #[value(name = "gpt-4.1")]
    Gpt41,
    #[value(name = "claude-sonnet")]
    ClaudeSonnet,
    #[value(name = "gemini-pro")]
    GeminiPro,
}

impl From<CliFit> for FitModel {
    fn from(value: CliFit) -> Self {
        match value {
            CliFit::Gpt41 => Self::Gpt41,
            CliFit::ClaudeSonnet => Self::ClaudeSonnet,
            CliFit::GeminiPro => Self::GeminiPro,
        }
    }
}

pub fn parse_positive_usize(value: &str) -> Result<usize, String> {
    let parsed = value
        .parse::<usize>()
        .map_err(|_| "must be a positive integer".to_string())?;
    if parsed == 0 {
        Err("must be greater than 0".to_string())
    } else if parsed < MIN_EXPLAINABLE_TRUNCATION_TOKENS {
        Err(format!(
            "must be at least {MIN_EXPLAINABLE_TRUNCATION_TOKENS} so truncation can be explained"
        ))
    } else {
        Ok(parsed)
    }
}
