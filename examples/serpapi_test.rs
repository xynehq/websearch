//! SERP API test example

use websearch::{providers::SerpApiProvider, types::DebugOptions, web_search, SearchOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    // Get API key from environment
    let api_key = std::env::var("SERPAPI_API_KEY")
        .or_else(|_| std::env::var("SERP_API_KEY"))
        .expect("SERPAPI_API_KEY environment variable is required");

    println!("🔍 Testing SERP API with key: {}...", &api_key[..8]);

    // Create SERP API provider
    let serpapi = SerpApiProvider::new(&api_key)?;

    // Test search
    let results = web_search(SearchOptions {
        query: "rust programming language".to_string(),
        max_results: Some(5),
        provider: Box::new(serpapi),
        debug: Some(DebugOptions {
            enabled: true,
            log_requests: true,
            log_responses: true,
        }),
        ..Default::default()
    })
    .await?;

    println!("✅ Found {} results:", results.len());
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

    Ok(())
}
