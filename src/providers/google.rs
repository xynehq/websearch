//! Google Custom Search API provider

use crate::{
    error::{SearchError, SearchResult},
    types::{ProviderConfig, SearchOptions, SearchProvider, SearchResult as SearchResultType},
    utils::{debug, http::HttpClient},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Google Custom Search API response types
#[derive(Debug, Deserialize, Serialize)]
struct GoogleSearchItem {
    title: String,
    link: String,
    #[serde(rename = "displayLink")]
    display_link: String,
    snippet: String,
    #[serde(default)]
    pagemap: Option<GooglePageMap>,
}

#[derive(Debug, Deserialize, Serialize)]
struct GooglePageMap {
    #[serde(default)]
    metatags: Option<Vec<HashMap<String, String>>>,
}

#[derive(Debug, Deserialize)]
struct GoogleSearchResponse {
    #[serde(default)]
    items: Option<Vec<GoogleSearchItem>>,
    #[serde(rename = "searchInformation")]
    search_information: Option<GoogleSearchInfo>,
}

#[derive(Debug, Deserialize)]
struct GoogleSearchInfo {
    #[serde(rename = "totalResults")]
    total_results: String,
    #[serde(rename = "searchTime")]
    search_time: f64,
}

/// Google Custom Search configuration
#[derive(Debug, Clone)]
pub struct GoogleConfig {
    /// Google API key
    pub api_key: String,
    /// Custom Search Engine ID
    pub cx: String,
    /// Base URL for the API
    pub base_url: String,
}

impl Default for GoogleConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            cx: String::new(),
            base_url: "https://www.googleapis.com/customsearch/v1".to_string(),
        }
    }
}

impl ProviderConfig for GoogleConfig {
    fn validate(&self) -> Result<(), SearchError> {
        if self.api_key.is_empty() {
            return Err(SearchError::ConfigError(
                "Google API key is required".to_string(),
            ));
        }
        if self.cx.is_empty() {
            return Err(SearchError::ConfigError(
                "Google Search Engine ID (cx) is required".to_string(),
            ));
        }
        Ok(())
    }

    fn base_url(&self) -> &str {
        &self.base_url
    }

    fn api_key(&self) -> Option<&str> {
        Some(&self.api_key)
    }
}

/// Google Custom Search provider
#[derive(Debug)]
pub struct GoogleProvider {
    config: GoogleConfig,
    http_client: HttpClient,
}

impl GoogleProvider {
    /// Create a new Google provider with API key and Search Engine ID
    pub fn new(api_key: &str, cx: &str) -> SearchResult<Self> {
        let config = GoogleConfig {
            api_key: api_key.to_string(),
            cx: cx.to_string(),
            ..Default::default()
        };

        config.validate()?;

        Ok(Self {
            config,
            http_client: HttpClient::new(),
        })
    }

    /// Create a new Google provider with custom configuration
    pub fn with_config(config: GoogleConfig) -> SearchResult<Self> {
        config.validate()?;

        Ok(Self {
            config,
            http_client: HttpClient::new(),
        })
    }

    /// Build the search URL with parameters
    fn build_search_url(&self, options: &SearchOptions) -> SearchResult<String> {
        let mut params = HashMap::new();

        params.insert("key".to_string(), self.config.api_key.clone());
        params.insert("cx".to_string(), self.config.cx.clone());
        params.insert("q".to_string(), options.query.clone());

        // Add max results (Google limits to 10 per request)
        if let Some(max_results) = options.max_results {
            let num = if max_results > 10 { 10 } else { max_results };
            params.insert("num".to_string(), num.to_string());
        }

        // Add pagination
        if let Some(page) = options.page {
            let max_results = options.max_results.unwrap_or(10);
            let start = (page - 1) * max_results + 1;
            params.insert("start".to_string(), start.to_string());
        }

        // Add language
        if let Some(language) = &options.language {
            params.insert("lr".to_string(), format!("lang_{language}"));
        }

        // Add region
        if let Some(region) = &options.region {
            params.insert("gl".to_string(), region.clone());
        }

        // Add safe search
        if let Some(safe_search) = &options.safe_search {
            let safe = match safe_search.to_string().as_str() {
                "off" => "off",
                _ => "active",
            };
            params.insert("safe".to_string(), safe.to_string());
        }

        crate::utils::http::build_url(&self.config.base_url, params)
    }
}

#[async_trait::async_trait]
impl SearchProvider for GoogleProvider {
    fn name(&self) -> &str {
        "google"
    }

    async fn search(&self, options: &SearchOptions) -> SearchResult<Vec<SearchResultType>> {
        // Log request if debugging is enabled
        debug::log_request(
            &options.debug,
            "Google Search request",
            &format!("query: {}", options.query),
        );

        let url = self.build_search_url(options)?;

        // Make the request
        let response: GoogleSearchResponse = self.http_client.get_json(&url).await?;

        // Log response if debugging is enabled
        debug::log_response(
            &options.debug,
            &format!(
                "Google Search returned {} results",
                response
                    .items
                    .as_ref()
                    .map(|items| items.len())
                    .unwrap_or(0)
            ),
        );

        // Convert Google results to standard format
        let results = if let Some(items) = response.items {
            items
                .into_iter()
                .map(|item| {
                    // Extract published date from metadata if available
                    let published_date = item
                        .pagemap
                        .as_ref()
                        .and_then(|pm| pm.metatags.as_ref())
                        .and_then(|tags| tags.first())
                        .and_then(|meta| {
                            meta.get("article:published_time")
                                .or_else(|| meta.get("date"))
                                .or_else(|| meta.get("og:updated_time"))
                        })
                        .cloned();

                    SearchResultType {
                        url: item.link.clone(),
                        title: item.title.clone(),
                        snippet: Some(item.snippet.clone()),
                        domain: Some(item.display_link.clone()),
                        published_date,
                        provider: Some("google".to_string()),
                        raw: serde_json::to_value(&item).ok(),
                    }
                })
                .collect()
        } else {
            Vec::new()
        };

        Ok(results)
    }

    fn config(&self) -> HashMap<String, String> {
        let mut config = HashMap::new();
        config.insert("api_key".to_string(), "***".to_string()); // Hide API key
        config.insert("cx".to_string(), self.config.cx.clone());
        config.insert("base_url".to_string(), self.config.base_url.clone());
        config
    }
}
