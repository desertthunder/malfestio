use thiserror::Error;

/// Errors that can occur during article extraction
#[derive(Error, Debug)]
pub enum Error {
    #[error("HTML parsing failed: {0}")]
    ParseError(String),

    #[error("XPath evaluation failed: {0}")]
    XPathError(String),

    #[error("Config parse error: {0}")]
    ConfigError(String),

    #[error("Extraction failed: {0}")]
    ExtractionError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for readability operations
pub type Result<T> = std::result::Result<T, Error>;
