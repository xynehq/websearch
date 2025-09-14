//! SerpAPI provider

use crate::{
    error::{SearchError, SearchResult},
    types::{SearchOptions, SearchProvider, SearchResult as SearchResultType},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
struct SerpApiSearchResult {
    position: Option<u32>,
    title: String,
    link: String,
    displayed_link: Option<String>,
    snippet: Option<String>,
    snippet_highlighted_words: Option<Vec<String>>,
    date: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SerpApiSearchMetadata {
    id: String,
    status: String,
    created_at: String,
    processed_at: String,
    total_time_taken: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct SerpApiSearchInformation {
    total_results: Option<u64>,
    time_taken_displayed: Option<f32>,
    query_displayed: String,
}

#[derive(Debug, Deserialize)]
struct SerpApiResponse {
    search_metadata: Option<SerpApiSearchMetadata>,
    search_information: Option<SerpApiSearchInformation>,
    organic_results: Option<Vec<SerpApiSearchResult>>,
    error: Option<String>,
}

#[derive(Debug)]
pub struct SerpApiProvider {
    api_key: String,
    engine: String,
    base_url: String,
}

impl SerpApiProvider {
    pub fn new(api_key: &str) -> SearchResult<Self> {
        if api_key.is_empty() {
            return Err(SearchError::ConfigError(
                "SerpAPI key is required".to_string(),
            ));
        }

        Ok(Self {
            api_key: api_key.to_string(),
            engine: "google".to_string(),
            base_url: "https://serpapi.com/search.json".to_string(),
        })
    }

    pub fn with_engine(mut self, engine: &str) -> Self {
        self.engine = engine.to_string();
        self
    }

    pub fn with_base_url(mut self, base_url: &str) -> Self {
        self.base_url = base_url.to_string();
        self
    }
}

#[async_trait::async_trait]
impl SearchProvider for SerpApiProvider {
    fn name(&self) -> &str {
        "serpapi"
    }

    async fn search(&self, options: &SearchOptions) -> SearchResult<Vec<SearchResultType>> {
        let client = reqwest::Client::new();

        // Build query parameters with owned strings
        let mut params = HashMap::new();
        params.insert("engine".to_string(), self.engine.clone());
        params.insert("api_key".to_string(), self.api_key.clone());
        params.insert("q".to_string(), options.query.clone());
        params.insert(
            "num".to_string(),
            options.max_results.unwrap_or(10).to_string(),
        );

        // Add pagination if page > 1
        if let Some(page) = options.page {
            if page > 1 {
                let max_results = options.max_results.unwrap_or(10);
                let start = (page - 1) * max_results + 1;
                params.insert("start".to_string(), start.to_string());
            }
        }

        // Add language if specified
        if let Some(ref language) = options.language {
            params.insert("hl".to_string(), language.clone());
        }

        // Add region if specified
        if let Some(ref region) = options.region {
            params.insert("gl".to_string(), region.clone());
        }

        // Add safe search if specified
        if let Some(ref safe_search) = options.safe_search {
            params.insert("safe".to_string(), safe_search.to_string());
        }

        // Make the request
        let request = client.get(&self.base_url).query(&params);

        let response = request.send().await.map_err(|e| SearchError::HttpError {
            message: format!("Failed to send request: {e}"),
            status_code: None,
            response_body: None,
        })?;

        let status = response.status();
        let response_text = response.text().await.map_err(|e| SearchError::HttpError {
            message: format!("Failed to read response: {e}"),
            status_code: None,
            response_body: None,
        })?;

        if !status.is_success() {
            return Err(SearchError::ProviderError(format!(
                "SerpAPI request failed with status {status}: {response_text}"
            )));
        }

        // Parse JSON response
        let serp_response: SerpApiResponse = serde_json::from_str(&response_text).map_err(|e| {
            SearchError::ParseError(format!("Failed to parse SerpAPI response: {e}"))
        })?;

        // Check for API error
        if let Some(error) = serp_response.error {
            return Err(SearchError::ProviderError(format!(
                "SerpAPI error: {error}"
            )));
        }

        // Extract results
        let organic_results = serp_response.organic_results.unwrap_or_default();

        if organic_results.is_empty() {
            return Ok(vec![]);
        }

        // Transform to standard format
        let results = organic_results
            .into_iter()
            .map(|result| {
                // Extract domain from displayed_link or link
                let domain = result
                    .displayed_link
                    .as_ref()
                    .or(Some(&result.link))
                    .and_then(|link| {
                        if let Ok(url) = url::Url::parse(link) {
                            url.host_str().map(|s| s.to_string())
                        } else {
                            link.split('/').next().map(|s| s.to_string())
                        }
                    });

                let raw_value = serde_json::to_value(&result).unwrap_or_default();
                SearchResultType {
                    url: result.link,
                    title: result.title,
                    snippet: result.snippet,
                    domain,
                    published_date: result.date,
                    provider: Some("serpapi".to_string()),
                    raw: Some(raw_value),
                }
            })
            .collect();

        Ok(results)
    }

    fn config(&self) -> HashMap<String, String> {
        let mut config = HashMap::new();
        config.insert("api_key".to_string(), "***".to_string());
        config.insert("engine".to_string(), self.engine.clone());
        config.insert("base_url".to_string(), self.base_url.clone());
        config
    }
}
