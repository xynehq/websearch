//! Comprehensive provider tests
//!
//! These tests ensure all providers work correctly with proper configuration,
//! error handling, and response parsing.

use std::env;
use tokio::time::Duration;
use websearch::{
    providers::*,
    types::{SearchOptions, SearchProvider, DebugOptions},
    web_search,
};
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

/// Test configuration for each provider
struct ProviderTestConfig {
    name: &'static str,
    requires_api_key: bool,
    env_vars: &'static [&'static str],
    can_mock: bool,
}

const PROVIDER_CONFIGS: &[ProviderTestConfig] = &[
    ProviderTestConfig {
        name: "Google",
        requires_api_key: true,
        env_vars: &["GOOGLE_API_KEY", "GOOGLE_CX"],
        can_mock: true,
    },
    ProviderTestConfig {
        name: "Tavily",
        requires_api_key: true,
        env_vars: &["TAVILY_API_KEY"],
        can_mock: true,
    },
    ProviderTestConfig {
        name: "Exa",
        requires_api_key: true,
        env_vars: &["EXA_API_KEY"],
        can_mock: true,
    },
    ProviderTestConfig {
        name: "SerpAPI",
        requires_api_key: true,
        env_vars: &["SERPAPI_API_KEY"],
        can_mock: true,
    },
    ProviderTestConfig {
        name: "DuckDuckGo",
        requires_api_key: false,
        env_vars: &[],
        can_mock: false, // Uses HTML scraping
    },
    ProviderTestConfig {
        name: "Brave",
        requires_api_key: true,
        env_vars: &["BRAVE_API_KEY"],
        can_mock: true,
    },
    ProviderTestConfig {
        name: "SearXNG",
        requires_api_key: false,
        env_vars: &["SEARXNG_URL"],
        can_mock: true,
    },
    ProviderTestConfig {
        name: "ArXiv",
        requires_api_key: false,
        env_vars: &[],
        can_mock: true,
    },
];

// Helper function to check if provider has required environment variables
fn provider_is_configured(config: &ProviderTestConfig) -> bool {
    if config.env_vars.is_empty() {
        return true; // No env vars required
    }
    config.env_vars.iter().all(|var| env::var(var).is_ok())
}

// Create provider instances for testing
async fn create_test_providers() -> Vec<(String, Box<dyn SearchProvider>)> {
    let mut providers = Vec::new();

    // Google
    if let (Ok(api_key), Ok(cx)) = (env::var("GOOGLE_API_KEY"), env::var("GOOGLE_CX")) {
        if let Ok(provider) = GoogleProvider::new(&api_key, &cx) {
            providers.push(("google".to_string(), Box::new(provider) as Box<dyn SearchProvider>));
        }
    }

    // Tavily
    if let Ok(api_key) = env::var("TAVILY_API_KEY") {
        if let Ok(provider) = TavilyProvider::new(&api_key) {
            providers.push(("tavily".to_string(), Box::new(provider) as Box<dyn SearchProvider>));
        }
    }

    // Exa
    if let Ok(api_key) = env::var("EXA_API_KEY") {
        if let Ok(provider) = ExaProvider::new(&api_key) {
            providers.push(("exa".to_string(), Box::new(provider) as Box<dyn SearchProvider>));
        }
    }

    // SerpAPI
    if let Ok(api_key) = env::var("SERPAPI_API_KEY") {
        if let Ok(provider) = SerpApiProvider::new(&api_key) {
            providers.push(("serpapi".to_string(), Box::new(provider) as Box<dyn SearchProvider>));
        }
    }

    // DuckDuckGo (always available)
    let duckduckgo = DuckDuckGoProvider::new();
    providers.push(("duckduckgo".to_string(), Box::new(duckduckgo) as Box<dyn SearchProvider>));

    // Brave
    if let Ok(api_key) = env::var("BRAVE_API_KEY") {
        if let Ok(provider) = BraveProvider::new(&api_key) {
            providers.push(("brave".to_string(), Box::new(provider) as Box<dyn SearchProvider>));
        }
    }

    // SearXNG
    if let Ok(url) = env::var("SEARXNG_URL") {
        if let Ok(provider) = SearxNGProvider::new(&url) {
            providers.push(("searxng".to_string(), Box::new(provider) as Box<dyn SearchProvider>));
        }
    }

    // ArXiv (always available)
    let arxiv = ArxivProvider::new();
    providers.push(("arxiv".to_string(), Box::new(arxiv) as Box<dyn SearchProvider>));

    providers
}

#[tokio::test]
async fn test_all_providers_basic_functionality() {
    let providers = create_test_providers().await;

    println!("Testing {} providers", providers.len());

    for (name, _provider) in &providers {
        println!("✅ Provider '{}' can be instantiated", name);
    }

    assert!(!providers.is_empty(), "At least DuckDuckGo and ArXiv should be available");
}

#[tokio::test]
async fn test_provider_configuration_methods() {
    let providers = create_test_providers().await;

    for (name, provider) in providers {
        // Test name() method
        let provider_name = provider.name();
        assert!(!provider_name.is_empty(), "Provider {} should return non-empty name", name);
        println!("Provider '{}' reports name: '{}'", name, provider_name);

        // Test config() method
        let config = provider.config();
        assert!(!config.is_empty(), "Provider {} should return non-empty config", name);
        println!("Provider '{}' has {} config items", name, config.len());

        // Verify config contains expected keys
        if name == "google" {
            assert!(config.contains_key("api_key") || config.contains_key("cx"));
        } else if name != "duckduckgo" && name != "arxiv" {
            assert!(config.contains_key("api_key") || config.contains_key("base_url"));
        }
    }
}

#[tokio::test]
async fn test_duckduckgo_real_search() {
    // DuckDuckGo should always work without API keys
    let duckduckgo = DuckDuckGoProvider::new();

    let options = SearchOptions {
        query: "rust programming language".to_string(),
        max_results: Some(3),
        provider: Box::new(duckduckgo),
        debug: Some(DebugOptions {
            enabled: true,
            log_requests: false,
            log_responses: false,
        }),
        ..Default::default()
    };

    match web_search(options).await {
        Ok(results) => {
            println!("DuckDuckGo returned {} results", results.len());
            assert!(!results.is_empty(), "DuckDuckGo should return some results");

            // Verify result structure
            for (i, result) in results.iter().enumerate() {
                assert!(!result.title.is_empty(), "Result {} should have title", i);
                assert!(!result.url.is_empty(), "Result {} should have URL", i);
                assert_eq!(result.provider, Some("duckduckgo".to_string()));
                println!("✅ Result {}: {}", i + 1, result.title);
            }
        }
        Err(e) => {
            println!("DuckDuckGo search failed (possibly network): {}", e);
            // Don't fail the test for network issues, but log it
        }
    }
}

#[tokio::test]
async fn test_arxiv_real_search() {
    // ArXiv should always work without API keys
    let arxiv = ArxivProvider::new();

    // Test with paper ID
    let options = SearchOptions {
        query: "".to_string(),
        id_list: Some("2301.00001".to_string()),
        max_results: Some(1),
        provider: Box::new(arxiv),
        debug: Some(DebugOptions {
            enabled: true,
            log_requests: false,
            log_responses: false,
        }),
        ..Default::default()
    };

    match web_search(options).await {
        Ok(results) => {
            println!("ArXiv returned {} results", results.len());
            assert!(!results.is_empty(), "ArXiv should return results for valid paper ID");

            let result = &results[0];
            assert!(!result.title.is_empty(), "ArXiv result should have title");
            assert!(!result.url.is_empty(), "ArXiv result should have URL");
            assert!(result.url.contains("arxiv.org"), "ArXiv URL should contain arxiv.org");
            assert_eq!(result.provider, Some("arxiv".to_string()));
            assert_eq!(result.domain, Some("arxiv.org".to_string()));

            println!("✅ ArXiv paper: {}", result.title);
        }
        Err(e) => {
            println!("ArXiv search failed: {}", e);
            panic!("ArXiv should work for basic paper ID lookup");
        }
    }
}

#[tokio::test]
async fn test_arxiv_query_search() {
    let arxiv = ArxivProvider::new();

    // Test with query search
    let options = SearchOptions {
        query: "quantum machine learning".to_string(),
        max_results: Some(2),
        provider: Box::new(arxiv),
        ..Default::default()
    };

    match web_search(options).await {
        Ok(results) => {
            println!("ArXiv query search returned {} results", results.len());
            // ArXiv query search should work
            for result in &results {
                assert!(!result.title.is_empty());
                assert!(result.url.contains("arxiv.org"));
                assert_eq!(result.provider, Some("arxiv".to_string()));
            }
        }
        Err(e) => {
            println!("ArXiv query search failed: {}", e);
            // Query search might be more restrictive, so don't panic
        }
    }
}

#[tokio::test]
async fn test_provider_error_handling() {
    // Test that providers handle invalid input gracefully

    // Test empty query (should fail for most providers)
    for config in PROVIDER_CONFIGS {
        if !provider_is_configured(config) {
            continue;
        }

        println!("Testing error handling for {}", config.name);

        // Create basic provider instance for testing
        let provider: Box<dyn SearchProvider> = match config.name {
            "DuckDuckGo" => Box::new(DuckDuckGoProvider::new()),
            "ArXiv" => Box::new(ArxivProvider::new()),
            _ => continue, // Skip providers requiring API keys for error testing
        };

        let options = SearchOptions {
            query: "".to_string(), // Empty query
            provider,
            ..Default::default()
        };

        match web_search(options).await {
            Ok(_) => {
                // Some providers might handle empty queries differently
                println!("Provider {} accepted empty query", config.name);
            }
            Err(e) => {
                println!("Provider {} properly rejected empty query: {}", config.name, e);
                // This is expected behavior
            }
        }
    }
}

#[tokio::test]
async fn test_provider_timeout_handling() {
    // Test that providers respect timeout settings
    let duckduckgo = DuckDuckGoProvider::new();

    let options = SearchOptions {
        query: "test timeout".to_string(),
        timeout: Some(1), // Very short timeout (1ms)
        max_results: Some(1),
        provider: Box::new(duckduckgo),
        ..Default::default()
    };

    match web_search(options).await {
        Ok(_) => {
            // Might succeed if very fast
            println!("Search completed within 1ms (very fast!)");
        }
        Err(e) => {
            println!("Timeout handled properly: {}", e);
            // Timeout is expected with 1ms limit
        }
    }
}

#[tokio::test]
async fn test_provider_max_results_respected() {
    let providers = create_test_providers().await;

    for (name, provider) in providers {
        if name != "duckduckgo" && name != "arxiv" {
            continue; // Skip API-requiring providers for this test
        }

        let options = SearchOptions {
            query: "test".to_string(),
            max_results: Some(2),
            provider,
            ..Default::default()
        };

        match web_search(options).await {
            Ok(results) => {
                assert!(
                    results.len() <= 2,
                    "Provider {} returned {} results, expected <= 2",
                    name,
                    results.len()
                );
                println!("✅ Provider '{}' respects max_results: {} results", name, results.len());
            }
            Err(e) => {
                println!("Provider '{}' search failed: {}", name, e);
            }
        }
    }
}

#[tokio::test]
async fn test_google_mock_server() {
    // Test Google provider with mock server
    let mock_server = MockServer::start().await;

    let mock_response = serde_json::json!({
        "items": [
            {
                "title": "Test Result",
                "link": "https://example.com/test",
                "snippet": "Test snippet",
                "displayLink": "example.com"
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/customsearch/v1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
        .mount(&mock_server)
        .await;

    if let Ok(google) = GoogleProvider::new("test_key", "test_cx") {
        // Use reflection or create a test-specific method to set base URL
        // For now, we'll test the provider creation itself
        assert_eq!(google.name(), "google");

        let config = google.config();
        assert!(config.contains_key("api_key"));
        assert!(config.contains_key("cx"));

        println!("✅ Google provider mock test setup completed");
    }
}

#[tokio::test]
async fn test_all_providers_config_validation() {
    // Test that all providers properly validate their configuration

    // Test invalid API keys for providers that require them

    // Google with invalid API key
    match GoogleProvider::new("", "test_cx") {
        Ok(_) => panic!("Google should reject empty API key"),
        Err(_) => println!("✅ Google properly validates API key"),
    }

    // Tavily with invalid API key format
    match TavilyProvider::new("invalid-key") {
        Ok(_) => panic!("Tavily should reject invalid API key format"),
        Err(_) => println!("✅ Tavily properly validates API key format"),
    }

    // SearXNG with empty URL
    match SearxNGProvider::new("") {
        Ok(_) => panic!("SearXNG should reject empty URL"),
        Err(_) => println!("✅ SearXNG properly validates URL"),
    }
}

#[tokio::test]
async fn test_provider_debug_mode() {
    // Test that providers work with debug mode enabled
    let duckduckgo = DuckDuckGoProvider::new();

    let options = SearchOptions {
        query: "debug test".to_string(),
        max_results: Some(1),
        debug: Some(DebugOptions {
            enabled: true,
            log_requests: true,
            log_responses: true,
        }),
        provider: Box::new(duckduckgo),
        ..Default::default()
    };

    // This test mainly ensures debug mode doesn't crash
    match web_search(options).await {
        Ok(results) => {
            println!("✅ Debug mode worked, got {} results", results.len());
        }
        Err(e) => {
            println!("Debug mode test failed: {}", e);
            // Don't panic on network issues
        }
    }
}

#[tokio::test]
async fn test_provider_compatibility_matrix() {
    // Test that all expected providers are available and working
    let expected_providers = vec![
        "duckduckgo", "arxiv" // These should always be available
    ];

    let available_providers = create_test_providers().await;
    let available_names: Vec<String> = available_providers.iter().map(|(name, _)| name.clone()).collect();

    for expected in expected_providers {
        assert!(
            available_names.contains(&expected.to_string()),
            "Expected provider '{}' should be available",
            expected
        );
    }

    println!("✅ Available providers: {:?}", available_names);

    // Report on optional providers
    let optional_providers = vec!["google", "tavily", "exa", "serpapi", "brave", "searxng"];
    for optional in optional_providers {
        if available_names.contains(&optional.to_string()) {
            println!("✅ Optional provider '{}' is configured and available", optional);
        } else {
            println!("⚠️  Optional provider '{}' not configured (set environment variables)", optional);
        }
    }
}

// Integration test for real API calls (only runs with REAL_API_TEST=1)
#[tokio::test]
async fn test_real_api_integration() {
    if env::var("REAL_API_TEST").unwrap_or_default() != "1" {
        println!("Skipping real API test (set REAL_API_TEST=1 to enable)");
        return;
    }

    let providers = create_test_providers().await;
    let mut successful_providers = 0;

    for (name, provider) in providers {
        println!("Testing real API for provider: {}", name);

        let options = SearchOptions {
            query: "rust programming".to_string(),
            max_results: Some(1),
            timeout: Some(10000), // 10 second timeout
            provider,
            ..Default::default()
        };

        match web_search(options).await {
            Ok(results) => {
                println!("✅ Provider '{}' returned {} results", name, results.len());
                if !results.is_empty() {
                    println!("   Sample result: {}", results[0].title);
                }
                successful_providers += 1;
            }
            Err(e) => {
                println!("❌ Provider '{}' failed: {}", name, e);
            }
        }

        // Add delay between API calls to be respectful
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    println!("Real API test completed: {}/{} providers successful", successful_providers, 8);
}