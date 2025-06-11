//! Error types for the Hitman HTTP client.

use thiserror::Error;

/// Result type alias for Hitman operations.
pub type Result<T> = std::result::Result<T, HitError>;

/// Errors that can occur during parsing and execution of .hit files.
#[derive(Error, Debug)]
pub enum HitError {
    /// Error reading the input file.
    #[error("Failed to read file '{path}': {source}")]
    FileRead {
        path: String,
        source: std::io::Error,
    },

    /// Error parsing a block in the .hit file.
    #[error("Failed to parse block at line {line}: {reason}")]
    ParseError { line: usize, reason: String },

    /// Invalid HTTP method.
    #[error("Invalid HTTP method: '{method}'")]
    InvalidMethod { method: String },

    /// Invalid URL format.
    #[error("Invalid URL: '{url}'")]
    InvalidUrl { url: String },

    /// Malformed header syntax.
    #[error("Malformed header syntax in line {line}: '{content}'")]
    MalformedHeader { line: usize, content: String },

    /// Malformed JSON data.
    #[error("Malformed JSON data in line {line}: {source}")]
    MalformedJson {
        line: usize,
        source: serde_json::Error,
    },

    /// Empty block with no content.
    #[error("Empty block found")]
    EmptyBlock,

    /// Duplicate directive in block.
    #[error("Duplicate {directive} directive found in line {line}")]
    DuplicateDirective { directive: String, line: usize },

    /// Unknown directive in block.
    #[error("Unknown directive '{directive}' in line {line}")]
    UnknownDirective { directive: String, line: usize },

    /// HTTP request execution error.
    #[error("HTTP request failed: {source}")]
    RequestError { source: reqwest::Error },

    /// URL parsing error.
    #[error("Invalid URL format: {source}")]
    UrlParseError { source: url::ParseError },
}
