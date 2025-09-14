//! Multi-provider search example demonstrating load balancing and failover

use tokio::time::Duration;
use websearch::{
    multi_provider::{
        MultiProviderConfig, MultiProviderSearch, MultiProviderStrategy, SearchOptionsMulti,
    },
    providers::{DuckDuckGoProvider, GoogleProvider},
    types::DebugOptions,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("🔍 Multi-Provider Search Examples\n");

    // Example 1: Failover Strategy
    println!("=== Failover Strategy ===");
    let mut failover_search = create_multi_provider_search(MultiProviderStrategy::Failover).await?;

    let results = failover_search
        .search(&SearchOptionsMulti {
            query: "Rust async programming".to_string(),
            max_results: Some(3),
            debug: Some(DebugOptions {
                enabled: true,
                log_requests: false,
                log_responses: true,
            }),
            ..Default::default()
        })
        .await?;

    println!("Failover results: {} found", results.len());
    for (i, result) in results.iter().enumerate() {
        println!(
            "{}. {} ({})",
            i + 1,
            result.title,
            result.provider.as_ref().unwrap_or(&"unknown".to_string())
        );
    }
    println!();

    // Example 2: Load Balance Strategy
    println!("=== Load Balance Strategy ===");
    let mut lb_search = create_multi_provider_search(MultiProviderStrategy::LoadBalance).await?;

    // Make multiple requests to see load balancing
    for i in 1..=3 {
        println!("Request {i}");
        let results = lb_search
            .search(&SearchOptionsMulti {
                query: format!("Rust web framework {i}"),
                max_results: Some(2),
                debug: Some(DebugOptions {
                    enabled: true,
                    log_requests: false,
                    log_responses: false,
                }),
                ..Default::default()
            })
            .await?;

        println!(
            "  Found {} results from {}",
            results.len(),
            results
                .first()
                .and_then(|r| r.provider.as_ref())
                .unwrap_or(&"unknown".to_string())
        );
    }
    println!();

    // Example 3: Aggregate Strategy
    println!("=== Aggregate Strategy ===");
    let mut agg_search = create_multi_provider_search(MultiProviderStrategy::Aggregate).await?;

    let results = agg_search
        .search(&SearchOptionsMulti {
            query: "Rust performance optimization".to_string(),
            max_results: Some(8), // Will get results from multiple providers
            debug: Some(DebugOptions {
                enabled: true,
                log_requests: false,
                log_responses: true,
            }),
            ..Default::default()
        })
        .await?;

    println!("Aggregated results: {} found", results.len());
    let mut provider_counts = std::collections::HashMap::new();
    for result in &results {
        if let Some(provider) = &result.provider {
            *provider_counts.entry(provider.clone()).or_insert(0) += 1;
        }
    }
    for (provider, count) in provider_counts {
        println!("  {provider}: {count} results");
    }
    println!();

    // Example 4: Race Strategy
    println!("=== Race First Strategy ===");
    let mut race_search = create_multi_provider_search(MultiProviderStrategy::RaceFirst).await?;

    let results = race_search
        .search(&SearchOptionsMulti {
            query: "Rust memory safety".to_string(),
            max_results: Some(3),
            debug: Some(DebugOptions {
                enabled: true,
                log_requests: false,
                log_responses: true,
            }),
            ..Default::default()
        })
        .await?;

    println!(
        "Race winner results: {} found from {}",
        results.len(),
        results
            .first()
            .and_then(|r| r.provider.as_ref())
            .unwrap_or(&"unknown".to_string())
    );
    println!();

    // Show provider statistics
    println!("=== Provider Statistics ===");
    for (name, stats) in lb_search.get_stats() {
        println!(
            "{}: {} total, {} successful, {} failed, {:.1}ms avg",
            name,
            stats.total_requests,
            stats.successful_requests,
            stats.failed_requests,
            stats.avg_response_time_ms
        );
    }

    Ok(())
}

async fn create_multi_provider_search(
    strategy: MultiProviderStrategy,
) -> Result<MultiProviderSearch, Box<dyn std::error::Error>> {
    let mut config = MultiProviderConfig::new(strategy)
        .with_timeout(Duration::from_secs(5))
        .with_max_concurrent(2);

    // Add DuckDuckGo (always works, no API key needed)
    config = config.add_provider(Box::new(DuckDuckGoProvider::new()));

    // Add Google if API keys are available
    if let (Ok(api_key), Ok(cx)) = (std::env::var("GOOGLE_API_KEY"), std::env::var("GOOGLE_CX")) {
        config = config.add_provider(Box::new(GoogleProvider::new(&api_key, &cx)?));
        println!("✓ Added Google provider");
    } else {
        println!("⚠ Google API credentials not found, using DuckDuckGo only");
    }

    Ok(MultiProviderSearch::new(config))
}
