pub mod budget;
pub mod cleaner;
pub mod config;
pub mod error;
pub mod html;
pub mod logs;
pub mod models;
pub mod renderer;
pub mod report;
pub mod scanner;

pub use budget::{count_tokens, FitModel, MIN_EXPLAINABLE_TRUNCATION_TOKENS};
pub use cleaner::clean_text;
pub use config::{CleanMode, CleanOptions, OutputFormat};
pub use error::ContextCleanError;
pub use models::{
    Budget, BudgetLimitSource, CleanResult, InputStats, Metadata, Metrics, NoiseSource,
    NoiseSourceKind, OutputBlock, RemovedSection, RemovedSectionKind, Truncation, Warning,
    WarningSeverity,
};
pub use renderer::render_result;
pub use report::{build_report, render_report, ContextReport, ReportOptions};
pub use scanner::{read_source, read_source_with_options, ReadOptions, SourceData};
