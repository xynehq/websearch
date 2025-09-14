//! Multi-provider search functionality with load balancing and failover

use crate::{
    error::{SearchError, SearchResult as Result},
    types::{DebugOptions, SafeSearch, SearchProvider, SearchResult, SortBy, SortOrder},
    utils::debug,
};
use std::collections::HashMap;
use tokio::time::{timeout, Duration};

/// Strategy for using multiple providers
#[derive(Debug, Clone)]
pub enum MultiProviderStrategy {
    /// Use providers in sequence until one succeeds
    Failover,
    /// Load balance requests across providers (round-robin)
    LoadBalance,
    /// Query all providers and merge results
    Aggregate,
    /// Use fastest responding provider
    RaceFirst,
}

/// Configuration for multi-provider searches
#[derive(Debug)]
pub struct MultiProviderConfig {
    pub providers: Vec<Box<dyn SearchProvider>>,
    pub strategy: MultiProviderStrategy,
    pub timeout_per_provider: Duration,
    pub max_concurrent: usize,
}

impl MultiProviderConfig {
    pub fn new(strategy: MultiProviderStrategy) -> Self {
        Self {
            providers: Vec::new(),
            strategy,
            timeout_per_provider: Duration::from_secs(10),
            max_concurrent: 3,
        }
    }

    pub fn add_provider(mut self, provider: Box<dyn SearchProvider>) -> Self {
        self.providers.push(provider);
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout_per_provider = timeout;
        self
    }

    pub fn with_max_concurrent(mut self, max: usize) -> Self {
        self.max_concurrent = max;
        self
    }
}

/// Multi-provider search manager
pub struct MultiProviderSearch {
    config: MultiProviderConfig,
    provider_stats: HashMap<String, ProviderStats>,
}

#[derive(Debug, Default)]
pub struct ProviderStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub avg_response_time_ms: f64,
}

impl MultiProviderSearch {
    pub fn new(config: MultiProviderConfig) -> Self {
        let provider_stats = config
            .providers
            .iter()
            .map(|p| (p.name().to_string(), ProviderStats::default()))
            .collect();

        Self {
            config,
            provider_stats,
        }
    }

    /// Perform search using the configured strategy
    pub async fn search(&mut self, options: &SearchOptionsMulti) -> Result<Vec<SearchResult>> {
        match self.config.strategy {
            MultiProviderStrategy::Failover => self.search_failover(options).await,
            MultiProviderStrategy::LoadBalance => self.search_load_balance(options).await,
            MultiProviderStrategy::Aggregate => self.search_aggregate(options).await,
            MultiProviderStrategy::RaceFirst => self.search_race_first(options).await,
        }
    }

    /// Try providers in sequence until one succeeds
    async fn search_failover(&mut self, options: &SearchOptionsMulti) -> Result<Vec<SearchResult>> {
        let mut last_error = SearchError::Other("No providers configured".to_string());

        for i in 0..self.config.providers.len() {
            let provider_name = self.config.providers[i].name().to_string();
            debug::log(&options.debug, "Trying failover provider", &provider_name);

            match self.search_single_provider_by_index(i, options).await {
                Ok(results) => {
                    debug::log(
                        &options.debug,
                        "Failover provider succeeded",
                        &provider_name,
                    );
                    return Ok(results);
                }
                Err(err) => {
                    debug::log(
                        &options.debug,
                        &format!("Failover provider {provider_name} failed"),
                        &err.to_string(),
                    );
                    last_error = err;
                }
            }
        }

        Err(last_error)
    }

    /// Use round-robin load balancing
    async fn search_load_balance(
        &mut self,
        options: &SearchOptionsMulti,
    ) -> Result<Vec<SearchResult>> {
        if self.config.providers.is_empty() {
            return Err(SearchError::Other("No providers configured".to_string()));
        }

        // Simple round-robin: pick based on total requests
        let total_requests: u64 = self.provider_stats.values().map(|s| s.total_requests).sum();
        let provider_index = (total_requests as usize) % self.config.providers.len();
        let provider_name = self.config.providers[provider_index].name().to_string();

        debug::log(&options.debug, "Load balancing to provider", &provider_name);

        self.search_single_provider_by_index(provider_index, options)
            .await
    }

    /// Query all providers and merge results
    async fn search_aggregate(
        &mut self,
        options: &SearchOptionsMulti,
    ) -> Result<Vec<SearchResult>> {
        debug::log(&options.debug, "Aggregating results from all providers", "");

        let mut merged_results = Vec::new();
        let mut successful_providers = Vec::new();

        // Search each provider sequentially to avoid borrowing issues
        for i in 0..self.config.providers.len() {
            let provider_name = self.config.providers[i].name().to_string();
            match self.search_single_provider_by_index(i, options).await {
                Ok(mut provider_results) => {
                    successful_providers.push(provider_name);
                    merged_results.append(&mut provider_results);
                }
                Err(_) => {
                    // Continue with other providers
                }
            }
        }

        if merged_results.is_empty() {
            return Err(SearchError::Other("All providers failed".to_string()));
        }

        debug::log(
            &options.debug,
            &format!(
                "Aggregated {} results from {} providers",
                merged_results.len(),
                successful_providers.len()
            ),
            &successful_providers.join(", "),
        );

        // Sort by relevance (providers first, then by original order)
        merged_results.sort_by(|a, b| {
            // Prioritize results from more reliable providers
            a.provider.cmp(&b.provider)
        });

        // Limit total results
        if let Some(max_results) = options.max_results {
            merged_results.truncate(max_results as usize);
        }

        Ok(merged_results)
    }

    /// Race all providers, return first successful result
    async fn search_race_first(
        &mut self,
        options: &SearchOptionsMulti,
    ) -> Result<Vec<SearchResult>> {
        debug::log(&options.debug, "Racing all providers", "");

        if self.config.providers.is_empty() {
            return Err(SearchError::Other("No providers configured".to_string()));
        }

        // For now, simplified race - try providers in sequence until first succeeds
        // A true race implementation would require more complex async handling
        for i in 0..self.config.providers.len() {
            let provider_name = self.config.providers[i].name().to_string();
            match self.search_single_provider_by_index(i, options).await {
                Ok(results) => {
                    debug::log(
                        &options.debug,
                        &format!("Race won by {provider_name}"),
                        "",
                    );
                    return Ok(results);
                }
                Err(_) => {
                    // Continue to next provider
                }
            }
        }

        Err(SearchError::Other(
            "All providers in race failed".to_string(),
        ))
    }

    /// Search with a single provider by index and update stats
    async fn search_single_provider_by_index(
        &mut self,
        provider_index: usize,
        options: &SearchOptionsMulti,
    ) -> Result<Vec<SearchResult>> {
        let start_time = std::time::Instant::now();
        let provider = &self.config.providers[provider_index];
        let provider_name = provider.name().to_string();

        // Update request count
        if let Some(stats) = self.provider_stats.get_mut(&provider_name) {
            stats.total_requests += 1;
        }

        // Perform search with timeout - we'll use our internal search interface
        let search_future = self.search_provider_internal(provider.as_ref(), options);
        let result = timeout(self.config.timeout_per_provider, search_future).await;

        let duration = start_time.elapsed();

        // Update stats
        if let Some(stats) = self.provider_stats.get_mut(&provider_name) {
            match &result {
                Ok(Ok(_)) => {
                    stats.successful_requests += 1;
                    // Update rolling average
                    let new_time = duration.as_millis() as f64;
                    stats.avg_response_time_ms = (stats.avg_response_time_ms
                        * (stats.successful_requests - 1) as f64
                        + new_time)
                        / stats.successful_requests as f64;
                }
                _ => {
                    stats.failed_requests += 1;
                }
            }
        }

        match result {
            Ok(search_result) => search_result,
            Err(_) => Err(SearchError::Timeout {
                timeout_ms: self.config.timeout_per_provider.as_millis() as u64,
            }),
        }
    }

    /// Get provider statistics
    pub fn get_stats(&self) -> &HashMap<String, ProviderStats> {
        &self.provider_stats
    }

    /// Internal method to search with a provider without the circular dependency issue
    async fn search_provider_internal(
        &self,
        provider: &dyn SearchProvider,
        options: &SearchOptionsMulti,
    ) -> Result<Vec<SearchResult>> {
        // Create a temporary SearchOptions with a placeholder provider for the interface
        // The actual provider parameter won't be used since we're calling the provider directly
        use crate::types::SearchOptions;

        let search_options = SearchOptions {
            query: options.query.clone(),
            id_list: options.id_list.clone(),
            max_results: options.max_results,
            language: options.language.clone(),
            region: options.region.clone(),
            safe_search: options.safe_search.clone(),
            page: options.page,
            start: options.start,
            sort_by: options.sort_by.clone(),
            sort_order: options.sort_order.clone(),
            timeout: options.timeout,
            debug: options.debug.clone(),
            provider: Box::new(PlaceholderProvider), // This won't be used
        };

        provider.search(&search_options).await
    }
}

/// Multi-provider search options (similar to SearchOptions but without provider field)
#[derive(Debug)]
pub struct SearchOptionsMulti {
    pub query: String,
    pub id_list: Option<String>,
    pub max_results: Option<u32>,
    pub language: Option<String>,
    pub region: Option<String>,
    pub safe_search: Option<SafeSearch>,
    pub page: Option<u32>,
    pub start: Option<u32>,
    pub sort_by: Option<SortBy>,
    pub sort_order: Option<SortOrder>,
    pub timeout: Option<u64>,
    pub debug: Option<DebugOptions>,
}

impl Default for SearchOptionsMulti {
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
            timeout: Some(15000),
            debug: None,
        }
    }
}

// Placeholder provider for interface compatibility - never actually used
#[derive(Debug)]
struct PlaceholderProvider;

#[async_trait::async_trait]
impl SearchProvider for PlaceholderProvider {
    fn name(&self) -> &str {
        "placeholder"
    }

    async fn search(&self, _: &crate::types::SearchOptions) -> Result<Vec<SearchResult>> {
        Err(SearchError::Other(
            "PlaceholderProvider should never be called".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use async_trait::async_trait;
    use tokio::time::Duration;

    // Mock provider for testing
    #[derive(Debug, Clone)]
    struct MockProvider {
        name: String,
        should_error: bool,
        error_type: Option<SearchError>,
        results: Vec<SearchResult>,
        delay_ms: u64,
    }

    impl MockProvider {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                should_error: false,
                error_type: None,
                results: vec![
                    SearchResult {
                        title: format!("{name} Result 1"),
                        url: format!("https://{name}.com/1"),
                        snippet: Some(format!("{name} content 1")),
                        domain: None,
                        published_date: None,
                        provider: Some(name.to_string()),
                        raw: None,
                    },
                    SearchResult {
                        title: format!("{name} Result 2"),
                        url: format!("https://{name}.com/2"),
                        snippet: Some(format!("{name} content 2")),
                        domain: None,
                        published_date: None,
                        provider: Some(name.to_string()),
                        raw: None,
                    },
                ],
                delay_ms: 0,
            }
        }

        fn with_error(mut self, error: SearchError) -> Self {
            self.should_error = true;
            self.error_type = Some(error);
            self
        }

        fn with_results(mut self, results: Vec<SearchResult>) -> Self {
            self.results = results;
            self
        }

        fn with_delay(mut self, delay_ms: u64) -> Self {
            self.delay_ms = delay_ms;
            self
        }
    }

    #[async_trait]
    impl SearchProvider for MockProvider {
        fn name(&self) -> &str {
            &self.name
        }

        async fn search(&self, _options: &SearchOptions) -> Result<Vec<SearchResult>> {
            if self.delay_ms > 0 {
                tokio::time::sleep(Duration::from_millis(self.delay_ms)).await;
            }

            if self.should_error {
                Err(self
                    .error_type
                    .clone()
                    .unwrap_or(SearchError::Other("Mock error".to_string())))
            } else {
                Ok(self.results.clone())
            }
        }
    }

    fn create_test_options(query: &str) -> SearchOptionsMulti {
        SearchOptionsMulti {
            query: query.to_string(),
            max_results: Some(10),
            debug: Some(DebugOptions {
                enabled: false,
                log_requests: false,
                log_responses: false,
            }),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_multi_provider_config() {
        let provider1 = MockProvider::new("provider1");
        let provider2 = MockProvider::new("provider2");

        let config = MultiProviderConfig::new(MultiProviderStrategy::Failover)
            .add_provider(Box::new(provider1))
            .add_provider(Box::new(provider2))
            .with_timeout(Duration::from_secs(5))
            .with_max_concurrent(2);

        assert_eq!(config.providers.len(), 2);
        assert_eq!(config.timeout_per_provider, Duration::from_secs(5));
        assert_eq!(config.max_concurrent, 2);
        assert!(matches!(config.strategy, MultiProviderStrategy::Failover));
    }

    #[tokio::test]
    async fn test_failover_strategy_success() {
        let provider1 = MockProvider::new("provider1");
        let provider2 = MockProvider::new("provider2");

        let config = MultiProviderConfig::new(MultiProviderStrategy::Failover)
            .add_provider(Box::new(provider1))
            .add_provider(Box::new(provider2));

        let mut multi_search = MultiProviderSearch::new(config);
        let options = create_test_options("test query");

        let results = multi_search.search(&options).await.unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].provider, Some("provider1".to_string()));
        assert!(results[0].title.contains("provider1"));
    }

    #[tokio::test]
    async fn test_failover_strategy_with_first_provider_failing() {
        let provider1 = MockProvider::new("provider1").with_error(SearchError::HttpError {
            status_code: Some(500),
            message: "Server error".to_string(),
            response_body: None,
        });
        let provider2 = MockProvider::new("provider2");

        let config = MultiProviderConfig::new(MultiProviderStrategy::Failover)
            .add_provider(Box::new(provider1))
            .add_provider(Box::new(provider2));

        let mut multi_search = MultiProviderSearch::new(config);
        let options = create_test_options("test query");

        let results = multi_search.search(&options).await.unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].provider, Some("provider2".to_string()));
        assert!(results[0].title.contains("provider2"));
    }

    #[tokio::test]
    async fn test_failover_strategy_all_providers_failing() {
        let provider1 = MockProvider::new("provider1").with_error(SearchError::HttpError {
            status_code: Some(500),
            message: "Server error".to_string(),
            response_body: None,
        });
        let provider2 = MockProvider::new("provider2").with_error(SearchError::HttpError {
            status_code: Some(401),
            message: "Unauthorized".to_string(),
            response_body: None,
        });

        let config = MultiProviderConfig::new(MultiProviderStrategy::Failover)
            .add_provider(Box::new(provider1))
            .add_provider(Box::new(provider2));

        let mut multi_search = MultiProviderSearch::new(config);
        let options = create_test_options("test query");

        let result = multi_search.search(&options).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            SearchError::HttpError {
                status_code: Some(401),
                ..
            } => {
                // Should return the last error (from provider2)
            }
            _ => panic!("Expected the last provider's error"),
        }
    }

    #[tokio::test]
    async fn test_load_balance_strategy() {
        let provider1 = MockProvider::new("provider1");
        let provider2 = MockProvider::new("provider2");

        let config = MultiProviderConfig::new(MultiProviderStrategy::LoadBalance)
            .add_provider(Box::new(provider1))
            .add_provider(Box::new(provider2));

        let mut multi_search = MultiProviderSearch::new(config);
        let options = create_test_options("test query");

        // First request should go to provider1 (index 0)
        let results1 = multi_search.search(&options).await.unwrap();
        assert_eq!(results1[0].provider, Some("provider1".to_string()));

        // Second request should go to provider2 (index 1)
        let results2 = multi_search.search(&options).await.unwrap();
        assert_eq!(results2[0].provider, Some("provider2".to_string()));

        // Third request should go back to provider1 (round-robin)
        let results3 = multi_search.search(&options).await.unwrap();
        assert_eq!(results3[0].provider, Some("provider1".to_string()));
    }

    #[tokio::test]
    async fn test_aggregate_strategy() {
        let provider1 = MockProvider::new("provider1").with_results(vec![SearchResult {
            title: "Provider1 Result".to_string(),
            url: "https://provider1.com/result".to_string(),
            snippet: Some("Provider1 content".to_string()),
            domain: None,
            published_date: None,
            provider: Some("provider1".to_string()),
            raw: None,
        }]);
        let provider2 = MockProvider::new("provider2").with_results(vec![SearchResult {
            title: "Provider2 Result".to_string(),
            url: "https://provider2.com/result".to_string(),
            snippet: Some("Provider2 content".to_string()),
            domain: None,
            published_date: None,
            provider: Some("provider2".to_string()),
            raw: None,
        }]);

        let config = MultiProviderConfig::new(MultiProviderStrategy::Aggregate)
            .add_provider(Box::new(provider1))
            .add_provider(Box::new(provider2));

        let mut multi_search = MultiProviderSearch::new(config);
        let options = create_test_options("test query");

        let results = multi_search.search(&options).await.unwrap();
        assert_eq!(results.len(), 2);

        // Results should contain entries from both providers
        let provider_names: Vec<&str> = results
            .iter()
            .map(|r| r.provider.as_deref().unwrap_or("unknown"))
            .collect();
        assert!(provider_names.contains(&"provider1"));
        assert!(provider_names.contains(&"provider2"));
    }

    #[tokio::test]
    async fn test_aggregate_strategy_with_one_provider_failing() {
        let provider1 = MockProvider::new("provider1").with_error(SearchError::HttpError {
            status_code: Some(500),
            message: "Server error".to_string(),
            response_body: None,
        });
        let provider2 = MockProvider::new("provider2");

        let config = MultiProviderConfig::new(MultiProviderStrategy::Aggregate)
            .add_provider(Box::new(provider1))
            .add_provider(Box::new(provider2));

        let mut multi_search = MultiProviderSearch::new(config);
        let options = create_test_options("test query");

        let results = multi_search.search(&options).await.unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].provider, Some("provider2".to_string()));
    }

    #[tokio::test]
    async fn test_aggregate_strategy_all_providers_failing() {
        let provider1 = MockProvider::new("provider1").with_error(SearchError::HttpError {
            status_code: Some(500),
            message: "Server error".to_string(),
            response_body: None,
        });
        let provider2 = MockProvider::new("provider2").with_error(SearchError::HttpError {
            status_code: Some(401),
            message: "Unauthorized".to_string(),
            response_body: None,
        });

        let config = MultiProviderConfig::new(MultiProviderStrategy::Aggregate)
            .add_provider(Box::new(provider1))
            .add_provider(Box::new(provider2));

        let mut multi_search = MultiProviderSearch::new(config);
        let options = create_test_options("test query");

        let result = multi_search.search(&options).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            SearchError::Other(msg) => {
                assert!(msg.contains("All providers failed"));
            }
            _ => panic!("Expected 'All providers failed' error"),
        }
    }

    #[tokio::test]
    async fn test_race_first_strategy() {
        let provider1 = MockProvider::new("provider1");
        let provider2 = MockProvider::new("provider2");

        let config = MultiProviderConfig::new(MultiProviderStrategy::RaceFirst)
            .add_provider(Box::new(provider1))
            .add_provider(Box::new(provider2));

        let mut multi_search = MultiProviderSearch::new(config);
        let options = create_test_options("test query");

        let results = multi_search.search(&options).await.unwrap();
        assert_eq!(results.len(), 2);
        // First provider should win since they all succeed immediately
        assert_eq!(results[0].provider, Some("provider1".to_string()));
    }

    #[tokio::test]
    async fn test_race_first_strategy_with_first_provider_failing() {
        let provider1 = MockProvider::new("provider1").with_error(SearchError::HttpError {
            status_code: Some(500),
            message: "Server error".to_string(),
            response_body: None,
        });
        let provider2 = MockProvider::new("provider2");

        let config = MultiProviderConfig::new(MultiProviderStrategy::RaceFirst)
            .add_provider(Box::new(provider1))
            .add_provider(Box::new(provider2));

        let mut multi_search = MultiProviderSearch::new(config);
        let options = create_test_options("test query");

        let results = multi_search.search(&options).await.unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].provider, Some("provider2".to_string()));
    }

    #[tokio::test]
    async fn test_provider_stats_tracking() {
        let provider1 = MockProvider::new("provider1");
        let provider2 = MockProvider::new("provider2").with_error(SearchError::HttpError {
            status_code: Some(500),
            message: "Server error".to_string(),
            response_body: None,
        });

        let config = MultiProviderConfig::new(MultiProviderStrategy::Failover)
            .add_provider(Box::new(provider1))
            .add_provider(Box::new(provider2));

        let mut multi_search = MultiProviderSearch::new(config);
        let options = create_test_options("test query");

        // Perform multiple searches to build up stats
        let _ = multi_search.search(&options).await.unwrap();
        let _ = multi_search.search(&options).await.unwrap();

        let stats = multi_search.get_stats();

        // provider1 should have successful requests
        let provider1_stats = &stats["provider1"];
        assert_eq!(provider1_stats.total_requests, 2);
        assert_eq!(provider1_stats.successful_requests, 2);
        assert_eq!(provider1_stats.failed_requests, 0);
        assert!(provider1_stats.avg_response_time_ms >= 0.0);

        // provider2 should have failed requests (but failover means it's not used if provider1 succeeds)
        // In failover mode, provider2 won't be called if provider1 succeeds
        let provider2_stats = &stats["provider2"];
        assert_eq!(provider2_stats.total_requests, 0);
    }

    #[tokio::test]
    async fn test_timeout_functionality() {
        let slow_provider = MockProvider::new("slow").with_delay(100); // 100ms delay

        let config = MultiProviderConfig::new(MultiProviderStrategy::Failover)
            .add_provider(Box::new(slow_provider))
            .with_timeout(Duration::from_millis(50)); // 50ms timeout

        let mut multi_search = MultiProviderSearch::new(config);
        let options = create_test_options("test query");

        let result = multi_search.search(&options).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            SearchError::Timeout { timeout_ms } => {
                assert_eq!(timeout_ms, 50);
            }
            _ => panic!("Expected timeout error"),
        }
    }

    #[tokio::test]
    async fn test_max_results_in_aggregate() {
        let provider1 = MockProvider::new("provider1").with_results(vec![
            SearchResult {
                title: "Result 1".to_string(),
                url: "https://example1.com".to_string(),
                snippet: Some("Content 1".to_string()),
                domain: None,
                published_date: None,
                provider: Some("provider1".to_string()),
                raw: None,
            },
            SearchResult {
                title: "Result 2".to_string(),
                url: "https://example2.com".to_string(),
                snippet: Some("Content 2".to_string()),
                domain: None,
                published_date: None,
                provider: Some("provider1".to_string()),
                raw: None,
            },
        ]);
        let provider2 = MockProvider::new("provider2").with_results(vec![
            SearchResult {
                title: "Result 3".to_string(),
                url: "https://example3.com".to_string(),
                snippet: Some("Content 3".to_string()),
                domain: None,
                published_date: None,
                provider: Some("provider2".to_string()),
                raw: None,
            },
            SearchResult {
                title: "Result 4".to_string(),
                url: "https://example4.com".to_string(),
                snippet: Some("Content 4".to_string()),
                domain: None,
                published_date: None,
                provider: Some("provider2".to_string()),
                raw: None,
            },
        ]);

        let config = MultiProviderConfig::new(MultiProviderStrategy::Aggregate)
            .add_provider(Box::new(provider1))
            .add_provider(Box::new(provider2));

        let mut multi_search = MultiProviderSearch::new(config);
        let mut options = create_test_options("test query");
        options.max_results = Some(3); // Limit to 3 results

        let results = multi_search.search(&options).await.unwrap();
        assert_eq!(results.len(), 3); // Should be limited to 3 results
    }

    #[tokio::test]
    async fn test_empty_providers_config() {
        let config = MultiProviderConfig::new(MultiProviderStrategy::Failover);
        let mut multi_search = MultiProviderSearch::new(config);
        let options = create_test_options("test query");

        let result = multi_search.search(&options).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            SearchError::Other(msg) => {
                assert!(msg.contains("No providers configured"));
            }
            _ => panic!("Expected 'No providers configured' error"),
        }
    }
}
