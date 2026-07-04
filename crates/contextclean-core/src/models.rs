use serde::{Deserialize, Serialize};

use crate::config::{CleanMode, OutputFormat};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanResult {
    pub version: String,
    pub mode: CleanMode,
    pub format: OutputFormat,
    pub source: Option<String>,
    pub input: InputStats,
    pub output: OutputBlock,
    pub metrics: Metrics,
    pub truncation: Truncation,
    pub removed_sections: Vec<RemovedSection>,
    pub noise_sources: Vec<NoiseSource>,
    pub warnings: Vec<Warning>,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct InputStats {
    pub bytes: usize,
    pub chars: usize,
    pub tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputBlock {
    pub bytes: usize,
    pub chars: usize,
    pub tokens: usize,
    pub content: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Metrics {
    pub input_tokens: usize,
    pub output_tokens: usize,
    pub tokens_saved: isize,
    pub compression_ratio: f64,
    pub reduction_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Truncation {
    pub applied: bool,
    pub limit_tokens: Option<usize>,
    pub tokens_removed: usize,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemovedSection {
    pub kind: RemovedSectionKind,
    pub label: String,
    pub tokens_removed: usize,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemovedSectionKind {
    HtmlExecutionBlock,
    HtmlBoilerplate,
    HtmlComment,
    DuplicateLine,
    CodeComment,
    Secret,
    Truncated,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoiseSource {
    pub kind: NoiseSourceKind,
    pub label: String,
    pub tokens_removed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoiseSourceKind {
    HtmlBoilerplate,
    Repetition,
    CodeComments,
    Secret,
    Truncation,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Warning {
    pub code: String,
    pub message: String,
    pub severity: WarningSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WarningSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub elapsed_ms: u128,
    pub engine: String,
}
