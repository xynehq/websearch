//! Core types and traits for the search SDK

use crate::error::SearchError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Represents a web search result returned by any search provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// URL of the search result
    pub url: String,
    /// Title of the web page
    pub title: String,
    /// Snippet/description of the web page
    pub snippet: Option<String>,
    /// The source website domain
    pub domain: Option<String>,
    /// When the result was published or last updated
    pub published_date: Option<String>,
    /// The search provider that returned this result
    pub provider: Option<String>,
    /// Raw response data from the provider
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<serde_json::Value>,
}

/// Debug options for the search SDK
#[derive(Debug, Clone, Default)]
pub struct DebugOptions {
    /// Enable verbose logging
    pub enabled: bool,
    /// Log request details (URLs, headers, etc.)
    pub log_requests: bool,
    /// Log full responses
    pub log_responses: bool,
}

/// Safe search setting levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SafeSearch {
    Off,
    Moderate,
    Strict,
}

impl fmt::Display for SafeSearch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SafeSearch::Off => write!(f, "off"),
            SafeSearch::Moderate => write!(f, "moderate"),
            SafeSearch::Strict => write!(f, "strict"),
        }
    }
}

/// Sort options for search results (primarily for Arxiv)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortBy {
    Relevance,
    LastUpdatedDate,
    SubmittedDate,
}

impl fmt::Display for SortBy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SortBy::Relevance => write!(f, "relevance"),
            SortBy::LastUpdatedDate => write!(f, "lastUpdatedDate"),
            SortBy::SubmittedDate => write!(f, "submittedDate"),
        }
    }
}

/// Sort order for search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl fmt::Display for SortOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SortOrder::Ascending => write!(f, "ascending"),
            SortOrder::Descending => write!(f, "descending"),
        }
    }
}

/// Common options for web search across all providers
#[derive(Debug)]
pub struct SearchOptions {
    /// The search query text
    pub query: String,
    /// (Arxiv specific) A comma-delimited list of Arxiv IDs to fetch
    pub id_list: Option<String>,
    /// Maximum number of results to return
    pub max_results: Option<u32>,
    /// Language/locale for results
    pub language: Option<String>,
    /// Country/region for results
    pub region: Option<String>,
    /// Safe search setting
    pub safe_search: Option<SafeSearch>,
    /// Result page number (for pagination)
    pub page: Option<u32>,
    /// (Arxiv specific) The starting index for results (pagination offset)
    pub start: Option<u32>,
    /// (Arxiv specific) Sort order for results
    pub sort_by: Option<SortBy>,
    /// (Arxiv specific) Sort direction
    pub sort_order: Option<SortOrder>,
    /// Custom timeout in milliseconds
    pub timeout: Option<u64>,
    /// Debug options
    pub debug: Option<DebugOptions>,
    /// The search provider to use
    pub provider: Box<dyn SearchProvider>,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            query: String::new(),
            id_list: None,
            max_results: Some(10),
            language: None,
            region: None,
            safe_search: None,
            page: Some(1),
            start: None,
            sort_by: None,
            sort_order: None,
            timeout: Some(15000), // 15 seconds
            debug: None,
            provider: Box::new(DummyProvider), // Will be replaced
        }
    }
}

/// Trait that all search provider implementations must satisfy
#[async_trait::async_trait]
pub trait SearchProvider: Send + Sync + std::fmt::Debug {
    /// Name of the search provider
    fn name(&self) -> &str;

    /// Search method implementation
    async fn search(&self, options: &SearchOptions) -> Result<Vec<SearchResult>, SearchError>;

    /// Get provider configuration (for debugging/logging)
    fn config(&self) -> HashMap<String, String> {
        HashMap::new()
    }
}

/// Dummy provider for default implementation (should not be used)
#[derive(Debug)]
struct DummyProvider;

#[async_trait::async_trait]
impl SearchProvider for DummyProvider {
    fn name(&self) -> &str {
        "dummy"
    }

    async fn search(&self, _options: &SearchOptions) -> Result<Vec<SearchResult>, SearchError> {
        Err(SearchError::InvalidInput(
            "No provider configured".to_string(),
        ))
    }
}

/// Provider configuration trait for consistent configuration patterns
pub trait ProviderConfig {
    /// Validate the configuration
    fn validate(&self) -> Result<(), SearchError>;

    /// Get the base URL for API requests
    fn base_url(&self) -> &str;

    /// Get API key if required
    fn api_key(&self) -> Option<&str> {
        None
    }
}
