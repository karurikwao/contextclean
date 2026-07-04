use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, ErrorKind, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{self, Command as ProcessCommand, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use clap::Parser;
use contextclean_cli::args::{parse_positive_usize, CliFit, CliFormat, CliMode};
use contextclean_cli::exit::{EXIT_OK, EXIT_PROCESSING, EXIT_USAGE};
use contextclean_core::{clean_text, render_result, CleanMode, CleanOptions, OutputFormat};

const DEFAULT_CAPTURE_LIMIT_BYTES: usize = 4 * 1024 * 1024;
const TIMEOUT_EXIT_CODE: i32 = 124;

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

    /// Maximum bytes to retain from each child output stream while draining pipes.
    #[arg(long = "capture-limit-bytes", value_parser = parse_positive_ctxrun_usize, default_value_t = DEFAULT_CAPTURE_LIMIT_BYTES)]
    capture_limit_bytes: usize,

    /// Kill the child command after this many seconds and clean the captured output.
    #[arg(long = "timeout-seconds", value_parser = parse_positive_ctxrun_usize)]
    timeout_seconds: Option<usize>,

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

    let output = run_child_streaming(&cli)?;

    if output
        .status
        .map(|status| status.success())
        .unwrap_or(false)
        && !output.timed_out
    {
        replay_success_output(&output)?;
        return Ok(EXIT_OK);
    }

    if !cli.quiet {
        let status = output
            .status
            .map(status_label)
            .unwrap_or_else(|| "timeout".to_string());
        eprintln!("ctxrun: command failed with {status}; cleaned output follows");
    }
    if cli.verbose && !cli.quiet {
        eprintln!("ctxrun: command: {}", cli.command.join(" "));
        eprintln!(
            "ctxrun: capture_limit_bytes={} stdout_truncated={} stderr_truncated={}",
            cli.capture_limit_bytes, output.stdout_truncated, output.stderr_truncated
        );
    }

    let captured = captured_output(&cli, &output);
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

    Ok(if output.timed_out {
        TIMEOUT_EXIT_CODE
    } else {
        output
            .status
            .and_then(|status| status.code())
            .unwrap_or(EXIT_PROCESSING)
    })
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

fn parse_positive_ctxrun_usize(value: &str) -> Result<usize, String> {
    let parsed = value
        .parse::<usize>()
        .map_err(|_| "must be a positive integer".to_string())?;
    if parsed == 0 {
        Err("must be greater than 0".to_string())
    } else {
        Ok(parsed)
    }
}

fn run_child_streaming(cli: &CtxrunCli) -> Result<CtxrunOutput, CtxrunError> {
    let mut child = ProcessCommand::new(&cli.command[0])
        .args(&cli.command[1..])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
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

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| CtxrunError::new("failed to capture stdout".to_string(), EXIT_PROCESSING))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| CtxrunError::new("failed to capture stderr".to_string(), EXIT_PROCESSING))?;

    let (stdout_spool_file, stdout_spool_path) = create_spool_file("stdout")?;
    let (stderr_spool_file, stderr_spool_path) = create_spool_file("stderr")?;

    let stdout_reader = read_stream_limited(
        stdout,
        stdout_spool_file,
        stdout_spool_path,
        cli.capture_limit_bytes,
    );
    let stderr_reader = read_stream_limited(
        stderr,
        stderr_spool_file,
        stderr_spool_path,
        cli.capture_limit_bytes,
    );
    let started = Instant::now();
    let timeout = cli
        .timeout_seconds
        .map(|seconds| Duration::from_secs(seconds as u64));
    let mut timed_out = false;
    let status = loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|error| CtxrunError::new(error.to_string(), EXIT_PROCESSING))?
        {
            break Some(status);
        }
        if let Some(timeout) = timeout {
            if started.elapsed() >= timeout {
                timed_out = true;
                let _ = child.kill();
                let status = child
                    .wait()
                    .map_err(|error| CtxrunError::new(error.to_string(), EXIT_PROCESSING))?;
                break Some(status);
            }
        }
        thread::sleep(Duration::from_millis(20));
    };

    let stdout = join_reader(stdout_reader)?;
    let stderr = join_reader(stderr_reader)?;

    Ok(CtxrunOutput {
        status,
        timed_out,
        stdout: stdout.bytes,
        stderr: stderr.bytes,
        stdout_truncated: stdout.truncated,
        stderr_truncated: stderr.truncated,
        stdout_spool: stdout.spool_path,
        stderr_spool: stderr.spool_path,
    })
}

fn read_stream_limited<R>(
    mut reader: R,
    mut spool: File,
    spool_path: PathBuf,
    limit: usize,
) -> thread::JoinHandle<io::Result<CapturedStream>>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut bytes = Vec::new();
        let mut total = 0usize;
        let mut buffer = [0u8; 16 * 1024];
        loop {
            let read = reader.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            spool.write_all(&buffer[..read])?;
            total = total.saturating_add(read);
            if bytes.len() < limit {
                let remaining = limit - bytes.len();
                let to_copy = remaining.min(read);
                bytes.extend_from_slice(&buffer[..to_copy]);
            }
        }
        spool.flush()?;
        Ok(CapturedStream {
            bytes,
            truncated: total > limit,
            spool_path,
        })
    })
}

fn join_reader(
    handle: thread::JoinHandle<io::Result<CapturedStream>>,
) -> Result<CapturedStream, CtxrunError> {
    handle
        .join()
        .map_err(|_| CtxrunError::new("failed to join output reader".to_string(), EXIT_PROCESSING))?
        .map_err(|error| CtxrunError::new(error.to_string(), EXIT_PROCESSING))
}

fn create_spool_file(stream_name: &str) -> Result<(File, PathBuf), CtxrunError> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    for attempt in 0..100 {
        let path = env::temp_dir().join(format!(
            "ctxrun-{}-{timestamp}-{stream_name}-{attempt}.tmp",
            process::id()
        ));
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => return Ok((file, path)),
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(CtxrunError::new(
                    format!("failed to create temporary {stream_name} capture: {error}"),
                    EXIT_PROCESSING,
                ));
            }
        }
    }
    Err(CtxrunError::new(
        format!("failed to create unique temporary {stream_name} capture"),
        EXIT_PROCESSING,
    ))
}

fn replay_success_output(output: &CtxrunOutput) -> Result<(), CtxrunError> {
    let mut stdout = io::stdout();
    replay_stream(&output.stdout_spool, &mut stdout)?;
    let mut stderr = io::stderr();
    replay_stream(&output.stderr_spool, &mut stderr)?;
    Ok(())
}

fn replay_stream<W>(path: &Path, writer: &mut W) -> Result<(), CtxrunError>
where
    W: Write,
{
    let mut file = File::open(path).map_err(|error| {
        CtxrunError::new(
            format!("failed to replay command output: {error}"),
            EXIT_PROCESSING,
        )
    })?;
    io::copy(&mut file, writer).map_err(|error| {
        CtxrunError::new(
            format!("failed to replay command output: {error}"),
            EXIT_PROCESSING,
        )
    })?;
    writer.flush().map_err(|error| {
        CtxrunError::new(
            format!("failed to flush command output: {error}"),
            EXIT_PROCESSING,
        )
    })?;
    Ok(())
}

fn captured_output(cli: &CtxrunCli, output: &CtxrunOutput) -> String {
    let mut captured = format!("$ {}\n", cli.command.join(" "));
    if output.timed_out {
        captured.push_str(&format!(
            "\n[ctxrun: command timed out after {} second(s)]\n",
            cli.timeout_seconds.unwrap_or_default()
        ));
    }
    if !output.stdout.is_empty() {
        captured.push_str("\n## stdout\n\n");
        captured.push_str(&String::from_utf8_lossy(&output.stdout));
        if output.stdout_truncated {
            captured.push_str(&format!(
                "\n[ctxrun: stdout truncated after {} bytes]\n",
                cli.capture_limit_bytes
            ));
        }
    }
    if !output.stderr.is_empty() {
        captured.push_str("\n## stderr\n\n");
        captured.push_str(&String::from_utf8_lossy(&output.stderr));
        if output.stderr_truncated {
            captured.push_str(&format!(
                "\n[ctxrun: stderr truncated after {} bytes]\n",
                cli.capture_limit_bytes
            ));
        }
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

#[derive(Debug)]
struct CapturedStream {
    bytes: Vec<u8>,
    truncated: bool,
    spool_path: PathBuf,
}

#[derive(Debug)]
struct CtxrunOutput {
    status: Option<ExitStatus>,
    timed_out: bool,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    stdout_truncated: bool,
    stderr_truncated: bool,
    stdout_spool: PathBuf,
    stderr_spool: PathBuf,
}

impl Drop for CtxrunOutput {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.stdout_spool);
        let _ = fs::remove_file(&self.stderr_spool);
    }
}
