//! Error types for armybox applets

use std::fmt;
use std::io;

/// Error type for applet failures
#[derive(Debug)]
pub enum AppletError {
    /// Applet not found
    NotFound(String),
    /// Invalid arguments
    InvalidArgs(String),
    /// I/O error
    Io(io::Error),
    /// Generic error message
    Message(String),
}

impl fmt::Display for AppletError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppletError::NotFound(name) => write!(f, "applet not found: {}", name),
            AppletError::InvalidArgs(msg) => write!(f, "invalid arguments: {}", msg),
            AppletError::Io(e) => write!(f, "{}", e),
            AppletError::Message(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for AppletError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppletError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for AppletError {
    fn from(e: io::Error) -> Self {
        AppletError::Io(e)
    }
}

impl From<&str> for AppletError {
    fn from(s: &str) -> Self {
        AppletError::Message(s.to_string())
    }
}

impl From<String> for AppletError {
    fn from(s: String) -> Self {
        AppletError::Message(s)
    }
}

/// Result type for applets
pub type AppletResult<T> = Result<T, AppletError>;
