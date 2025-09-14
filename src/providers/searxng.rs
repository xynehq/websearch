//! SearXNG provider

use crate::{
    error::{SearchError, SearchResult},
    types::{SearchOptions, SearchProvider, SearchResult as SearchResultType},
};
use std::collections::HashMap;

#[derive(Debug)]
pub struct SearxNGProvider {
    base_url: String,
}

impl SearxNGProvider {
    pub fn new(base_url: &str) -> SearchResult<Self> {
        if base_url.is_empty() {
            return Err(SearchError::ConfigError(
                "SearXNG base URL is required".to_string(),
            ));
        }

        Ok(Self {
            base_url: base_url.to_string(),
        })
    }
}

#[async_trait::async_trait]
impl SearchProvider for SearxNGProvider {
    fn name(&self) -> &str {
        "searxng"
    }

    async fn search(&self, _options: &SearchOptions) -> SearchResult<Vec<SearchResultType>> {
        Err(SearchError::ProviderError(
            "SearXNG provider implementation coming soon".to_string(),
        ))
    }

    fn config(&self) -> HashMap<String, String> {
        let mut config = HashMap::new();
        config.insert("base_url".to_string(), self.base_url.clone());
        config
    }
}
