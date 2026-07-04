use std::fs;
use std::io::{self, IsTerminal, Read};
use std::path::Path;

use ignore::WalkBuilder;

use crate::error::ContextCleanError;
use crate::models::{Warning, WarningSeverity};

#[derive(Debug, Clone)]
pub struct SourceData {
    pub name: Option<String>,
    pub content: String,
    pub warnings: Vec<Warning>,
}

pub fn read_source(input: Option<&Path>) -> Result<SourceData, ContextCleanError> {
    match input {
        Some(path) if path == Path::new("-") => read_stdin(),
        Some(path) if path.is_file() => read_file(path),
        Some(path) if path.is_dir() => read_directory(path),
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
    let mut content = String::new();
    io::stdin()
        .read_to_string(&mut content)
        .map_err(|error| ContextCleanError::ReadInput(error.to_string()))?;

    if content.trim().is_empty() {
        return Err(ContextCleanError::MissingInput);
    }

    Ok(SourceData {
        name: Some("stdin".to_string()),
        content,
        warnings: Vec::new(),
    })
}

fn read_file(path: &Path) -> Result<SourceData, ContextCleanError> {
    let content = fs::read_to_string(path)
        .map_err(|error| ContextCleanError::ReadInput(format!("{}: {error}", path.display())))?;

    Ok(SourceData {
        name: Some(path.display().to_string()),
        content,
        warnings: Vec::new(),
    })
}

fn read_directory(path: &Path) -> Result<SourceData, ContextCleanError> {
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
        if should_skip_default(entry_path) {
            continue;
        }
        files.push(entry_path.to_path_buf());
    }

    files.sort();

    let mut content = String::new();
    for file in files {
        match fs::read(&file) {
            Ok(bytes) if looks_binary(&bytes) => warnings.push(warning(
                "binary_file_skipped",
                format!("skipped binary file: {}", display_relative(path, &file)),
            )),
            Ok(bytes) => match String::from_utf8(bytes) {
                Ok(text) => {
                    content.push_str(&format!(
                        "\n\n## File: {}\n\n",
                        display_relative(path, &file)
                    ));
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

fn should_skip_default(path: &Path) -> bool {
    let path_text = path
        .to_string_lossy()
        .replace('\\', "/")
        .to_ascii_lowercase();
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    path_text.contains("/.git/")
        || path_text.contains("/.aws/")
        || path_text.contains("/.azure/")
        || path_text.contains("/.docker/")
        || path_text.contains("/.gnupg/")
        || path_text.contains("/.kube/")
        || path_text.contains("/.ssh/")
        || path_text.contains("/node_modules/")
        || path_text.contains("/target/")
        || path_text.contains("/dist/")
        || path_text.contains("/build/")
        || path_text.contains("/coverage/")
        || path_text.contains("/.cache/")
        || path_text.contains("/.next/")
        || path_text.contains("/.turbo/")
        || file_name.starts_with(".env")
        || file_name == ".netrc"
        || file_name == ".npmrc"
        || file_name == ".pypirc"
        || file_name == ".dockercfg"
        || file_name == "credentials"
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

    use super::read_source;

    #[test]
    fn directory_reader_skips_sensitive_defaults() {
        let temp = tempdir().unwrap();
        fs::create_dir_all(temp.path().join("src")).unwrap();
        fs::write(temp.path().join("src/app.rs"), "fn main() {}\n").unwrap();
        fs::write(temp.path().join(".env"), "TOKEN=secret-value\n").unwrap();

        let data = read_source(Some(temp.path())).unwrap();

        assert!(data.content.contains("src/app.rs"));
        assert!(!data.content.contains("secret-value"));
    }
}
