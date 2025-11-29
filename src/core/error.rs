use thiserror::Error;
#[derive(Error, Debug)]
pub enum AdbError {
    #[error("ADB not found in PATH or current directory")]
    AdbNotFound,
    #[error("ADB command failed: {0}")]
    CommandFailed(String),
    #[error("ADB timeout after {0}s")]
    Timeout(u64),
    #[error("Device not found: {0}")]
    DeviceNotFound(String),
    #[error("No devices connected")]
    NoDevices,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("UTF-8 decode error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}
pub type Result<T> = std::result::Result<T, AdbError>;