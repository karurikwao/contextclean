pub mod cleaner;
pub mod config;
pub mod error;
pub mod html;
pub mod logs;
pub mod models;
pub mod renderer;
pub mod scanner;

pub use cleaner::clean_text;
pub use config::{CleanMode, CleanOptions, OutputFormat};
pub use error::ContextCleanError;
pub use models::{
    CleanResult, InputStats, Metadata, Metrics, NoiseSource, NoiseSourceKind, OutputBlock,
    RemovedSection, RemovedSectionKind, Truncation, Warning, WarningSeverity,
};
pub use renderer::render_result;
pub use scanner::{read_source, SourceData};
