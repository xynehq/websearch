//! Smart aggregation focused on finding unique links and avoiding quota waste

use std::{collections::HashSet, env};
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

    println!("🔍 Smart Aggregate Search - Maximize Unique Links");
    println!("=================================================\n");

    // Get API credentials
    let google_api_key =
        env::var("GOOGLE_API_KEY").map_err(|_| "GOOGLE_API_KEY environment variable not set")?;
    let google_cx = env::var("GOOGLE_CX").map_err(|_| "GOOGLE_CX environment variable not set")?;
    let exa_api_key =
        env::var("EXA_API_KEY").map_err(|_| "EXA_API_KEY environment variable not set")?;

    // Smart Aggregation Strategy - NO racing, focus on unique content
    let mut config =
        MultiProviderConfig::new(MultiProviderStrategy::Aggregate).with_max_concurrent(2); // Run both simultaneously but don't race

    config = config.add_provider(Box::new(GoogleProvider::new(&google_api_key, &google_cx)?));
    config = config.add_provider(Box::new(ExaProvider::new(&exa_api_key)?));

    let mut multi_search = MultiProviderSearch::new(config);

    // Test with a topic that should give different results from each engine
    let query = "machine learning frameworks comparison 2024";

    println!("📋 Query: {query}");
    println!("🎯 Strategy: Get unique links from both Google + Exa");
    println!("💡 Quota usage: 1 Google call + 1 Exa call = Maximum value\n");

    let results = multi_search
        .search(&SearchOptionsMulti {
            query: query.to_string(),
            max_results: Some(8), // Get more results to find more unique links
            debug: Some(DebugOptions {
                enabled: true,
                log_requests: false,
                log_responses: false,
            }),
            ..Default::default()
        })
        .await?;

    // Analyze the results
    println!("📊 **RESULTS ANALYSIS**");
    println!("======================");

    let mut google_results = Vec::new();
    let mut exa_results = Vec::new();
    let mut all_urls = HashSet::new();
    let mut all_domains = HashSet::new();

    for result in &results {
        all_urls.insert(result.url.clone());
        if let Some(domain) = &result.domain {
            all_domains.insert(domain.clone());
        }

        match result.provider.as_deref() {
            Some("google") => google_results.push(result),
            Some("exa") => exa_results.push(result),
            _ => {}
        }
    }

    println!("🔍 Total results: {}", results.len());
    println!("🔗 Unique URLs: {}", all_urls.len());
    println!("🌐 Unique domains: {}", all_domains.len());
    println!("📈 Google contributed: {} results", google_results.len());
    println!("📈 Exa contributed: {} results", exa_results.len());

    // Check for overlap
    let google_urls: HashSet<String> = google_results.iter().map(|r| r.url.clone()).collect();
    let exa_urls: HashSet<String> = exa_results.iter().map(|r| r.url.clone()).collect();
    let overlap = google_urls.intersection(&exa_urls).count();

    println!(
        "🔄 URL overlap: {} ({}% efficiency)",
        overlap,
        (((all_urls.len() as f64) / (results.len() as f64)) * 100.0) as u32
    );

    if overlap == 0 {
        println!("🎉 Perfect! Zero overlap = Maximum unique content discovery");
    } else {
        println!("✅ Good efficiency with minimal overlap");
    }

    println!("\n📋 **UNIQUE RESULTS BREAKDOWN**");
    println!("==============================");

    // Show Google-specific findings
    println!("\n🔵 **Google-specific discoveries:**");
    for (i, result) in google_results.iter().take(3).enumerate() {
        println!("  {}. {}", i + 1, result.title);
        println!("     🔗 {}", result.url);
        if let Some(domain) = &result.domain {
            println!("     🌐 {domain}");
        }
    }

    // Show Exa-specific findings
    println!("\n🟢 **Exa-specific discoveries:**");
    for (i, result) in exa_results.iter().take(3).enumerate() {
        println!("  {}. {}", i + 1, result.title);
        println!("     🔗 {}", result.url);
        if let Some(domain) = &result.domain {
            println!("     🌐 {domain}");
        }
    }

    // Show domain diversity
    println!("\n🌐 **Domain Diversity Analysis:**");
    let mut domain_counts = std::collections::HashMap::new();
    for result in &results {
        if let Some(domain) = &result.domain {
            *domain_counts.entry(domain.clone()).or_insert(0) += 1;
        }
    }

    println!(
        "   Found content from {} unique domains:",
        all_domains.len()
    );
    for (domain, count) in domain_counts.iter().take(8) {
        println!("   • {domain}: {count} result(s)");
    }

    // Show quota efficiency
    println!("\n💰 **Quota Efficiency Report:**");
    println!("=============================");
    println!("🔸 API calls made: 2 (1 Google + 1 Exa)");
    println!("🔸 Unique URLs discovered: {}", all_urls.len());
    println!("🔸 URLs per API call: {:.1}", all_urls.len() as f64 / 2.0);
    println!("🔸 Content diversity: {} unique domains", all_domains.len());

    if all_urls.len() >= results.len() * 90 / 100 {
        println!("🎯 **EXCELLENT**: >90% unique content - minimal waste");
    } else if all_urls.len() >= results.len() * 75 / 100 {
        println!("✅ **GOOD**: >75% unique content - efficient aggregation");
    } else {
        println!("⚠️ **MODERATE**: Some overlap detected - still better than racing");
    }

    println!("\n💡 **Key Benefits of This Approach:**");
    println!("====================================");
    println!("🚀 Maximum content discovery per API quota used");
    println!("🔍 Different engines find different relevant content");
    println!("💰 No quota waste from racing or redundant calls");
    println!("🌐 Broader domain coverage than single provider");
    println!("📊 Get both mainstream (Google) and specialized (Exa) perspectives");

    println!("\n🎯 **Recommendation**: Use this strategy for:");
    println!("• Research and comprehensive content discovery");
    println!("• When you want maximum coverage per quota");
    println!("• Finding diverse sources and perspectives");
    println!("• Building comprehensive link databases");

    Ok(())
}
