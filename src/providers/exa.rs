//! Exa Search API provider

use crate::{
    error::{SearchError, SearchResult},
    types::{SearchOptions, SearchProvider, SearchResult as SearchResultType},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

const DEFAULT_BASE_URL: &str = "https://api.exa.ai/search";

#[derive(Debug, Deserialize)]
struct ExaSearchResult {
    id: String,
    title: String,
    url: String,
    text: Option<String>, // Only present when include_contents is true
    #[serde(rename = "publishedDate")]
    published_date: Option<String>,
    author: Option<String>,
    score: Option<f64>, // Relevance score
}

#[derive(Debug, Deserialize)]
struct ExaSearchResponse {
    #[serde(rename = "requestId")]
    request_id: String,
    #[serde(rename = "autopromptString")]
    autoPrompt_string: String,
    results: Vec<ExaSearchResult>,
    #[serde(rename = "searchTime")]
    search_time: Option<f64>,
}

#[derive(Debug, Serialize)]
struct ExaSearchRequest {
    query: String,
    #[serde(rename = "max_results")]
    max_results: Option<usize>,
    model: String,
    #[serde(rename = "include_contents")]
    include_contents: bool,
}

#[derive(Debug)]
pub struct ExaProvider {
    api_key: String,
    base_url: String,
    model: String,
    include_contents: bool,
}

impl ExaProvider {
    pub fn new(api_key: &str) -> SearchResult<Self> {
        if api_key.is_empty() {
            return Err(SearchError::ConfigError(
                "Exa API key is required".to_string(),
            ));
        }

        Ok(Self {
            api_key: api_key.to_string(),
            base_url: DEFAULT_BASE_URL.to_string(),
            model: "keyword".to_string(),
            include_contents: false,
        })
    }

    pub fn new_advanced(api_key: &str) -> SearchResult<Self> {
        let mut provider = Self::new(api_key)?;
        provider.include_contents = true;
        Ok(provider)
    }

    pub fn with_model(mut self, model: &str) -> SearchResult<Self> {
        if model != "keyword" && model != "embeddings" {
            return Err(SearchError::ConfigError(
                "Model must be 'keyword' or 'embeddings'".to_string(),
            ));
        }
        self.model = model.to_string();
        Ok(self)
    }

    pub fn with_contents(mut self, include_contents: bool) -> Self {
        self.include_contents = include_contents;
        self
    }

    pub fn with_base_url(mut self, base_url: &str) -> Self {
        self.base_url = base_url.to_string();
        self
    }
}

#[async_trait::async_trait]
impl SearchProvider for ExaProvider {
    fn name(&self) -> &str {
        "exa"
    }

    async fn search(&self, options: &SearchOptions) -> SearchResult<Vec<SearchResultType>> {
        if options.query.trim().is_empty() {
            return Err(SearchError::InvalidInput(
                "Query cannot be empty".to_string(),
            ));
        }

        let client = reqwest::Client::new();

        let request_body = ExaSearchRequest {
            query: options.query.clone(),
            max_results: options.max_results.map(|n| n as usize),
            model: self.model.clone(),
            include_contents: self.include_contents,
        };

        if let Some(debug) = &options.debug {
            if debug.enabled && debug.log_requests {
                log::info!(
                    "Exa API request: {} with query: {}",
                    self.base_url,
                    options.query
                );
            }
        }

        let response = client
            .post(&self.base_url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request_body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    SearchError::Timeout { timeout_ms: 15000 }
                } else {
                    SearchError::ProviderError(format!("Exa API request failed: {e}"))
                }
            })?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            let error_msg = match status.as_u16() {
                401 => "Invalid API key. Check your Exa API token in the Authorization header.",
                403 => "Access denied. Your Exa API token may have insufficient permissions or has expired.",
                429 => "Rate limit exceeded. You have reached your Exa API quota or sent too many requests.",
                400 => "Bad request. Check your search parameters, especially the query and model values.",
                500..=599 => "Exa server error. The service might be experiencing issues. Try again later.",
                _ => "Exa API request failed",
            };

            return Err(SearchError::ProviderError(format!(
                "{error_msg}: {error_text} (Status: {status})"
            )));
        }

        let exa_response: ExaSearchResponse = response.json().await.map_err(|e| {
            SearchError::ProviderError(format!("Failed to parse Exa response: {e}"))
        })?;

        if let Some(debug) = &options.debug {
            if debug.enabled && debug.log_responses {
                log::info!(
                    "Exa API response: {} results for query: {}",
                    exa_response.results.len(),
                    exa_response.autoPrompt_string
                );
            }
        }

        let results = exa_response
            .results
            .into_iter()
            .map(|result| {
                let domain = Url::parse(&result.url)
                    .ok()
                    .and_then(|url| url.host_str().map(|s| s.to_string()));

                let mut raw_data = HashMap::new();
                raw_data.insert(
                    "id".to_string(),
                    serde_json::Value::String(result.id.clone()),
                );

                if let Some(score) = result.score {
                    raw_data.insert(
                        "score".to_string(),
                        serde_json::Value::Number(
                            serde_json::Number::from_f64(score)
                                .unwrap_or_else(|| serde_json::Number::from(0)),
                        ),
                    );
                }
                if let Some(author) = &result.author {
                    raw_data.insert(
                        "author".to_string(),
                        serde_json::Value::String(author.clone()),
                    );
                }

                SearchResultType {
                    url: result.url,
                    title: result.title,
                    snippet: result.text, // This might be None if content isn't included
                    domain,
                    published_date: result.published_date,
                    provider: Some("exa".to_string()),
                    raw: if raw_data.is_empty() {
                        None
                    } else {
                        Some(serde_json::to_value(raw_data).unwrap_or_default())
                    },
                }
            })
            .collect();

        Ok(results)
    }

    fn config(&self) -> HashMap<String, String> {
        let mut config = HashMap::new();
        config.insert("api_key".to_string(), "***".to_string());
        config.insert("base_url".to_string(), self.base_url.clone());
        config.insert("model".to_string(), self.model.clone());
        config.insert(
            "include_contents".to_string(),
            self.include_contents.to_string(),
        );
        config
    }
}
