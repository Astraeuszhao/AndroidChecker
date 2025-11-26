use thiserror::Error;`n#[derive(Error, Debug)]
pub enum AdbError {
    #[error("ADB not found in PATH or current directory")]
    AdbNotFound,`n    #[error("ADB command failed: {0}")]
    CommandFailed(String),`n    #[error("ADB timeout after {0}s")]
    Timeout(u64),`n    #[error("Device not found: {0}")]
    DeviceNotFound(String),`n    #[error("No devices connected")]
    NoDevices,`n    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),`n    #[error("UTF-8 decode error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),`n    #[error("Parse error: {0}")]
    Parse(String),`n    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),`n    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}`npub type Result<T> = std::result::Result<T, AdbError>;
