use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CleanMode {
    Light,
    Standard,
    Aggressive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    Text,
    Markdown,
    Json,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanOptions {
    pub mode: CleanMode,
    pub format: OutputFormat,
    pub max_tokens: Option<usize>,
    pub strip_comments: bool,
    pub redact_secrets: bool,
    pub source_name: Option<String>,
}

impl Default for CleanOptions {
    fn default() -> Self {
        Self {
            mode: CleanMode::Standard,
            format: OutputFormat::Markdown,
            max_tokens: None,
            strip_comments: false,
            redact_secrets: true,
            source_name: None,
        }
    }
}
