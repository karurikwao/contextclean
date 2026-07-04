use std::fs;
use std::io::{self, IsTerminal, Read};
use std::path::Path;

use ignore::WalkBuilder;

use crate::error::ContextCleanError;
use crate::models::{Warning, WarningSeverity};

const MAX_FILE_BYTES: u64 = 1_048_576;
const MAX_DIRECTORY_BYTES: usize = 4 * 1_048_576;
const GENERATED_COMPONENTS: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    "dist",
    "build",
    "coverage",
    ".cache",
    ".next",
    ".turbo",
    ".wrangler",
    ".vite",
    ".pytest_cache",
    "__pycache__",
    ".mypy_cache",
    ".ruff_cache",
    ".parcel-cache",
    ".yarn",
    ".pnpm-store",
    ".venv",
    "venv",
    "tmp",
    "temp",
];
const SENSITIVE_COMPONENTS: &[&str] = &[
    ".aws",
    ".azure",
    ".config",
    ".docker",
    ".gcloud",
    ".gnupg",
    ".gradle",
    ".kube",
    ".local",
    ".m2",
    ".npm",
    ".pulumi",
    ".ssh",
    ".terraform",
];

#[derive(Debug, Clone)]
pub struct SourceData {
    pub name: Option<String>,
    pub content: String,
    pub warnings: Vec<Warning>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ReadOptions {
    pub include_sensitive: bool,
}

pub fn read_source(input: Option<&Path>) -> Result<SourceData, ContextCleanError> {
    read_source_with_options(input, &ReadOptions::default())
}

pub fn read_source_with_options(
    input: Option<&Path>,
    options: &ReadOptions,
) -> Result<SourceData, ContextCleanError> {
    match input {
        Some(path) if path == Path::new("-") => read_stdin(),
        Some(path) if path.is_file() => read_file(path, options),
        Some(path) if path.is_dir() => read_directory(path, options),
        Some(path) => Err(ContextCleanError::InputNotFound(path.display().to_string())),
        None => {
            if io::stdin().is_terminal() {
                Err(ContextCleanError::MissingInput)
            } else {
                read_stdin()
            }
        }
    }
}

fn read_stdin() -> Result<SourceData, ContextCleanError> {
    let mut bytes = Vec::new();
    io::stdin()
        .take(MAX_DIRECTORY_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| ContextCleanError::ReadInput(error.to_string()))?;
    if bytes.len() > MAX_DIRECTORY_BYTES {
        return Err(ContextCleanError::ReadInput(format!(
            "stdin input exceeds {MAX_DIRECTORY_BYTES} bytes"
        )));
    }
    let content = String::from_utf8(bytes)
        .map_err(|_| ContextCleanError::ReadInput("stdin input is not valid UTF-8".to_string()))?;

    if content.trim().is_empty() {
        return Err(ContextCleanError::MissingInput);
    }

    Ok(SourceData {
        name: Some("stdin".to_string()),
        content,
        warnings: Vec::new(),
    })
}

fn read_file(path: &Path, options: &ReadOptions) -> Result<SourceData, ContextCleanError> {
    if !options.include_sensitive && is_sensitive_path(path) {
        return Err(ContextCleanError::ReadInput(format!(
            "sensitive input requires --include-sensitive: {}",
            path.display()
        )));
    }
    let metadata = fs::metadata(path)
        .map_err(|error| ContextCleanError::ReadInput(format!("{}: {error}", path.display())))?;
    if metadata.len() > MAX_FILE_BYTES {
        return Err(ContextCleanError::ReadInput(format!(
            "{} exceeds {MAX_FILE_BYTES} bytes",
            path.display()
        )));
    }

    let bytes = fs::read(path)
        .map_err(|error| ContextCleanError::ReadInput(format!("{}: {error}", path.display())))?;
    if looks_binary(&bytes) {
        return Err(ContextCleanError::ReadInput(format!(
            "{} appears to be a binary file",
            path.display()
        )));
    }
    let content = String::from_utf8(bytes).map_err(|_| {
        ContextCleanError::ReadInput(format!("{} is not valid UTF-8", path.display()))
    })?;

    Ok(SourceData {
        name: Some(path.display().to_string()),
        content,
        warnings: Vec::new(),
    })
}

fn read_directory(path: &Path, options: &ReadOptions) -> Result<SourceData, ContextCleanError> {
    if !options.include_sensitive && is_sensitive_path(path) {
        return Err(ContextCleanError::ReadInput(format!(
            "sensitive input requires --include-sensitive: {}",
            path.display()
        )));
    }
    if is_generated_root(path) {
        return Err(ContextCleanError::ReadInput(format!(
            "generated input is skipped by default: {}",
            path.display()
        )));
    }

    let mut files = Vec::new();
    let mut warnings = Vec::new();
    let mut builder = WalkBuilder::new(path);
    builder
        .add_custom_ignore_filename(".ctxcleanignore")
        .git_ignore(true)
        .git_exclude(true)
        .git_global(true)
        .require_git(false)
        .hidden(false);

    for entry in builder.build() {
        let entry = entry.map_err(|error| ContextCleanError::ReadInput(error.to_string()))?;
        let entry_path = entry.path();
        if !entry_path.is_file() {
            continue;
        }
        let relative_path = entry_path.strip_prefix(path).unwrap_or(entry_path);
        match classify_skip(relative_path) {
            Some(SkipReason::Generated) => continue,
            Some(SkipReason::Sensitive) if !options.include_sensitive => {
                warnings.push(warning(
                    "sensitive_path_skipped",
                    format!(
                        "skipped sensitive path; pass --include-sensitive to include: {}",
                        display_relative(path, entry_path)
                    ),
                ));
                continue;
            }
            _ => {}
        }
        files.push(entry_path.to_path_buf());
    }

    files.sort();

    let mut content = String::new();
    let mut aggregate_bytes = 0usize;
    for file in files {
        let metadata = match fs::metadata(&file) {
            Ok(metadata) if metadata.len() > MAX_FILE_BYTES => {
                warnings.push(warning(
                    "oversized_file_skipped",
                    format!(
                        "skipped file over {} bytes: {}",
                        MAX_FILE_BYTES,
                        display_relative(path, &file)
                    ),
                ));
                continue;
            }
            Ok(metadata) => metadata,
            Err(error) => {
                warnings.push(warning(
                    "file_read_failed",
                    format!("failed to stat {}: {error}", display_relative(path, &file)),
                ));
                continue;
            }
        };

        if aggregate_bytes >= MAX_DIRECTORY_BYTES {
            warnings.push(warning(
                "directory_byte_limit_reached",
                format!(
                    "skipped remaining files after {} aggregate bytes",
                    MAX_DIRECTORY_BYTES
                ),
            ));
            break;
        }

        let header = format!("\n\n## File: {}\n\n", display_relative(path, &file));
        let emitted_bytes = header
            .len()
            .saturating_add(usize::try_from(metadata.len()).unwrap_or(usize::MAX));
        if aggregate_bytes.saturating_add(emitted_bytes) > MAX_DIRECTORY_BYTES {
            warnings.push(warning(
                "directory_byte_limit_reached",
                format!(
                    "skipped file after {} aggregate bytes: {}",
                    MAX_DIRECTORY_BYTES,
                    display_relative(path, &file)
                ),
            ));
            continue;
        }

        match fs::read(&file) {
            Ok(bytes) if looks_binary(&bytes) => warnings.push(warning(
                "binary_file_skipped",
                format!("skipped binary file: {}", display_relative(path, &file)),
            )),
            Ok(bytes) => match String::from_utf8(bytes) {
                Ok(text) => {
                    let emitted_bytes = header.len().saturating_add(text.len());
                    if aggregate_bytes.saturating_add(emitted_bytes) > MAX_DIRECTORY_BYTES {
                        warnings.push(warning(
                            "directory_byte_limit_reached",
                            format!(
                                "skipped file after {} aggregate bytes: {}",
                                MAX_DIRECTORY_BYTES,
                                display_relative(path, &file)
                            ),
                        ));
                        continue;
                    }
                    aggregate_bytes += emitted_bytes;
                    content.push_str(&header);
                    content.push_str(&text);
                }
                Err(_) => warnings.push(warning(
                    "non_utf8_file_skipped",
                    format!("skipped non-utf8 file: {}", display_relative(path, &file)),
                )),
            },
            Err(error) => warnings.push(warning(
                "file_read_failed",
                format!("failed to read {}: {error}", display_relative(path, &file)),
            )),
        }
    }

    if content.trim().is_empty() {
        return Err(ContextCleanError::ReadInput(format!(
            "no readable text files found in {}",
            path.display()
        )));
    }

    Ok(SourceData {
        name: Some(path.display().to_string()),
        content,
        warnings,
    })
}

fn looks_binary(bytes: &[u8]) -> bool {
    bytes.iter().take(1024).any(|byte| *byte == 0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SkipReason {
    Generated,
    Sensitive,
}

fn classify_skip(path: &Path) -> Option<SkipReason> {
    if has_generated_component(path) {
        return Some(SkipReason::Generated);
    }
    if is_sensitive_path(path) {
        return Some(SkipReason::Sensitive);
    }
    None
}

fn is_sensitive_path(path: &Path) -> bool {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    has_sensitive_component(path)
        || file_name.starts_with(".env")
        || file_name == ".netrc"
        || file_name == ".boto"
        || file_name == ".s3cfg"
        || file_name == ".npmrc"
        || file_name == ".pypirc"
        || file_name == ".dockercfg"
        || file_name == "credentials"
        || file_name == "credentials.json"
        || file_name == "application_default_credentials.json"
        || file_name.ends_with(".pem")
        || file_name.ends_with(".key")
        || file_name.ends_with(".p12")
        || file_name.ends_with(".pfx")
        || file_name.ends_with(".crt")
        || file_name.ends_with(".cer")
        || file_name.starts_with("id_rsa")
        || file_name.starts_with("id_ed25519")
        || file_name.ends_with(".kdbx")
        || file_name.ends_with(".sqlite")
        || file_name.ends_with(".db")
        || file_name == ".gitignore"
        || file_name == ".ctxcleanignore"
}

fn has_generated_component(path: &Path) -> bool {
    has_component(path, GENERATED_COMPONENTS)
}

fn has_sensitive_component(path: &Path) -> bool {
    has_component(path, SENSITIVE_COMPONENTS)
}

fn is_generated_root(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| {
            let name = name.to_ascii_lowercase();
            GENERATED_COMPONENTS
                .iter()
                .any(|component| *component == name)
        })
        .unwrap_or(false)
}

fn has_component(path: &Path, names: &[&str]) -> bool {
    path.components().any(|component| {
        let component = component.as_os_str().to_string_lossy().to_ascii_lowercase();
        names.contains(&component.as_str())
    })
}

fn display_relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
        .replace('\\', "/")
}

fn warning(code: &str, message: String) -> Warning {
    Warning {
        code: code.to_string(),
        message,
        severity: WarningSeverity::Warning,
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::{
        read_source, read_source_with_options, ReadOptions, MAX_DIRECTORY_BYTES, MAX_FILE_BYTES,
    };

    #[test]
    fn directory_reader_skips_sensitive_defaults() {
        let temp = tempdir().unwrap();
        fs::create_dir_all(temp.path().join("src")).unwrap();
        fs::write(temp.path().join("src/app.rs"), "fn main() {}\n").unwrap();
        fs::write(temp.path().join(".env"), "TOKEN=secret-value\n").unwrap();

        let data = read_source(Some(temp.path())).unwrap();

        assert!(data.content.contains("src/app.rs"));
        assert!(!data.content.contains("secret-value"));
        assert!(data
            .warnings
            .iter()
            .any(|warning| warning.code == "sensitive_path_skipped"));
    }

    #[test]
    fn directory_reader_skips_hidden_credential_dirs_and_large_files() {
        let temp = tempdir().unwrap();
        fs::create_dir_all(temp.path().join("src")).unwrap();
        fs::create_dir_all(temp.path().join(".gcloud")).unwrap();
        fs::write(temp.path().join("src/app.rs"), "fn main() {}\n").unwrap();
        fs::write(
            temp.path()
                .join(".gcloud/application_default_credentials.json"),
            "GCLOUD_SECRET",
        )
        .unwrap();
        fs::write(
            temp.path().join("large.log"),
            "x".repeat(MAX_FILE_BYTES as usize + 1),
        )
        .unwrap();

        let data = read_source(Some(temp.path())).unwrap();

        assert!(data.content.contains("src/app.rs"));
        assert!(!data.content.contains("GCLOUD_SECRET"));
        assert!(!data.content.contains("large.log"));
        assert!(data
            .warnings
            .iter()
            .any(|warning| warning.code == "oversized_file_skipped"));
    }

    #[test]
    fn include_sensitive_allows_sensitive_files_for_redaction_pipeline() {
        let temp = tempdir().unwrap();
        fs::create_dir_all(temp.path().join("src")).unwrap();
        fs::write(temp.path().join("src/app.rs"), "fn main() {}\n").unwrap();
        fs::write(temp.path().join(".env"), "TOKEN=secret-value\n").unwrap();

        let data = read_source_with_options(
            Some(temp.path()),
            &ReadOptions {
                include_sensitive: true,
            },
        )
        .unwrap();

        assert!(data.content.contains(".env"));
        assert!(data.content.contains("secret-value"));
    }

    #[test]
    fn explicit_sensitive_file_requires_opt_in() {
        let temp = tempdir().unwrap();
        let file = temp.path().join(".env");
        fs::write(&file, "TOKEN=secret-value\n").unwrap();

        let error = read_source(Some(&file)).unwrap_err();

        assert!(error.to_string().contains("--include-sensitive"));
    }

    #[test]
    fn explicit_oversized_file_is_rejected() {
        let temp = tempdir().unwrap();
        let file = temp.path().join("too-large.log");
        fs::write(&file, "x".repeat(MAX_FILE_BYTES as usize + 1)).unwrap();

        let error = read_source(Some(&file)).unwrap_err();

        assert!(error.to_string().contains("exceeds"));
    }

    #[test]
    fn explicit_binary_file_is_rejected() {
        let temp = tempdir().unwrap();
        let file = temp.path().join("binary.log");
        fs::write(&file, b"VISIBLE_BEFORE\0VISIBLE_AFTER").unwrap();

        let error = read_source(Some(&file)).unwrap_err();

        assert!(error.to_string().contains("binary file"));
    }

    #[test]
    fn directory_reader_skips_root_sensitive_directories() {
        let temp = tempdir().unwrap();
        fs::create_dir_all(temp.path().join(".ssh")).unwrap();
        fs::write(
            temp.path().join(".ssh/config"),
            "Host prod\n  IdentityFile secret.pem",
        )
        .unwrap();

        let error = read_source(Some(&temp.path().join(".ssh"))).unwrap_err();

        assert!(error.to_string().contains("--include-sensitive"));
    }

    #[test]
    fn directory_reader_rejects_generated_root_directories() {
        let temp = tempdir().unwrap();
        let generated = temp.path().join("node_modules");
        fs::create_dir_all(&generated).unwrap();
        fs::write(generated.join("package.txt"), "SHOULD_NOT_READ").unwrap();

        let error = read_source(Some(&generated)).unwrap_err();

        assert!(error.to_string().contains("generated input is skipped"));
    }

    #[test]
    fn directory_reader_enforces_aggregate_byte_limit_before_reads() {
        let temp = tempdir().unwrap();
        fs::create_dir_all(temp.path().join("logs")).unwrap();
        for index in 0..5 {
            fs::write(
                temp.path().join("logs").join(format!("{index}.txt")),
                format!("KEEP_{index}\n{}", "x".repeat(950_000)),
            )
            .unwrap();
        }

        let data = read_source(Some(temp.path())).unwrap();

        assert!(data.content.len() <= MAX_DIRECTORY_BYTES + 4096);
        assert!(data
            .warnings
            .iter()
            .any(|warning| warning.code == "directory_byte_limit_reached"));
    }

    #[test]
    fn directory_reader_counts_generated_headers_against_byte_limit() {
        let temp = tempdir().unwrap();
        fs::create_dir_all(temp.path().join("tiny")).unwrap();
        for index in 0..22_000 {
            let long_name = format!(
                "empty-file-with-intentionally-long-name-to-count-generated-header-bytes-{index:05}-{}.txt",
                "x".repeat(120)
            );
            fs::write(temp.path().join("tiny").join(long_name), "").unwrap();
        }

        let data = read_source(Some(temp.path())).unwrap();

        assert!(data.content.len() <= MAX_DIRECTORY_BYTES);
        assert!(data
            .warnings
            .iter()
            .any(|warning| warning.code == "directory_byte_limit_reached"));
    }
}
