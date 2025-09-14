//! Comprehensive integration tests for Tavily provider
//!
//! These tests cover API integration, error handling, rate limiting,
//! and various response scenarios using mock servers.

use serde_json::json;
use std::time::Duration;
use websearch::{
    error::SearchError,
    providers::tavily::TavilyProvider,
    types::{DebugOptions, SearchOptions, SearchProvider},
    web_search,
};
use wiremock::{
    matchers::{body_partial_json, header, method, path},
    Mock, MockServer, ResponseTemplate,
};

async fn setup_mock_server() -> MockServer {
    MockServer::start().await
}

fn create_test_options_with_provider(provider: TavilyProvider, query: &str) -> SearchOptions {
    SearchOptions {
        query: query.to_string(),
        max_results: Some(5),
        debug: Some(DebugOptions {
            enabled: true,
            log_requests: true,
            log_responses: true,
        }),
        provider: Box::new(provider),
        ..Default::default()
    }
}

fn create_successful_tavily_response() -> serde_json::Value {
    json!({
        "answer": "Rust is a systems programming language focused on safety and performance.",
        "query": "rust programming language",
        "response_time": 1.23,
        "images": [],
        "results": [
            {
                "title": "The Rust Programming Language",
                "url": "https://www.rust-lang.org/",
                "content": "Rust is a language empowering everyone to build reliable and efficient software. Rust is blazingly fast and memory-efficient with no runtime or garbage collector.",
                "score": 0.95,
                "published_date": "2024-01-15"
            },
            {
                "title": "Rust Tutorial - Learn Rust Programming",
                "url": "https://doc.rust-lang.org/book/",
                "content": "The Rust Programming Language book is the official guide to learning Rust. It covers everything from basic syntax to advanced concepts.",
                "score": 0.88,
                "published_date": "2024-01-10"
            },
            {
                "title": "Why Rust is the Future of Systems Programming",
                "url": "https://example.com/rust-future",
                "content": "Rust provides memory safety without garbage collection, making it ideal for system-level programming where performance matters.",
                "score": 0.82,
                "published_date": null
            }
        ],
        "follow_up_questions": [
            "What are the main features of Rust?",
            "How does Rust compare to C++?",
            "What companies use Rust in production?"
        ]
    })
}

#[tokio::test]
async fn test_tavily_successful_search() {
    let mock_server = setup_mock_server().await;

    // Setup successful response mock - note the base URL includes /search
    Mock::given(method("POST"))
        .and(path("/")) // Changed from "/search" to "/" since base_url includes /search
        .and(header("content-type", "application/json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_successful_tavily_response()))
        .mount(&mock_server)
        .await;

    let provider = TavilyProvider::new("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")
        .unwrap()
        .with_base_url(&mock_server.uri());

    let options = create_test_options_with_provider(provider, "rust programming language");
    let results = web_search(options).await.unwrap();

    assert_eq!(results.len(), 3);
    assert_eq!(results[0].title, "The Rust Programming Language");
    assert_eq!(results[0].url, "https://www.rust-lang.org/");
    assert_eq!(results[0].provider, Some("tavily".to_string()));
    assert!(results[0].snippet.is_some());
    assert_eq!(results[0].domain, Some("www.rust-lang.org".to_string()));
    assert_eq!(results[0].published_date, Some("2024-01-15".to_string()));

    // Check that raw data is preserved
    assert!(results[0].raw.is_some());
    let raw = results[0].raw.as_ref().unwrap();
    assert_eq!(raw["score"], 0.95);
}

#[tokio::test]
async fn test_tavily_search_with_advanced_depth() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("POST"))
        .and(path("/"))
        .and(body_partial_json(json!({
            "search_depth": "advanced",
            "include_raw_content": true
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_successful_tavily_response()))
        .mount(&mock_server)
        .await;

    let provider = TavilyProvider::new_advanced("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")
        .unwrap()
        .with_base_url(&mock_server.uri());

    let options = create_test_options_with_provider(provider, "advanced rust concepts");
    let results = web_search(options).await.unwrap();

    assert!(!results.is_empty());
}

#[tokio::test]
async fn test_tavily_search_with_images() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("POST"))
        .and(path("/"))
        .and(body_partial_json(json!({
            "include_images": true
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "answer": null,
            "query": "rust programming examples",
            "response_time": 0.8,
            "images": [
                "https://example.com/rust-logo.png",
                "https://example.com/code-example.png"
            ],
            "results": [],
            "follow_up_questions": []
        })))
        .mount(&mock_server)
        .await;

    let provider = TavilyProvider::new("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")
        .unwrap()
        .with_images(true)
        .with_base_url(&mock_server.uri());

    let options = create_test_options_with_provider(provider, "rust programming examples");
    let results = web_search(options).await.unwrap();

    assert_eq!(results.len(), 0); // No text results in this response
}

#[tokio::test]
async fn test_tavily_api_key_validation_errors() {
    let mock_server = setup_mock_server().await;

    // Test 401 Unauthorized
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": "Unauthorized",
            "message": "Invalid API key"
        })))
        .mount(&mock_server)
        .await;

    let provider = TavilyProvider::new("tvly-dev-invalid-key")
        .unwrap()
        .with_base_url(&mock_server.uri());

    let options = create_test_options_with_provider(provider, "test query");
    let result = web_search(options).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        SearchError::ProviderError(msg) => {
            assert!(msg.contains("401"));
            assert!(msg.contains("Unauthorized"));
        }
        _ => panic!("Expected ProviderError with 401 status"),
    }
}

#[tokio::test]
async fn test_tavily_rate_limit_handling() {
    let mock_server = setup_mock_server().await;

    // Test 429 Rate Limit
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(429).set_body_json(json!({
            "error": "Rate limit exceeded",
            "message": "Too many requests. Please try again later."
        })))
        .mount(&mock_server)
        .await;

    let provider = TavilyProvider::new("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")
        .unwrap()
        .with_base_url(&mock_server.uri());

    let options = create_test_options_with_provider(provider, "test query");
    let result = web_search(options).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        SearchError::ProviderError(msg) => {
            assert!(msg.contains("429"));
            assert!(msg.contains("Rate limit exceeded"));
        }
        _ => panic!("Expected ProviderError with 429 status"),
    }
}

#[tokio::test]
async fn test_tavily_payment_required_error() {
    let mock_server = setup_mock_server().await;

    // Test 402 Payment Required
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(402).set_body_json(json!({
            "error": "Payment required",
            "message": "Your account has insufficient credits"
        })))
        .mount(&mock_server)
        .await;

    let provider = TavilyProvider::new("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")
        .unwrap()
        .with_base_url(&mock_server.uri());

    let options = create_test_options_with_provider(provider, "test query");
    let result = web_search(options).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        SearchError::ProviderError(msg) => {
            assert!(msg.contains("402"));
            assert!(msg.contains("Payment required"));
        }
        _ => panic!("Expected ProviderError with 402 status"),
    }
}

#[tokio::test]
async fn test_tavily_server_error_handling() {
    let mock_server = setup_mock_server().await;

    // Test 500 Server Error
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "error": "Internal server error",
            "message": "Something went wrong on our end"
        })))
        .mount(&mock_server)
        .await;

    let provider = TavilyProvider::new("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")
        .unwrap()
        .with_base_url(&mock_server.uri());

    let options = create_test_options_with_provider(provider, "test query");
    let result = web_search(options).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        SearchError::ProviderError(msg) => {
            assert!(msg.contains("500"));
            assert!(msg.contains("server error"));
        }
        _ => panic!("Expected ProviderError with 500 status"),
    }
}

#[tokio::test]
async fn test_tavily_malformed_response() {
    let mock_server = setup_mock_server().await;

    // Test malformed JSON response
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string("invalid json response"))
        .mount(&mock_server)
        .await;

    let provider = TavilyProvider::new("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")
        .unwrap()
        .with_base_url(&mock_server.uri());

    let options = create_test_options_with_provider(provider, "test query");
    let result = web_search(options).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        SearchError::ProviderError(msg) => {
            assert!(msg.contains("Parsing error"));
        }
        _ => panic!("Expected ProviderError wrapping ParseError"),
    }
}

#[tokio::test]
async fn test_tavily_empty_results() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "answer": null,
            "query": "very obscure query",
            "response_time": 0.5,
            "images": [],
            "results": [],
            "follow_up_questions": []
        })))
        .mount(&mock_server)
        .await;

    let provider = TavilyProvider::new("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")
        .unwrap()
        .with_base_url(&mock_server.uri());

    let options = create_test_options_with_provider(provider, "very obscure query");
    let results = web_search(options).await.unwrap();

    assert_eq!(results.len(), 0);
}

#[tokio::test]
async fn test_tavily_max_results_limit() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("POST"))
        .and(path("/"))
        .and(body_partial_json(json!({
            "max_results": 50  // Should be capped at 50
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_successful_tavily_response()))
        .mount(&mock_server)
        .await;

    let provider = TavilyProvider::new("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")
        .unwrap()
        .with_base_url(&mock_server.uri());

    let mut options = create_test_options_with_provider(provider, "test query");
    options.max_results = Some(100); // Request more than Tavily's limit

    let _results = web_search(options).await.unwrap();
    // The mock verifies that max_results was capped at 50
}

#[tokio::test]
async fn test_tavily_unicode_and_special_characters() {
    let mock_server = setup_mock_server().await;

    let unicode_response = json!({
        "answer": "Unicode search results",
        "query": "🔍 поиск 中文搜索",
        "response_time": 1.1,
        "images": [],
        "results": [
            {
                "title": "Unicode Search Results 🌐",
                "url": "https://example.com/unicode",
                "content": "Supporting unicode characters in search: 中文, العربية, русский, 🔍",
                "score": 0.9,
                "published_date": null
            }
        ],
        "follow_up_questions": []
    });

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(unicode_response))
        .mount(&mock_server)
        .await;

    let provider = TavilyProvider::new("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")
        .unwrap()
        .with_base_url(&mock_server.uri());

    let options = create_test_options_with_provider(provider, "🔍 поиск 中文搜索");
    let results = web_search(options).await.unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Unicode Search Results 🌐");
    assert!(results[0].snippet.as_ref().unwrap().contains("中文"));
}

#[tokio::test]
async fn test_tavily_request_timeout() {
    let mock_server = setup_mock_server().await;

    // Setup a mock that responds with delay
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(create_successful_tavily_response())
                .set_delay(Duration::from_secs(10)),
        ) // 10 second delay
        .mount(&mock_server)
        .await;

    let provider = TavilyProvider::new("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")
        .unwrap()
        .with_base_url(&mock_server.uri());

    let mut options = create_test_options_with_provider(provider, "test query");
    options.timeout = Some(1000); // 1 second timeout

    let start_time = std::time::Instant::now();
    let result = web_search(options).await;
    let elapsed = start_time.elapsed();

    // Should timeout quickly, not wait for the full 10 seconds
    assert!(elapsed < Duration::from_secs(5));
    assert!(result.is_err());
}

#[tokio::test]
async fn test_tavily_network_error() {
    // Test with invalid URL to simulate network error
    let provider = TavilyProvider::new("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")
        .unwrap()
        .with_base_url("http://invalid-url-that-does-not-exist.invalid");

    let options = create_test_options_with_provider(provider, "test query");
    let result = web_search(options).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        SearchError::ProviderError(msg) => {
            assert!(msg.contains("HTTP request failed"));
        }
        _ => panic!("Expected ProviderError wrapping HttpError"),
    }
}

#[tokio::test]
async fn test_tavily_provider_configuration_persistence() {
    let provider = TavilyProvider::new("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")
        .unwrap()
        .with_answer(false)
        .with_images(true)
        .with_search_depth("advanced")
        .unwrap();

    let config = provider.config();

    assert_eq!(config.get("include_answer"), Some(&"false".to_string()));
    assert_eq!(config.get("include_images"), Some(&"true".to_string()));
    assert_eq!(config.get("search_depth"), Some(&"advanced".to_string()));
    assert_eq!(config.get("provider"), Some(&"tavily".to_string()));
}

#[test]
fn test_tavily_api_key_format_validation() {
    // Test various invalid API key formats
    let invalid_keys = vec![
        "",
        "invalid",
        "api-key-without-prefix",
        "tavily-wrong-prefix",
        "TVLY-uppercase",
    ];

    for key in invalid_keys {
        let result = TavilyProvider::new(key);
        assert!(result.is_err(), "Should reject invalid key: {key}");
    }

    // Test valid format
    let valid_result = TavilyProvider::new("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");
    assert!(valid_result.is_ok());
}
