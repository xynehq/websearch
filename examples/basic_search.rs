//! Basic search example using the search SDK

use websearch::{providers::GoogleProvider, types::DebugOptions, web_search, SearchOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    // Example with Google (requires API key and CX)
    if let (Ok(api_key), Ok(cx)) = (std::env::var("GOOGLE_API_KEY"), std::env::var("GOOGLE_CX")) {
        println!("🔍 Testing Google search...");

        let google = GoogleProvider::new(&api_key, &cx)?;

        let results = web_search(SearchOptions {
            query: "Rust programming language".to_string(),
            max_results: Some(5),
            provider: Box::new(google),
            debug: Some(DebugOptions {
                enabled: true,
                log_requests: true,
                log_responses: true,
            }),
            ..Default::default()
        })
        .await?;

        println!("Found {} results:", results.len());
        for (i, result) in results.iter().enumerate() {
            println!("{}. {}", i + 1, result.title);
            println!("   URL: {}", result.url);
            if let Some(snippet) = &result.snippet {
                println!("   {snippet}");
            }
            if let Some(domain) = &result.domain {
                println!("   Domain: {domain}");
            }
            println!();
        }
    } else {
        println!("⚠️ Google API credentials not found. Set GOOGLE_API_KEY and GOOGLE_CX environment variables.");
    }

    // Example with DuckDuckGo (no API key required)
    println!("🦆 Testing DuckDuckGo search...");

    let duckduckgo = websearch::providers::DuckDuckGoProvider::new();

    let results = web_search(SearchOptions {
        query: "Rust programming".to_string(),
        max_results: Some(3),
        provider: Box::new(duckduckgo),
        debug: Some(DebugOptions {
            enabled: true,
            log_requests: false,
            log_responses: true,
        }),
        ..Default::default()
    })
    .await?;

    println!("Found {} results:", results.len());
    for (i, result) in results.iter().enumerate() {
        println!("{}. {}", i + 1, result.title);
        println!("   URL: {}", result.url);
        if let Some(snippet) = &result.snippet {
            println!("   {snippet}");
        }
        println!();
    }

    Ok(())
}
