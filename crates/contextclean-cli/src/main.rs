use clap::Parser;
use contextclean_cli::args::{Cli, CliCommand, CliMode};
use contextclean_cli::exit::{EXIT_OK, EXIT_USAGE};
use contextclean_cli::mcp::run_mcp_server;
use contextclean_cli::support::{run_clean_path, run_report_path, ArtifactKind, CliSupportError};

fn main() {
    std::process::exit(match run() {
        Ok(()) => EXIT_OK,
        Err(AppError::Support(error)) => {
            eprintln!("error: {error}");
            error.exit_code()
        }
        Err(AppError::Usage(message)) => {
            eprintln!("error: {message}");
            EXIT_USAGE
        }
    });
}

fn run() -> Result<(), AppError> {
    let cli = Cli::parse();
    cli.validate().map_err(AppError::Usage)?;

    match cli.command {
        Some(CliCommand::Gha(args)) => run_clean_path(
            Some(&args.input),
            &args.options,
            CliMode::Aggressive,
            ArtifactKind::CleanedOutput,
        )
        .map_err(AppError::Support),
        Some(CliCommand::Repo(args)) => run_clean_path(
            Some(&args.input),
            &args.options,
            CliMode::Standard,
            ArtifactKind::CleanedOutput,
        )
        .map_err(AppError::Support),
        Some(CliCommand::Mcp(_)) => run_mcp_server().map_err(AppError::Support),
        Some(CliCommand::Report(args)) => {
            run_report_path(&args.input, &args.options, CliMode::Standard)
                .map_err(AppError::Support)
        }
        None => run_clean_path(
            cli.clean.input.as_deref(),
            &cli.clean.options,
            CliMode::Standard,
            ArtifactKind::CleanedOutput,
        )
        .map_err(AppError::Support),
    }
}

#[derive(Debug)]
enum AppError {
    Support(CliSupportError),
    Usage(String),
}
