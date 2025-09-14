//! ArXiv API provider for searching academic papers

use crate::{
    error::{SearchError, SearchResult},
    types::{SearchOptions, SearchProvider, SearchResult as SearchResultType},
};
use serde::Deserialize;
use std::collections::HashMap;
use url::Url;

#[derive(Debug, Deserialize)]
struct ArxivEntry {
    id: String,
    title: String,
    summary: String,
    published: String,
    #[serde(default)]
    authors: Vec<ArxivAuthor>,
    #[serde(default)]
    links: Vec<ArxivLink>,
}

#[derive(Debug, Deserialize)]
struct ArxivAuthor {
    name: String,
}

#[derive(Debug, Deserialize)]
struct ArxivLink {
    #[serde(rename = "href")]
    href: String,
    #[serde(rename = "type")]
    link_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ArxivFeed {
    #[serde(rename = "entry", default)]
    entries: Vec<ArxivEntry>,
}

#[derive(Debug)]
pub struct ArxivProvider {
    base_url: String,
}

impl ArxivProvider {
    pub fn new() -> Self {
        Self {
            base_url: "http://export.arxiv.org/api/query".to_string(),
        }
    }
}

impl Default for ArxivProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl SearchProvider for ArxivProvider {
    fn name(&self) -> &str {
        "arxiv"
    }

    async fn search(&self, options: &SearchOptions) -> SearchResult<Vec<SearchResultType>> {
        let client = reqwest::Client::new();
        let mut url = Url::parse(&self.base_url)?;

        // Build query parameters with proper lifetime management
        let search_query;
        let start_str;
        let max_results_str;

        let mut query_params = Vec::new();

        if let Some(id_list) = &options.id_list {
            // Search by specific ArXiv IDs
            query_params.push(("id_list", id_list.as_str()));
        } else if !options.query.trim().is_empty() {
            // Search by query string
            search_query = format!("all:{}", options.query.trim());
            query_params.push(("search_query", search_query.as_str()));
        } else {
            return Err(SearchError::InvalidInput(
                "ArXiv search requires either a search query or ID list".to_string(),
            ));
        }

        // Add pagination parameters
        if let Some(start) = options.start {
            start_str = start.to_string();
            query_params.push(("start", start_str.as_str()));
        }

        let max_results = options.max_results.unwrap_or(10).min(50); // ArXiv max is 50
        max_results_str = max_results.to_string();
        query_params.push(("max_results", max_results_str.as_str()));

        // Add sort parameters
        if let Some(sort_by) = &options.sort_by {
            let sort_by_str = match sort_by {
                crate::types::SortBy::Relevance => "relevance",
                crate::types::SortBy::SubmittedDate => "submittedDate",
                crate::types::SortBy::LastUpdatedDate => "lastUpdatedDate",
            };
            query_params.push(("sortBy", sort_by_str));
        }

        if let Some(sort_order) = &options.sort_order {
            let sort_order_str = match sort_order {
                crate::types::SortOrder::Ascending => "ascending",
                crate::types::SortOrder::Descending => "descending",
            };
            query_params.push(("sortOrder", sort_order_str));
        }

        url.query_pairs_mut().extend_pairs(query_params);

        if let Some(debug) = &options.debug {
            if debug.enabled && debug.log_requests {
                log::info!("ArXiv API request: {}", url);
            }
        }

        let response = client
            .get(url.as_str())
            .send()
            .await
            .map_err(|e| {
                SearchError::HttpError {
                    message: format!("ArXiv API request failed: {e}"),
                    status_code: None,
                    response_body: None,
                }
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());

            return Err(SearchError::HttpError {
                message: format!("ArXiv API returned error: {}", status),
                status_code: Some(status.as_u16()),
                response_body: Some(error_text),
            });
        }

        let xml_text = response.text().await.map_err(|e| {
            SearchError::ParseError(format!("Failed to read ArXiv response: {e}"))
        })?;

        if let Some(debug) = &options.debug {
            if debug.enabled && debug.log_responses {
                log::info!("ArXiv API response received ({} bytes)", xml_text.len());
            }
        }

        // Parse XML response
        let feed: ArxivFeed = quick_xml::de::from_str(&xml_text).map_err(|e| {
            SearchError::ParseError(format!("Failed to parse ArXiv XML: {e}"))
        })?;

        // Convert to standard format
        let results: Vec<SearchResultType> = feed
            .entries
            .into_iter()
            .map(|entry| {
                // Extract ArXiv ID from the full ID URL
                let arxiv_id = entry.id
                    .split('/')
                    .last()
                    .unwrap_or(&entry.id)
                    .to_string();

                // Find the paper URL
                let paper_url = entry.links
                    .iter()
                    .find(|link| link.link_type.as_deref() == Some("text/html"))
                    .map(|link| link.href.clone())
                    .unwrap_or_else(|| format!("https://arxiv.org/abs/{}", arxiv_id));

                // Create author list
                let authors: Vec<String> = entry.authors.iter().map(|a| a.name.clone()).collect();
                let authors_string = if authors.is_empty() {
                    None
                } else {
                    Some(authors.join(", "))
                };

                // Store raw data
                let mut raw_data = HashMap::new();
                raw_data.insert("arxiv_id".to_string(), serde_json::Value::String(arxiv_id.clone()));
                raw_data.insert("published".to_string(), serde_json::Value::String(entry.published.clone()));
                if let Some(authors_str) = &authors_string {
                    raw_data.insert("authors".to_string(), serde_json::Value::String(authors_str.clone()));
                }

                SearchResultType {
                    url: paper_url,
                    title: entry.title.trim().to_string(),
                    snippet: Some(entry.summary.trim().to_string()),
                    domain: Some("arxiv.org".to_string()),
                    published_date: Some(entry.published),
                    provider: Some("arxiv".to_string()),
                    raw: Some(serde_json::to_value(raw_data).unwrap_or_default()),
                }
            })
            .collect();

        Ok(results)
    }

    fn config(&self) -> HashMap<String, String> {
        let mut config = HashMap::new();
        config.insert("provider".to_string(), "arxiv".to_string());
        config.insert("base_url".to_string(), self.base_url.clone());
        config.insert("max_results".to_string(), "50".to_string());
        config
    }
}
