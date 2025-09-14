//! Simple example showing how another project would use this search SDK

use std::env;
use websearch::{
    multi_provider::{
        MultiProviderConfig, MultiProviderSearch, MultiProviderStrategy, SearchOptionsMulti,
    },
    providers::{ExaProvider, GoogleProvider},
    types::{DebugOptions, SearchProvider},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Simple Usage Example - How Other Projects Use This SDK");
    println!("=========================================================\n");

    // This is how another project would use the search SDK

    // Option 1: Single Provider (Simple)
    println!("📋 Option 1: Single Provider Usage");
    println!("----------------------------------");

    if let Ok(exa_key) = env::var("EXA_API_KEY") {
        let exa_provider = ExaProvider::new(&exa_key)?;

        // This is the main interface other projects would use
        let results = exa_provider
            .search(&websearch::types::SearchOptions {
                query: "Rust web frameworks".to_string(),
                max_results: Some(3),
                debug: Some(DebugOptions {
                    enabled: false,
                    log_requests: false,
                    log_responses: false,
                }),
                provider: Box::new(ExaProvider::new(&exa_key)?), // Required by the trait
                ..Default::default()
            })
            .await?;

        println!("✅ Found {} results using Exa:", results.len());
        for (i, result) in results.iter().enumerate() {
            println!("  {}. {}", i + 1, result.title);
            println!("     🔗 {}", result.url);
        }
    }

    println!("\n{}\n", "=".repeat(50));

    // Option 2: Multi-Provider (Advanced)
    println!("📋 Option 2: Multi-Provider Usage (Recommended)");
    println!("-----------------------------------------------");

    let mut config = MultiProviderConfig::new(MultiProviderStrategy::Aggregate);

    // Add available providers
    if let (Ok(google_key), Ok(cx)) = (env::var("GOOGLE_API_KEY"), env::var("GOOGLE_CX")) {
        config = config.add_provider(Box::new(GoogleProvider::new(&google_key, &cx)?));
        println!("✅ Added Google provider");
    }

    if let Ok(exa_key) = env::var("EXA_API_KEY") {
        config = config.add_provider(Box::new(ExaProvider::new(&exa_key)?));
        println!("✅ Added Exa provider");
    }

    let mut multi_search = MultiProviderSearch::new(config);

    let results = multi_search
        .search(&SearchOptionsMulti {
            query: "artificial intelligence trends 2024".to_string(),
            max_results: Some(5),
            debug: Some(DebugOptions {
                enabled: false,
                log_requests: false,
                log_responses: false,
            }),
            ..Default::default()
        })
        .await?;

    println!(
        "\n🔍 Multi-provider search found {} unique results:",
        results.len()
    );

    // Group by provider to show diversity
    let mut google_count = 0;
    let mut exa_count = 0;

    for result in &results {
        match result.provider.as_deref() {
            Some("google") => google_count += 1,
            Some("exa") => exa_count += 1,
            _ => {}
        }
    }

    println!("  • Google contributed: {google_count} results");
    println!("  • Exa contributed: {exa_count} results");

    println!("\n📋 Sample results:");
    for (i, result) in results.iter().take(3).enumerate() {
        println!(
            "  {}. {} ({})",
            i + 1,
            result.title,
            result.provider.as_ref().unwrap_or(&"unknown".to_string())
        );
        if let Some(domain) = &result.domain {
            println!("     🌐 {domain}");
        }
    }

    println!("\n💡 **How Other Projects Would Add This:**");
    println!("==========================================");
    println!();
    println!("1. Add to Cargo.toml:");
    println!("   [dependencies]");
    println!("   search-sdk-rust = \"0.0.1\"");
    println!("   tokio = {{ version = \"1.0\", features = [\"full\"] }}");
    println!();
    println!("2. Set environment variables:");
    println!("   export GOOGLE_API_KEY=\"your_key\"");
    println!("   export GOOGLE_CX=\"your_cx\"");
    println!("   export EXA_API_KEY=\"your_key\"");
    println!();
    println!("3. Use in code:");
    println!("   use websearch::{{providers::*, multi_provider::*}};");
    println!("   // Then use as shown in this example");
    println!();
    println!("🎯 **Benefits for other projects:**");
    println!("  • Multiple search providers in one library");
    println!("  • Smart aggregation to maximize unique results");
    println!("  • Automatic failover and load balancing");
    println!("  • Consistent API across all providers");
    println!("  • Built-in error handling and debugging");

    Ok(())
}
