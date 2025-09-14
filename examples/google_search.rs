//! Google Custom Search API example demonstrating various search capabilities

use std::env;
use websearch::{
    providers::GoogleProvider,
    types::{DebugOptions, SafeSearch, SearchOptions, SearchProvider},
    web_search,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    println!("🔍 Google Custom Search SDK Example");
    println!("===================================\n");

    // Get API credentials from environment
    let api_key =
        env::var("GOOGLE_API_KEY").map_err(|_| "GOOGLE_API_KEY environment variable not set")?;
    let cx = env::var("GOOGLE_CX").map_err(|_| "GOOGLE_CX environment variable not set")?;

    // Example 1: Basic Google search
    println!("📋 Example 1: Basic Google Search");
    println!("----------------------------------");

    let basic_provider = GoogleProvider::new(&api_key, &cx)?;
    let basic_options = SearchOptions {
        query: "Rust programming language 2024".to_string(),
        max_results: Some(5),
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
            }
        }
        Err(e) => {
            eprintln!("❌ Basic search failed: {e}");
        }
    }

    println!("\n{}\n", "=".repeat(50));

    // Example 2: Search with language and region
    println!("📋 Example 2: Google Search with Language & Region");
    println!("--------------------------------------------------");

    let regional_provider = GoogleProvider::new(&api_key, &cx)?;
    let regional_options = SearchOptions {
        query: "machine learning tutorials".to_string(),
        max_results: Some(3),
        language: Some("en".to_string()),
        region: Some("us".to_string()),
        debug: Some(DebugOptions {
            enabled: true,
            log_requests: false,
            log_responses: false,
        }),
        provider: Box::new(regional_provider),
        ..Default::default()
    };

    match web_search(regional_options).await {
        Ok(results) => {
            println!("✅ Regional search found {} results:", results.len());
            for (i, result) in results.iter().enumerate() {
                println!("\n{}. {}", i + 1, result.title);
                println!("   🔗 {}", result.url);
                if let Some(snippet) = &result.snippet {
                    let truncated = if snippet.len() > 120 {
                        format!("{}...", &snippet[..120])
                    } else {
                        snippet.clone()
                    };
                    println!("   📄 {truncated}");
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Regional search failed: {e}");
        }
    }

    println!("\n{}\n", "=".repeat(50));

    // Example 3: Search with safe search enabled
    println!("📋 Example 3: Google Search with Safe Search");
    println!("--------------------------------------------");

    let safe_provider = GoogleProvider::new(&api_key, &cx)?;
    let safe_options = SearchOptions {
        query: "artificial intelligence ethics".to_string(),
        max_results: Some(4),
        safe_search: Some(SafeSearch::Strict),
        debug: Some(DebugOptions {
            enabled: true,
            log_requests: false,
            log_responses: false,
        }),
        provider: Box::new(safe_provider),
        ..Default::default()
    };

    match web_search(safe_options).await {
        Ok(results) => {
            println!("✅ Safe search found {} results:", results.len());
            for (i, result) in results.iter().enumerate() {
                println!("\n{}. {}", i + 1, result.title);
                println!("   🔗 {}", result.url);
                if let Some(domain) = &result.domain {
                    println!("   🌐 Domain: {domain}");
                }
                if let Some(snippet) = &result.snippet {
                    let truncated = if snippet.len() > 100 {
                        format!("{}...", &snippet[..100])
                    } else {
                        snippet.clone()
                    };
                    println!("   📄 {truncated}");
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Safe search failed: {e}");
        }
    }

    println!("\n{}\n", "=".repeat(50));

    // Example 4: Paginated search
    println!("📋 Example 4: Google Search with Pagination");
    println!("--------------------------------------------");

    let paginated_provider = GoogleProvider::new(&api_key, &cx)?;
    let paginated_options = SearchOptions {
        query: "web development frameworks".to_string(),
        max_results: Some(3),
        page: Some(2), // Get second page
        debug: Some(DebugOptions {
            enabled: true,
            log_requests: false,
            log_responses: false,
        }),
        provider: Box::new(paginated_provider),
        ..Default::default()
    };

    match web_search(paginated_options).await {
        Ok(results) => {
            println!(
                "✅ Paginated search (page 2) found {} results:",
                results.len()
            );
            for (i, result) in results.iter().enumerate() {
                println!("\n{}. {}", i + 1, result.title);
                println!("   🔗 {}", result.url);
                if let Some(domain) = &result.domain {
                    println!("   🌐 Domain: {domain}");
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Paginated search failed: {e}");
        }
    }

    println!("\n{}\n", "=".repeat(50));

    // Example 5: Provider configuration demonstration
    println!("📋 Example 5: Provider Configuration");
    println!("------------------------------------");

    let configured_provider = GoogleProvider::new(&api_key, &cx)?;
    let config = configured_provider.config();
    println!("🔧 Provider Configuration:");
    for (key, value) in &config {
        println!("   • {key}: {value}");
    }

    println!("\n{}\n", "=".repeat(50));

    // Example 6: Error handling demonstration
    println!("📋 Example 6: Error Handling");
    println!("----------------------------");

    // Try with invalid API key
    match GoogleProvider::new("invalid-key", &cx) {
        Ok(_) => println!("❌ Should have failed with invalid key"),
        Err(e) => println!("✅ Correctly caught configuration error: {e}"),
    }

    // Try with invalid CX
    match GoogleProvider::new(&api_key, "invalid-cx") {
        Ok(_) => println!("❌ Should have failed with invalid CX"),
        Err(e) => println!("✅ Correctly caught configuration error: {e}"),
    }

    // Try with empty query
    let error_provider = GoogleProvider::new(&api_key, &cx)?;
    let error_options = SearchOptions {
        query: "".to_string(),
        provider: Box::new(error_provider),
        ..Default::default()
    };

    match web_search(error_options).await {
        Ok(_) => println!("❌ Should have failed with empty query"),
        Err(e) => println!("✅ Correctly caught input validation error: {e}"),
    }

    println!("\n🎉 Google Custom Search SDK examples completed!");
    println!("\n💡 Tips:");
    println!("  • Google Custom Search API has a daily limit of 100 free queries");
    println!("  • Results are limited to 10 per request (max_results capped at 10)");
    println!("  • Use pagination to get more results");
    println!("  • Create custom search engines at https://cse.google.com/cse/");
    println!("  • Monitor your API usage in Google Cloud Console");

    Ok(())
}
