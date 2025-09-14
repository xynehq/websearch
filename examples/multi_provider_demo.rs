//! Multi-provider demo showing the benefits of using multiple search engines

use std::env;
use tokio::time::{Duration, Instant};
use websearch::{
    multi_provider::{
        MultiProviderConfig, MultiProviderSearch, MultiProviderStrategy, SearchOptionsMulti,
    },
    providers::{ExaProvider, GoogleProvider},
    types::DebugOptions,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("🔍 Multi-Provider Search Benefits Demo");
    println!("======================================\n");

    // Get API credentials
    let google_api_key = env::var("GOOGLE_API_KEY").ok();
    let google_cx = env::var("GOOGLE_CX").ok();
    let exa_api_key = env::var("EXA_API_KEY").ok();

    // Demo 1: Speed Comparison - Race Strategy
    println!("🏁 Demo 1: Speed Comparison (Race Strategy)");
    println!("===========================================");

    if let (Some(google_key), Some(cx), Some(exa_key)) = (
        google_api_key.as_ref(),
        google_cx.as_ref(),
        exa_api_key.as_ref(),
    ) {
        let mut config = MultiProviderConfig::new(MultiProviderStrategy::RaceFirst)
            .with_timeout(Duration::from_secs(10));

        // Add both providers
        config = config.add_provider(Box::new(GoogleProvider::new(google_key, cx)?));
        config = config.add_provider(Box::new(ExaProvider::new(exa_key)?));

        let mut multi_search = MultiProviderSearch::new(config);

        let start = Instant::now();
        let results = multi_search
            .search(&SearchOptionsMulti {
                query: "artificial intelligence 2024".to_string(),
                max_results: Some(3),
                debug: Some(DebugOptions {
                    enabled: true,
                    log_requests: false,
                    log_responses: true,
                }),
                ..Default::default()
            })
            .await?;

        let duration = start.elapsed();
        println!("⚡ Race completed in: {duration:?}");
        println!(
            "🏆 Winner: {} (returned {} results)",
            results
                .first()
                .and_then(|r| r.provider.as_ref())
                .unwrap_or(&"unknown".to_string()),
            results.len()
        );

        for (i, result) in results.iter().take(2).enumerate() {
            println!(
                "  {}. {} ({})",
                i + 1,
                result.title,
                result.provider.as_ref().unwrap_or(&"unknown".to_string())
            );
        }
    }

    println!("\n{}\n", "=".repeat(60));

    // Demo 2: Quality Comparison - Aggregate Strategy
    println!("📊 Demo 2: Quality Comparison (Aggregate Strategy)");
    println!("=================================================");

    if let (Some(google_key), Some(cx), Some(exa_key)) = (
        google_api_key.as_ref(),
        google_cx.as_ref(),
        exa_api_key.as_ref(),
    ) {
        let mut config = MultiProviderConfig::new(MultiProviderStrategy::Aggregate)
            .with_timeout(Duration::from_secs(10))
            .with_max_concurrent(2);

        config = config.add_provider(Box::new(GoogleProvider::new(google_key, cx)?));
        config = config.add_provider(Box::new(ExaProvider::new(exa_key)?));

        let mut multi_search = MultiProviderSearch::new(config);

        let results = multi_search
            .search(&SearchOptionsMulti {
                query: "machine learning best practices".to_string(),
                max_results: Some(5),
                debug: Some(DebugOptions {
                    enabled: true,
                    log_requests: false,
                    log_responses: false,
                }),
                ..Default::default()
            })
            .await?;

        println!("🔍 Total aggregated results: {}", results.len());

        // Analyze results by provider
        let mut provider_counts = std::collections::HashMap::new();
        let mut unique_domains = std::collections::HashSet::new();

        for result in &results {
            if let Some(provider) = &result.provider {
                *provider_counts.entry(provider.clone()).or_insert(0) += 1;
            }
            if let Some(domain) = &result.domain {
                unique_domains.insert(domain.clone());
            }
        }

        println!("📈 Results by provider:");
        for (provider, count) in &provider_counts {
            println!("  • {provider}: {count} results");
        }
        println!("🌐 Unique domains found: {}", unique_domains.len());

        println!("\n🎯 Sample aggregated results:");
        for (i, result) in results.iter().take(4).enumerate() {
            println!(
                "  {}. {} ({})",
                i + 1,
                result.title,
                result.provider.as_ref().unwrap_or(&"unknown".to_string())
            );
            if let Some(domain) = &result.domain {
                println!("     🔗 {domain}");
            }
        }
    }

    println!("\n{}\n", "=".repeat(60));

    // Demo 3: Reliability - Failover Strategy
    println!("🛡️  Demo 3: Reliability (Failover Strategy)");
    println!("===========================================");

    if let (Some(google_key), Some(cx), Some(exa_key)) = (
        google_api_key.as_ref(),
        google_cx.as_ref(),
        exa_api_key.as_ref(),
    ) {
        let mut config = MultiProviderConfig::new(MultiProviderStrategy::Failover)
            .with_timeout(Duration::from_secs(5));

        // Add providers in priority order
        config = config.add_provider(Box::new(GoogleProvider::new(google_key, cx)?));
        config = config.add_provider(Box::new(ExaProvider::new(exa_key)?));

        let mut multi_search = MultiProviderSearch::new(config);

        // Test failover with a few searches
        for i in 1..=3 {
            println!("\n🔄 Search attempt {i}");
            let results = multi_search
                .search(&SearchOptionsMulti {
                    query: format!("software engineering practices {i}"),
                    max_results: Some(2),
                    debug: Some(DebugOptions {
                        enabled: true,
                        log_requests: false,
                        log_responses: false,
                    }),
                    ..Default::default()
                })
                .await?;

            if !results.is_empty() {
                println!(
                    "✅ Success with: {}",
                    results
                        .first()
                        .and_then(|r| r.provider.as_ref())
                        .unwrap_or(&"unknown".to_string())
                );
            }
        }

        // Show provider statistics
        println!("\n📊 Provider Performance Stats:");
        for (name, stats) in multi_search.get_stats() {
            println!(
                "  • {}: {} total, {} successful, {} failed, {:.1}ms avg",
                name,
                stats.total_requests,
                stats.successful_requests,
                stats.failed_requests,
                stats.avg_response_time_ms
            );
        }
    }

    println!("\n{}\n", "=".repeat(60));

    // Demo 4: Load Balancing
    println!("⚖️  Demo 4: Load Balancing Strategy");
    println!("==================================");

    if let (Some(google_key), Some(cx), Some(exa_key)) = (
        google_api_key.as_ref(),
        google_cx.as_ref(),
        exa_api_key.as_ref(),
    ) {
        let mut config = MultiProviderConfig::new(MultiProviderStrategy::LoadBalance)
            .with_timeout(Duration::from_secs(8));

        config = config.add_provider(Box::new(GoogleProvider::new(google_key, cx)?));
        config = config.add_provider(Box::new(ExaProvider::new(exa_key)?));

        let mut multi_search = MultiProviderSearch::new(config);

        println!("🔄 Making multiple requests to demonstrate load balancing:");

        for i in 1..=4 {
            let results = multi_search
                .search(&SearchOptionsMulti {
                    query: format!("programming tutorial {i}"),
                    max_results: Some(1),
                    debug: Some(DebugOptions {
                        enabled: false,
                        log_requests: false,
                        log_responses: false,
                    }),
                    ..Default::default()
                })
                .await?;

            if let Some(result) = results.first() {
                println!(
                    "  Request {}: {} handled search",
                    i,
                    result.provider.as_ref().unwrap_or(&"unknown".to_string())
                );
            }
        }

        println!("\n📈 Final load balancing stats:");
        for (name, stats) in multi_search.get_stats() {
            println!(
                "  • {}: {} requests ({:.1}% of total)",
                name,
                stats.total_requests,
                (stats.total_requests as f64 / 4.0) * 100.0
            );
        }
    }

    println!("\n🎉 Multi-Provider Demo Completed!");
    println!("\n💡 Key Benefits Demonstrated:");
    println!("  • ⚡ Speed: Race strategy gets fastest results");
    println!("  • 📊 Quality: Aggregate strategy provides broader coverage");
    println!("  • 🛡️  Reliability: Failover ensures availability");
    println!("  • ⚖️  Efficiency: Load balancing distributes usage");
    println!("  • 💰 Cost: Distribute API calls across multiple quotas");

    Ok(())
}
