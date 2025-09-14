//! Arxiv API provider

use crate::{
    error::{SearchError, SearchResult},
    types::{SearchOptions, SearchProvider, SearchResult as SearchResultType},
};
use std::collections::HashMap;

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

    async fn search(&self, _options: &SearchOptions) -> SearchResult<Vec<SearchResultType>> {
        Err(SearchError::ProviderError(
            "Arxiv provider implementation coming soon".to_string(),
        ))
    }

    fn config(&self) -> HashMap<String, String> {
        let mut config = HashMap::new();
        config.insert("base_url".to_string(), self.base_url.clone());
        config
    }
}
