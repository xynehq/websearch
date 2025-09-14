//! Brave Search API provider

use crate::{
    error::{SearchError, SearchResult},
    types::{SearchOptions, SearchProvider, SearchResult as SearchResultType},
};
use std::collections::HashMap;

/// Brave Search provider (stub implementation)
#[derive(Debug)]
pub struct BraveProvider {
    api_key: String,
}

impl BraveProvider {
    pub fn new(api_key: &str) -> SearchResult<Self> {
        if api_key.is_empty() {
            return Err(SearchError::ConfigError(
                "Brave API key is required".to_string(),
            ));
        }

        Ok(Self {
            api_key: api_key.to_string(),
        })
    }
}

#[async_trait::async_trait]
impl SearchProvider for BraveProvider {
    fn name(&self) -> &str {
        "brave"
    }

    async fn search(&self, _options: &SearchOptions) -> SearchResult<Vec<SearchResultType>> {
        Err(SearchError::ProviderError(
            "Brave provider implementation coming soon".to_string(),
        ))
    }

    fn config(&self) -> HashMap<String, String> {
        let mut config = HashMap::new();
        config.insert("api_key".to_string(), "***".to_string());
        config
    }
}
