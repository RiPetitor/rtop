use std::io;
use std::path::PathBuf;

use thiserror::Error;

/// Main error type for rtop
#[derive(Debug, Error)]
pub enum RtopError {
    /// Terminal initialization or operation failed
    #[error("Terminal error: {0}")]
    Terminal(#[from] io::Error),

    /// Configuration parsing failed
    #[error("Configuration error: {message}")]
    Config { message: String },

    /// Configuration file not found
    #[error("Configuration file not found: {path}")]
    ConfigNotFound { path: PathBuf },

    /// Configuration file is invalid
    #[error("Invalid configuration file {path}: {reason}")]
    ConfigInvalid { path: PathBuf, reason: String },

    /// GPU probing failed
    #[error("GPU probe error: {gpu_id}: {reason}")]
    GpuProbe { gpu_id: String, reason: String },

    /// GPU not found
    #[error("GPU not found: {gpu_id}")]
    GpuNotFound { gpu_id: String },

    /// GPU operation timeout
    #[error("GPU operation timeout for {gpu_id} after {timeout_ms}ms")]
    GpuTimeout { gpu_id: String, timeout_ms: u64 },

    /// Process operation failed
    #[error("Process error: {message}")]
    Process { message: String },

    /// Process not found
    #[error("Process not found: PID {pid}")]
    ProcessNotFound { pid: u32 },

    /// Container operation failed
    #[error("Container error: {message}")]
    Container { message: String },

    /// Container not found
    #[error("Container not found: {key}")]
    ContainerNotFound { key: String },

    /// Network operation failed
    #[error("Network error: {message}")]
    Network { message: String },

    /// Invalid argument provided
    #[error("Invalid argument: {argument}")]
    InvalidArgument { argument: String },

    /// System operation failed
    #[error("System error: {message}")]
    System { message: String },
}

impl RtopError {
    /// Create a configuration error
    pub fn config(message: impl Into<String>) -> Self {
        RtopError::Config {
            message: message.into(),
        }
    }

    /// Create a process error
    pub fn process(message: impl Into<String>) -> Self {
        RtopError::Process {
            message: message.into(),
        }
    }

    /// Create a GPU probe error
    pub fn gpu_probe(gpu_id: impl Into<String>, reason: impl Into<String>) -> Self {
        RtopError::GpuProbe {
            gpu_id: gpu_id.into(),
            reason: reason.into(),
        }
    }

    /// Create a container error
    pub fn container(message: impl Into<String>) -> Self {
        RtopError::Container {
            message: message.into(),
        }
    }

    /// Create an invalid argument error
    pub fn invalid_argument(argument: impl Into<String>) -> Self {
        RtopError::InvalidArgument {
            argument: argument.into(),
        }
    }

    /// Create a system error
    pub fn system(message: impl Into<String>) -> Self {
        RtopError::System {
            message: message.into(),
        }
    }

    /// Create a process not found error
    pub fn process_not_found(pid: u32) -> Self {
        RtopError::ProcessNotFound { pid }
    }

    /// Create a GPU not found error
    pub fn gpu_not_found(gpu_id: impl Into<String>) -> Self {
        RtopError::GpuNotFound {
            gpu_id: gpu_id.into(),
        }
    }

    /// Create a GPU timeout error
    pub fn gpu_timeout(gpu_id: impl Into<String>, timeout_ms: u64) -> Self {
        RtopError::GpuTimeout {
            gpu_id: gpu_id.into(),
            timeout_ms,
        }
    }

    /// Create a container not found error
    pub fn container_not_found(key: impl Into<String>) -> Self {
        RtopError::ContainerNotFound { key: key.into() }
    }

    /// Create a config not found error
    pub fn config_not_found(path: PathBuf) -> Self {
        RtopError::ConfigNotFound { path }
    }

    /// Create a config invalid error
    pub fn config_invalid(path: PathBuf, reason: impl Into<String>) -> Self {
        RtopError::ConfigInvalid {
            path,
            reason: reason.into(),
        }
    }
}

/// Result type alias for rtop operations
pub type Result<T> = std::result::Result<T, RtopError>;

impl From<toml::de::Error> for RtopError {
    fn from(err: toml::de::Error) -> Self {
        RtopError::Config {
            message: err.to_string(),
        }
    }
}

impl From<toml::ser::Error> for RtopError {
    fn from(err: toml::ser::Error) -> Self {
        RtopError::Config {
            message: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for RtopError {
    fn from(err: serde_json::Error) -> Self {
        RtopError::Config {
            message: err.to_string(),
        }
    }
}

impl From<serde_yaml_ng::Error> for RtopError {
    fn from(err: serde_yaml_ng::Error) -> Self {
        RtopError::Config {
            message: err.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_messages() {
        let err = RtopError::process("Test error");
        assert!(err.to_string().contains("Test error"));
        assert!(matches!(err, RtopError::Process { .. }));
    }

    #[test]
    fn test_error_creation_helpers() {
        let err = RtopError::gpu_probe("gpu:0", "Not found");
        assert!(
            matches!(err, RtopError::GpuProbe { gpu_id, reason } if gpu_id == "gpu:0" && reason == "Not found")
        );

        let err = RtopError::process_not_found(1234);
        assert!(matches!(err, RtopError::ProcessNotFound { pid } if pid == 1234));

        let err = RtopError::invalid_argument("test_arg");
        assert!(matches!(err, RtopError::InvalidArgument { argument } if argument == "test_arg"));
    }
}
