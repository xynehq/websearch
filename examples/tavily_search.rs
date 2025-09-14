//! Tavily search example demonstrating AI-powered search capabilities

use std::env;
use websearch::{
    providers::TavilyProvider,
    types::{DebugOptions, SearchOptions, SearchProvider},
    web_search,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    println!("🔍 Tavily Search SDK Example");
    println!("============================\n");

    // Get API key from environment variable
    let api_key = env::var("TAVILY_API_KEY")
        .expect("TAVILY_API_KEY environment variable is required. Set it with: export TAVILY_API_KEY=tvly-dev-your-api-key");

    // Example 1: Basic search
    println!("📋 Example 1: Basic Tavily Search");
    println!("----------------------------------");

    let basic_provider = TavilyProvider::new(&api_key)?;
    let basic_options = SearchOptions {
        query: "latest developments in AI and machine learning 2024".to_string(),
        max_results: Some(3),
        debug: Some(DebugOptions {
            enabled: true,
            log_requests: false,
            log_responses: false,
        }),
        provider: Box::new(basic_provider),
        ..Default::default()
    };

    match web_search(basic_options).await {
        Ok(results) => {
            println!("✅ Found {} results:", results.len());
            for (i, result) in results.iter().enumerate() {
                println!("\n{}. {}", i + 1, result.title);
                println!("   🔗 {}", result.url);
                if let Some(domain) = &result.domain {
                    println!("   🌐 Domain: {domain}");
                }
                if let Some(snippet) = &result.snippet {
                    let truncated = if snippet.len() > 150 {
                        format!("{}...", &snippet[..150])
                    } else {
                        snippet.clone()
                    };
                    println!("   📄 {truncated}");
                }
                if let Some(published_date) = &result.published_date {
                    println!("   📅 Published: {published_date}");
                }

                // Show Tavily-specific data from raw response
                if let Some(raw) = &result.raw {
                    if let Some(score) = raw.get("score") {
                        println!("   ⭐ Relevance Score: {score}");
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Basic search failed: {e}");
        }
    }

    println!("\n{}\n", "=".repeat(50));

    // Example 2: Advanced search with more features
    println!("📋 Example 2: Advanced Tavily Search");
    println!("------------------------------------");

    let advanced_provider = TavilyProvider::new_advanced(&api_key)?
        .with_answer(true)
        .with_images(false);

    let advanced_options = SearchOptions {
        query: "Rust programming language memory safety features".to_string(),
        max_results: Some(5),
        debug: Some(DebugOptions {
            enabled: true,
            log_requests: false,
            log_responses: false,
        }),
        provider: Box::new(advanced_provider),
        ..Default::default()
    };

    match web_search(advanced_options).await {
        Ok(results) => {
            println!("✅ Advanced search found {} results:", results.len());
            for (i, result) in results.iter().enumerate() {
                println!("\n{}. {}", i + 1, result.title);
                println!("   🔗 {}", result.url);
                if let Some(snippet) = &result.snippet {
                    let truncated = if snippet.len() > 200 {
                        format!("{}...", &snippet[..200])
                    } else {
                        snippet.clone()
                    };
                    println!("   📄 {truncated}");
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Advanced search failed: {e}");
        }
    }

    println!("\n{}\n", "=".repeat(50));

    // Example 3: Configuration demonstration
    println!("📋 Example 3: Provider Configuration");
    println!("------------------------------------");

    let configured_provider = TavilyProvider::new(&api_key)?
        .with_search_depth("basic")?
        .with_answer(false)
        .with_images(true);

    let config = configured_provider.config();
    println!("🔧 Provider Configuration:");
    for (key, value) in &config {
        println!("   • {key}: {value}");
    }

    println!("\n{}\n", "=".repeat(50));

    // Example 4: Error handling demonstration
    println!("📋 Example 4: Error Handling");
    println!("----------------------------");

    // Try with invalid API key to demonstrate error handling
    match TavilyProvider::new("invalid-key") {
        Ok(_) => println!("❌ Should have failed with invalid key"),
        Err(e) => println!("✅ Correctly caught configuration error: {e}"),
    }

    // Try with empty query
    let error_provider = TavilyProvider::new(&api_key)?;
    let error_options = SearchOptions {
        query: "".to_string(),
        provider: Box::new(error_provider),
        ..Default::default()
    };

    match web_search(error_options).await {
        Ok(_) => println!("❌ Should have failed with empty query"),
        Err(e) => println!("✅ Correctly caught input validation error: {e}"),
    }

    println!("\n🎉 Tavily Search SDK examples completed!");
    println!("\n💡 Tips:");
    println!("  • Set TAVILY_API_KEY environment variable with your real API key");
    println!("  • Tavily is optimized for AI/LLM applications with high-quality results");
    println!("  • Use 'advanced' search depth for more comprehensive results");
    println!("  • Enable answer generation for direct answers to questions");
    println!("  • Monitor your API usage to stay within rate limits");

    Ok(())
}
