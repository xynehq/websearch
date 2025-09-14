//! DuckDuckGo search provider (uses HTML scraping)

use crate::{
    error::{SearchError, SearchResult},
    types::{ProviderConfig, SearchOptions, SearchProvider, SearchResult as SearchResultType},
    utils::{debug, http::HttpClient},
};
use scraper::{Html, Selector};
use std::collections::HashMap;

/// DuckDuckGo search types
#[derive(Debug, Clone)]
pub enum SearchType {
    Text,
    Images,
    News,
}

impl ToString for SearchType {
    fn to_string(&self) -> String {
        match self {
            SearchType::Text => "text".to_string(),
            SearchType::Images => "images".to_string(),
            SearchType::News => "news".to_string(),
        }
    }
}

/// DuckDuckGo configuration
#[derive(Debug, Clone)]
pub struct DuckDuckGoConfig {
    /// Base URL for DuckDuckGo
    pub base_url: String,
    /// Search type
    pub search_type: SearchType,
    /// Whether to use lite version
    pub use_lite: bool,
    /// User agent for requests
    pub user_agent: String,
}

impl Default for DuckDuckGoConfig {
    fn default() -> Self {
        Self {
            base_url: "https://html.duckduckgo.com/html".to_string(),
            search_type: SearchType::Text,
            use_lite: false,
            user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36".to_string(),
        }
    }
}

impl ProviderConfig for DuckDuckGoConfig {
    fn validate(&self) -> Result<(), SearchError> {
        // DuckDuckGo doesn't require API keys, so basic validation
        if self.base_url.is_empty() {
            return Err(SearchError::ConfigError("Base URL is required".to_string()));
        }
        Ok(())
    }

    fn base_url(&self) -> &str {
        &self.base_url
    }
}

/// DuckDuckGo search provider
#[derive(Debug)]
pub struct DuckDuckGoProvider {
    config: DuckDuckGoConfig,
    http_client: HttpClient,
}

impl DuckDuckGoProvider {
    /// Create a new DuckDuckGo provider with default configuration
    pub fn new() -> Self {
        Self::with_config(DuckDuckGoConfig::default())
    }

    /// Create a new DuckDuckGo provider with custom configuration
    pub fn with_config(config: DuckDuckGoConfig) -> Self {
        Self {
            config,
            http_client: HttpClient::new(),
        }
    }

    /// Create a DuckDuckGo provider for image search
    pub fn for_images() -> Self {
        let mut config = DuckDuckGoConfig::default();
        config.search_type = SearchType::Images;
        config.base_url = "https://duckduckgo.com/i.js".to_string();
        Self::with_config(config)
    }

    /// Create a DuckDuckGo provider for news search
    pub fn for_news() -> Self {
        let mut config = DuckDuckGoConfig::default();
        config.search_type = SearchType::News;
        config.base_url = "https://duckduckgo.com/news.js".to_string();
        Self::with_config(config)
    }

    /// Perform text search using HTML scraping
    async fn search_text(&self, options: &SearchOptions) -> SearchResult<Vec<SearchResultType>> {
        let mut headers = HashMap::new();
        headers.insert("User-Agent".to_string(), self.config.user_agent.clone());
        headers.insert(
            "Referer".to_string(),
            "https://html.duckduckgo.com/".to_string(),
        );

        let mut form_data = HashMap::new();
        form_data.insert("q".to_string(), options.query.clone());
        form_data.insert("b".to_string(), "".to_string());

        // Add region if provided
        if let Some(region) = &options.region {
            form_data.insert("kl".to_string(), region.clone());
        } else {
            form_data.insert("kl".to_string(), "wt-wt".to_string()); // Default to worldwide
        }

        debug::log_request(
            &options.debug,
            "DuckDuckGo Text Search request",
            &format!("query: {}", options.query),
        );

        let html = self
            .http_client
            .post_form_text_with_headers(&self.config.base_url, form_data, headers)
            .await?;

        debug::log_response(
            &options.debug,
            &format!("DuckDuckGo HTML response received (length: {})", html.len()),
        );

        // Parse HTML and extract search results
        self.parse_text_results(&html, options.max_results.unwrap_or(10))
    }

    /// Parse HTML search results from DuckDuckGo
    fn parse_text_results(
        &self,
        html: &str,
        max_results: u32,
    ) -> SearchResult<Vec<SearchResultType>> {
        let document = Html::parse_document(html);
        let mut results = Vec::new();

        // Selector for search result links
        let result_selector = Selector::parse("h2.result__title a")
            .map_err(|_| SearchError::ParseError("Invalid CSS selector for results".to_string()))?;

        // Selector for result snippets
        let snippet_selector = Selector::parse(".result__snippet").map_err(|_| {
            SearchError::ParseError("Invalid CSS selector for snippets".to_string())
        })?;

        let result_links: Vec<_> = document.select(&result_selector).collect();
        let result_snippets: Vec<_> = document.select(&snippet_selector).collect();

        for (i, link_element) in result_links.iter().enumerate() {
            if results.len() >= max_results as usize {
                break;
            }

            if let Some(href) = link_element.value().attr("href") {
                // Skip DuckDuckGo internal links
                if href.contains("duckduckgo.com") || href.contains("google.com/search") {
                    continue;
                }

                let url = crate::utils::http::normalize_url(href);
                let title = crate::utils::http::normalize_text(&link_element.inner_html());

                // Get corresponding snippet
                let snippet = result_snippets.get(i).map(|snippet_elem| {
                    crate::utils::http::normalize_text(&snippet_elem.inner_html())
                });

                let domain = crate::utils::http::extract_domain(&url);

                results.push(SearchResultType {
                    url,
                    title,
                    snippet,
                    domain,
                    published_date: None,
                    provider: Some("duckduckgo".to_string()),
                    raw: None,
                });
            }
        }

        Ok(results)
    }
}

impl Default for DuckDuckGoProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl SearchProvider for DuckDuckGoProvider {
    fn name(&self) -> &str {
        "duckduckgo"
    }

    async fn search(&self, options: &SearchOptions) -> SearchResult<Vec<SearchResultType>> {
        match self.config.search_type {
            SearchType::Text => self.search_text(options).await,
            SearchType::Images => {
                // For now, fall back to text search for images and news
                // Full implementation would require handling DuckDuckGo's vqd parameter
                Err(SearchError::ProviderError(
                    "Image search not yet implemented".to_string(),
                ))
            }
            SearchType::News => Err(SearchError::ProviderError(
                "News search not yet implemented".to_string(),
            )),
        }
    }

    fn config(&self) -> HashMap<String, String> {
        let mut config = HashMap::new();
        config.insert("base_url".to_string(), self.config.base_url.clone());
        config.insert(
            "search_type".to_string(),
            self.config.search_type.to_string(),
        );
        config.insert("use_lite".to_string(), self.config.use_lite.to_string());
        config
    }
}
