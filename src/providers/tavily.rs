//! Tavily Search API provider
//!
//! Tavily is an AI-powered search API optimized for LLM agents and applications.
//! It provides comprehensive, real-time search results with high relevance.

use crate::{
    error::{SearchError, SearchResult},
    types::{SearchOptions, SearchProvider, SearchResult as SearchResultType},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tavily search result structure
#[derive(Debug, Deserialize, Serialize)]
struct TavilySearchResult {
    title: String,
    url: String,
    content: String,
    score: Option<f64>,
    published_date: Option<String>,
}

/// Tavily API response structure
#[derive(Debug, Deserialize)]
struct TavilyResponse {
    answer: Option<String>,
    query: String,
    response_time: Option<f64>,
    images: Option<Vec<String>>,
    results: Vec<TavilySearchResult>,
    follow_up_questions: Option<Vec<String>>,
}

/// Tavily search request structure
#[derive(Debug, Serialize)]
struct TavilyRequest {
    api_key: String,
    query: String,
    search_depth: String,
    include_answer: bool,
    include_images: bool,
    include_raw_content: bool,
    max_results: u32,
    include_domains: Option<Vec<String>>,
    exclude_domains: Option<Vec<String>>,
}

/// Tavily Search API provider
#[derive(Debug, Clone)]
pub struct TavilyProvider {
    api_key: String,
    base_url: String,
    search_depth: String,
    include_answer: bool,
    include_images: bool,
    include_raw_content: bool,
}

impl TavilyProvider {
    /// Create a new Tavily provider with the given API key
    pub fn new(api_key: &str) -> SearchResult<Self> {
        if api_key.is_empty() {
            return Err(SearchError::ConfigError(
                "Tavily API key is required".to_string(),
            ));
        }

        // Validate API key format (should start with "tvly-")
        if !api_key.starts_with("tvly-") {
            return Err(SearchError::ConfigError(
                "Invalid Tavily API key format. Keys should start with 'tvly-'".to_string(),
            ));
        }

        Ok(Self {
            api_key: api_key.to_string(),
            base_url: "https://api.tavily.com/search".to_string(),
            search_depth: "basic".to_string(), // "basic" or "advanced"
            include_answer: true,
            include_images: false,
            include_raw_content: false,
        })
    }

    /// Create a new Tavily provider with advanced search depth
    pub fn new_advanced(api_key: &str) -> SearchResult<Self> {
        let mut provider = Self::new(api_key)?;
        provider.search_depth = "advanced".to_string();
        provider.include_raw_content = true;
        Ok(provider)
    }

    /// Enable or disable answer generation
    pub fn with_answer(mut self, include_answer: bool) -> Self {
        self.include_answer = include_answer;
        self
    }

    /// Enable or disable image results
    pub fn with_images(mut self, include_images: bool) -> Self {
        self.include_images = include_images;
        self
    }

    /// Set search depth ("basic" or "advanced")
    pub fn with_search_depth(mut self, depth: &str) -> SearchResult<Self> {
        if depth != "basic" && depth != "advanced" {
            return Err(SearchError::ConfigError(
                "Search depth must be 'basic' or 'advanced'".to_string(),
            ));
        }
        self.search_depth = depth.to_string();
        Ok(self)
    }

    /// Set custom base URL (for testing or enterprise endpoints)
    pub fn with_base_url(mut self, base_url: &str) -> Self {
        self.base_url = base_url.to_string();
        self
    }
}

#[async_trait::async_trait]
impl SearchProvider for TavilyProvider {
    fn name(&self) -> &str {
        "tavily"
    }

    async fn search(&self, options: &SearchOptions) -> SearchResult<Vec<SearchResultType>> {
        if options.query.is_empty() {
            return Err(SearchError::InvalidInput(
                "Query cannot be empty".to_string(),
            ));
        }

        let timeout_duration = std::time::Duration::from_millis(options.timeout.unwrap_or(15000));
        let client = reqwest::Client::builder()
            .timeout(timeout_duration)
            .build()
            .map_err(|e| {
                SearchError::ConfigError(format!("Failed to create HTTP client: {e}"))
            })?;

        let max_results = options.max_results.unwrap_or(10).min(50); // Tavily max is 50

        let request_body = TavilyRequest {
            api_key: self.api_key.clone(),
            query: options.query.clone(),
            search_depth: self.search_depth.clone(),
            include_answer: self.include_answer,
            include_images: self.include_images,
            include_raw_content: self.include_raw_content,
            max_results,
            include_domains: None, // Could be added as future enhancement
            exclude_domains: None, // Could be added as future enhancement
        };

        let response = client
            .post(&self.base_url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| SearchError::HttpError {
                message: format!("Failed to send request to Tavily: {e}"),
                status_code: None,
                response_body: None,
            })?;

        let status = response.status();
        let response_text = response.text().await.map_err(|e| SearchError::HttpError {
            message: format!("Failed to read Tavily response: {e}"),
            status_code: Some(status.as_u16()),
            response_body: None,
        })?;

        if !status.is_success() {
            let error_msg = match status.as_u16() {
                400 => "Bad request - check your query parameters",
                401 => "Unauthorized - check your API key",
                402 => "Payment required - check your Tavily account billing",
                403 => "Forbidden - API key may be invalid or suspended",
                429 => "Rate limit exceeded - too many requests",
                500..=599 => "Tavily server error - try again later",
                _ => "Unknown error occurred",
            };

            return Err(SearchError::HttpError {
                message: format!("Tavily API error ({status}): {error_msg}"),
                status_code: Some(status.as_u16()),
                response_body: Some(response_text),
            });
        }

        let tavily_response: TavilyResponse =
            serde_json::from_str(&response_text).map_err(|e| {
                SearchError::ParseError(format!(
                    "Failed to parse Tavily response: {e}. Response: {response_text}"
                ))
            })?;

        // Convert Tavily results to our standard format
        let results: Vec<SearchResultType> = tavily_response
            .results
            .into_iter()
            .map(|result| {
                // Store the original result as raw data
                let raw_value = serde_json::to_value(&result).unwrap_or_default();

                SearchResultType {
                    url: result.url,
                    title: result.title,
                    snippet: Some(result.content),
                    domain: extract_domain(raw_value["url"].as_str().unwrap_or("")),
                    published_date: result.published_date,
                    provider: Some("tavily".to_string()),
                    raw: Some(raw_value),
                }
            })
            .collect();

        Ok(results)
    }

    fn config(&self) -> HashMap<String, String> {
        let mut config = HashMap::new();
        config.insert("provider".to_string(), "tavily".to_string());
        config.insert("api_key".to_string(), "***".to_string());
        config.insert("base_url".to_string(), self.base_url.clone());
        config.insert("search_depth".to_string(), self.search_depth.clone());
        config.insert(
            "include_answer".to_string(),
            self.include_answer.to_string(),
        );
        config.insert(
            "include_images".to_string(),
            self.include_images.to_string(),
        );
        config.insert(
            "include_raw_content".to_string(),
            self.include_raw_content.to_string(),
        );
        config
    }
}

/// Extract domain from URL
fn extract_domain(url: &str) -> Option<String> {
    if let Ok(parsed_url) = url::Url::parse(url) {
        parsed_url.host_str().map(|host| host.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tavily_provider_new() {
        // Valid API key
        let provider = TavilyProvider::new("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");
        assert!(provider.is_ok());

        // Empty API key
        let provider = TavilyProvider::new("");
        assert!(provider.is_err());
        match provider.unwrap_err() {
            SearchError::ConfigError(msg) => assert!(msg.contains("required")),
            _ => panic!("Expected ConfigError"),
        }

        // Invalid API key format
        let provider = TavilyProvider::new("invalid-key");
        assert!(provider.is_err());
        match provider.unwrap_err() {
            SearchError::ConfigError(msg) => assert!(msg.contains("tvly-")),
            _ => panic!("Expected ConfigError"),
        }
    }

    #[test]
    fn test_tavily_provider_configuration() {
        let provider = TavilyProvider::new("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")
            .unwrap()
            .with_answer(false)
            .with_images(true);

        assert!(!provider.include_answer);
        assert!(provider.include_images);
        assert_eq!(provider.search_depth, "basic");
    }

    #[test]
    fn test_tavily_provider_advanced() {
        let provider = TavilyProvider::new_advanced("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");
        assert!(provider.is_ok());
        let provider = provider.unwrap();
        assert_eq!(provider.search_depth, "advanced");
        assert!(provider.include_raw_content);
    }

    #[test]
    fn test_tavily_search_depth_validation() {
        let provider = TavilyProvider::new("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").unwrap();

        // Valid search depths - create new providers for each test
        let provider1 = TavilyProvider::new("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").unwrap();
        let provider2 = TavilyProvider::new("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").unwrap();
        let provider3 = TavilyProvider::new("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").unwrap();

        assert!(provider1.with_search_depth("basic").is_ok());
        assert!(provider2.with_search_depth("advanced").is_ok());

        // Invalid search depth
        assert!(provider3.with_search_depth("invalid").is_err());
    }

    #[test]
    fn test_tavily_provider_name() {
        let provider = TavilyProvider::new("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").unwrap();
        assert_eq!(provider.name(), "tavily");
    }

    #[test]
    fn test_tavily_provider_config() {
        let provider = TavilyProvider::new("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").unwrap();
        let config = provider.config();

        assert_eq!(config.get("provider"), Some(&"tavily".to_string()));
        assert_eq!(config.get("api_key"), Some(&"***".to_string()));
        assert!(config.contains_key("base_url"));
        assert!(config.contains_key("search_depth"));
    }

    #[tokio::test]
    async fn test_tavily_search_empty_query() {
        let provider = TavilyProvider::new("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").unwrap();
        let options = SearchOptions {
            query: "".to_string(),
            provider: Box::new(provider),
            ..Default::default()
        };

        let result = options.provider.search(&options).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            SearchError::InvalidInput(msg) => assert!(msg.contains("empty")),
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[test]
    fn test_extract_domain() {
        assert_eq!(
            extract_domain("https://example.com/path"),
            Some("example.com".to_string())
        );
        assert_eq!(
            extract_domain("http://subdomain.example.org"),
            Some("subdomain.example.org".to_string())
        );
        assert_eq!(extract_domain("invalid-url"), None);
        assert_eq!(extract_domain(""), None);
    }
}
