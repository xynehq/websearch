//! Error types for the search SDK

use thiserror::Error;

/// Result type alias for search operations
pub type SearchResult<T> = std::result::Result<T, SearchError>;

/// Comprehensive error types for search operations
#[derive(Error, Debug, Clone)]
pub enum SearchError {
    /// HTTP request failed
    #[error("HTTP request failed: {message}")]
    HttpError {
        message: String,
        status_code: Option<u16>,
        response_body: Option<String>,
    },

    /// Invalid input parameters
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Provider-specific error
    #[error("Provider error: {0}")]
    ProviderError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Parsing error (JSON, XML, HTML)
    #[error("Parsing error: {0}")]
    ParseError(String),

    /// Timeout error
    #[error("Request timed out after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    /// Authentication failed
    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    /// Generic error for unhandled cases
    #[error("Search error: {0}")]
    Other(String),
}

impl From<reqwest::Error> for SearchError {
    fn from(error: reqwest::Error) -> Self {
        if error.is_timeout() {
            SearchError::Timeout {
                timeout_ms: 15000, // Default timeout
            }
        } else if error.is_status() {
            let status_code = error.status().map(|s| s.as_u16());
            let message = error.to_string();

            // Determine if it's an auth error based on status code
            if let Some(401 | 403) = status_code {
                SearchError::AuthenticationError(message)
            } else if let Some(429) = status_code {
                SearchError::RateLimit(message)
            } else {
                SearchError::HttpError {
                    message,
                    status_code,
                    response_body: None,
                }
            }
        } else {
            SearchError::HttpError {
                message: error.to_string(),
                status_code: None,
                response_body: None,
            }
        }
    }
}

impl From<serde_json::Error> for SearchError {
    fn from(error: serde_json::Error) -> Self {
        SearchError::ParseError(format!("JSON parsing failed: {error}"))
    }
}

impl From<url::ParseError> for SearchError {
    fn from(error: url::ParseError) -> Self {
        SearchError::InvalidInput(format!("Invalid URL: {error}"))
    }
}

impl From<std::io::Error> for SearchError {
    fn from(error: std::io::Error) -> Self {
        SearchError::Other(format!("IO error: {error}"))
    }
}
