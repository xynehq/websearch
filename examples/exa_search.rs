//! Exa search example demonstrating AI-powered semantic search capabilities

use std::env;
use websearch::{
    providers::ExaProvider,
    types::{DebugOptions, SearchOptions, SearchProvider},
    web_search,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    println!("🔍 Exa Search SDK Example");
    println!("=========================\n");

    // Get API key from environment
    let api_key =
        env::var("EXA_API_KEY").map_err(|_| "EXA_API_KEY environment variable not set")?;

    // Example 1: Basic keyword search
    println!("📋 Example 1: Basic Exa Keyword Search");
    println!("--------------------------------------");

    let basic_provider = ExaProvider::new(&api_key)?;
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
                // Check for relevance score in raw data
                if let Some(raw) = &result.raw {
                    if let Some(score) = raw.get("relevance_score") {
                        if let Some(score_num) = score.as_f64() {
                            println!("   ⭐ Relevance Score: {score_num:.3}");
                        }
                    }
                }

                // Show Exa-specific data from raw response
                if let Some(raw) = &result.raw {
                    if let Some(author) = raw.get("author") {
                        println!("   ✍️ Author: {author}");
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Basic search failed: {e}");
        }
    }

    println!("\n{}\n", "=".repeat(50));

    // Example 2: Embeddings-based search
    println!("📋 Example 2: Exa Embeddings Search");
    println!("-----------------------------------");

    let embeddings_provider = ExaProvider::new(&api_key)?
        .with_model("embeddings")?
        .with_contents(true);

    let embeddings_options = SearchOptions {
        query: "Rust programming language memory safety features".to_string(),
        max_results: Some(4),
        debug: Some(DebugOptions {
            enabled: true,
            log_requests: false,
            log_responses: false,
        }),
        provider: Box::new(embeddings_provider),
        ..Default::default()
    };

    match web_search(embeddings_options).await {
        Ok(results) => {
            println!("✅ Embeddings search found {} results:", results.len());
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
                // Check for relevance score in raw data
                if let Some(raw) = &result.raw {
                    if let Some(score) = raw.get("relevance_score") {
                        if let Some(score_num) = score.as_f64() {
                            println!("   ⭐ Relevance Score: {score_num:.3}");
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Embeddings search failed: {e}");
        }
    }

    println!("\n{}\n", "=".repeat(50));

    // Example 3: Advanced search with full content
    println!("📋 Example 3: Exa Advanced Search with Content");
    println!("----------------------------------------------");

    let advanced_provider = ExaProvider::new_advanced(&api_key)?.with_model("keyword")?;

    let advanced_options = SearchOptions {
        query: "quantum computing breakthroughs".to_string(),
        max_results: Some(2),
        debug: Some(DebugOptions {
            enabled: true,
            log_requests: false,
            log_responses: true,
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
                if let Some(domain) = &result.domain {
                    println!("   🌐 Domain: {domain}");
                }
                if let Some(snippet) = &result.snippet {
                    let content_preview = if snippet.len() > 300 {
                        format!("{}...", &snippet[..300])
                    } else {
                        snippet.clone()
                    };
                    println!("   📄 Content: {content_preview}");
                }
                // Check for relevance score in raw data
                if let Some(raw) = &result.raw {
                    if let Some(score) = raw.get("relevance_score") {
                        if let Some(score_num) = score.as_f64() {
                            println!("   ⭐ Relevance Score: {score_num:.3}");
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Advanced search failed: {e}");
        }
    }

    println!("\n{}\n", "=".repeat(50));

    // Example 4: Provider configuration demonstration
    println!("📋 Example 4: Provider Configuration");
    println!("------------------------------------");

    let configured_provider = ExaProvider::new(&api_key)?
        .with_model("embeddings")?
        .with_contents(true);

    let config = configured_provider.config();
    println!("🔧 Provider Configuration:");
    for (key, value) in &config {
        println!("   • {key}: {value}");
    }

    println!("\n{}\n", "=".repeat(50));

    // Example 5: Error handling demonstration
    println!("📋 Example 5: Error Handling");
    println!("----------------------------");

    // Try with invalid API key to demonstrate error handling
    match ExaProvider::new("invalid-key") {
        Ok(_) => println!("❌ Should have failed with invalid key"),
        Err(e) => println!("✅ Correctly caught configuration error: {e}"),
    }

    // Try with invalid model
    match ExaProvider::new(&api_key)?.with_model("invalid-model") {
        Ok(_) => println!("❌ Should have failed with invalid model"),
        Err(e) => println!("✅ Correctly caught model validation error: {e}"),
    }

    // Try with empty query
    let error_provider = ExaProvider::new(&api_key)?;
    let error_options = SearchOptions {
        query: "".to_string(),
        provider: Box::new(error_provider),
        ..Default::default()
    };

    match web_search(error_options).await {
        Ok(_) => println!("❌ Should have failed with empty query"),
        Err(e) => println!("✅ Correctly caught input validation error: {e}"),
    }

    println!("\n🎉 Exa Search SDK examples completed!");
    println!("\n💡 Tips:");
    println!("  • Use 'keyword' model for traditional search, 'embeddings' for semantic search");
    println!("  • Enable content extraction for more detailed results");
    println!("  • Exa provides high-quality, AI-curated search results");
    println!("  • Check relevance scores to assess result quality");
    println!("  • Monitor your API usage to stay within rate limits");

    Ok(())
}
