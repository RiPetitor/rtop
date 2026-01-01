use std::io;

use thiserror::Error;

/// Main error type for rtop
#[derive(Debug, Error)]
pub enum RtopError {
    /// Terminal initialization or operation failed
    #[error("Terminal error: {0}")]
    Terminal(#[from] io::Error),

    /// Configuration parsing failed
    #[error("Configuration error: {0}")]
    Config(String),

    /// GPU probing failed
    #[error("GPU probe error: {0}")]
    GpuProbe(String),

    /// Process operation failed
    #[error("Process error: {0}")]
    Process(String),
}

/// Result type alias for rtop operations
pub type Result<T> = std::result::Result<T, RtopError>;

impl From<toml::de::Error> for RtopError {
    fn from(err: toml::de::Error) -> Self {
        RtopError::Config(err.to_string())
    }
}
