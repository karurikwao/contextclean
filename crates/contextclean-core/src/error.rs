use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContextCleanError {
    #[error("missing input; pass a file path or pipe content on stdin")]
    MissingInput,

    #[error("input file not found: {0}")]
    InputNotFound(String),

    #[error("failed to read input: {0}")]
    ReadInput(String),

    #[error("failed to write output: {0}")]
    WriteOutput(String),

    #[error("output file already exists; use --force to overwrite: {0}")]
    OutputExists(PathBuf),

    #[error("failed to serialize output as json: {0}")]
    Serialize(String),
}

impl ContextCleanError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::MissingInput | Self::InputNotFound(_) | Self::OutputExists(_) => 2,
            Self::ReadInput(_) | Self::WriteOutput(_) => 3,
            Self::Serialize(_) => 1,
        }
    }
}
