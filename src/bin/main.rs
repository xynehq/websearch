//! WebSearch CLI - Command-line interface for the websearch SDK
//!
//! A powerful CLI tool for searching across multiple search providers including
//! Google, Tavily, Exa, SerpAPI, DuckDuckGo, Brave, SearXNG, and ArXiv.

use clap::{Parser, Subcommand, ValueEnum};
use colored::*;
use std::env;
use websearch::{
    multi_provider::{MultiProviderConfig, MultiProviderSearch, MultiProviderStrategy, SearchOptionsMulti},
    providers::*,
    types::{DebugOptions, SafeSearch, SearchOptions, SortBy, SortOrder},
    web_search,
};

#[derive(Parser)]
#[command(name = "websearch")]
#[command(about = "Multi-provider web search CLI")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Search using a single provider
    Single {
        /// Search query
        query: String,

        /// Search provider
        #[arg(short, long, value_enum)]
        provider: Provider,

        /// Maximum number of results
        #[arg(short, long, default_value = "10")]
        max_results: u32,

        /// Language code (e.g., en, es, fr)
        #[arg(short, long)]
        language: Option<String>,

        /// Region code (e.g., US, UK, DE)
        #[arg(short, long)]
        region: Option<String>,

        /// Safe search setting
        #[arg(short, long, value_enum)]
        safe_search: Option<SafeSearchCli>,

        /// Enable debug output
        #[arg(short, long)]
        debug: bool,

        /// Show raw provider response
        #[arg(long)]
        raw: bool,

        /// Output format
        #[arg(short, long, value_enum, default_value = "table")]
        format: OutputFormat,
    },
    /// Search using multiple providers
    Multi {
        /// Search query
        query: String,

        /// Multi-provider strategy
        #[arg(short, long, value_enum, default_value = "aggregate")]
        strategy: StrategyCli,

        /// Providers to use (if not specified, uses available providers)
        #[arg(short, long, value_enum)]
        providers: Vec<Provider>,

        /// Maximum number of results per provider
        #[arg(short, long, default_value = "5")]
        max_results: u32,

        /// Enable debug output
        #[arg(short, long)]
        debug: bool,

        /// Output format
        #[arg(short, long, value_enum, default_value = "table")]
        format: OutputFormat,

        /// Show provider statistics
        #[arg(long)]
        stats: bool,
    },
    /// Search ArXiv papers by ID
    Arxiv {
        /// Comma-separated ArXiv IDs (e.g., "1234.5678,2345.6789")
        ids: String,

        /// Maximum number of results
        #[arg(short, long, default_value = "10")]
        max_results: u32,

        /// Sort by field
        #[arg(long, value_enum)]
        sort_by: Option<SortByCli>,

        /// Sort order
        #[arg(long, value_enum)]
        sort_order: Option<SortOrderCli>,

        /// Output format
        #[arg(short, long, value_enum, default_value = "table")]
        format: OutputFormat,
    },
    /// List available providers and their status
    Providers,
}

#[derive(ValueEnum, Clone, Debug)]
enum Provider {
    Google,
    Tavily,
    Exa,
    Serpapi,
    Duckduckgo,
    Brave,
    Searxng,
    Arxiv,
}

#[derive(ValueEnum, Clone, Debug)]
enum StrategyCli {
    Failover,
    LoadBalance,
    Aggregate,
    Race,
}

#[derive(ValueEnum, Clone, Debug)]
enum SafeSearchCli {
    Off,
    Moderate,
    Strict,
}

#[derive(ValueEnum, Clone, Debug)]
enum SortByCli {
    Relevance,
    SubmittedDate,
    LastUpdatedDate,
}

#[derive(ValueEnum, Clone, Debug)]
enum SortOrderCli {
    Ascending,
    Descending,
}

#[derive(ValueEnum, Clone, Debug)]
enum OutputFormat {
    Table,
    Json,
    Simple,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Single {
            query,
            provider,
            max_results,
            language,
            region,
            safe_search,
            debug,
            raw,
            format,
        } => {
            handle_single_search(
                query,
                provider,
                max_results,
                language,
                region,
                safe_search,
                debug,
                raw,
                format,
            )
            .await?;
        }
        Commands::Multi {
            query,
            strategy,
            providers,
            max_results,
            debug,
            format,
            stats,
        } => {
            handle_multi_search(query, strategy, providers, max_results, debug, format, stats).await?;
        }
        Commands::Arxiv {
            ids,
            max_results,
            sort_by,
            sort_order,
            format,
        } => {
            handle_arxiv_search(ids, max_results, sort_by, sort_order, format).await?;
        }
        Commands::Providers => {
            handle_list_providers().await?;
        }
    }

    Ok(())
}

async fn handle_single_search(
    query: String,
    provider: Provider,
    max_results: u32,
    language: Option<String>,
    region: Option<String>,
    safe_search: Option<SafeSearchCli>,
    debug: bool,
    raw: bool,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let provider_name = format!("{:?}", provider).to_lowercase();
    let provider_box = create_provider(provider).await?;

    let options = SearchOptions {
        query: query.clone(),
        max_results: Some(max_results),
        language,
        region,
        safe_search: safe_search.map(|s| match s {
            SafeSearchCli::Off => SafeSearch::Off,
            SafeSearchCli::Moderate => SafeSearch::Moderate,
            SafeSearchCli::Strict => SafeSearch::Strict,
        }),
        debug: if debug {
            Some(DebugOptions {
                enabled: true,
                log_requests: true,
                log_responses: false,
            })
        } else {
            None
        },
        provider: provider_box,
        ..Default::default()
    };

    let results = web_search(options).await?;

    display_results(&results, &format, raw, Some(&provider_name));
    Ok(())
}

async fn handle_multi_search(
    query: String,
    strategy: StrategyCli,
    providers: Vec<Provider>,
    max_results: u32,
    debug: bool,
    format: OutputFormat,
    stats: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let strategy = match strategy {
        StrategyCli::Failover => MultiProviderStrategy::Failover,
        StrategyCli::LoadBalance => MultiProviderStrategy::LoadBalance,
        StrategyCli::Aggregate => MultiProviderStrategy::Aggregate,
        StrategyCli::Race => MultiProviderStrategy::Aggregate, // Use Aggregate as Race strategy
    };

    let mut config = MultiProviderConfig::new(strategy);

    // If no providers specified, try to add all available ones
    let providers_to_use = if providers.is_empty() {
        get_available_providers().await
    } else {
        providers
    };

    for provider in providers_to_use {
        if let Ok(provider_box) = create_provider(provider).await {
            config = config.add_provider(provider_box);
        }
    }

    let mut multi_search = MultiProviderSearch::new(config);

    let options = SearchOptionsMulti {
        query: query.clone(),
        max_results: Some(max_results),
        debug: if debug {
            Some(DebugOptions {
                enabled: true,
                log_requests: true,
                log_responses: false,
            })
        } else {
            None
        },
        ..Default::default()
    };

    let results = multi_search.search(&options).await?;

    display_results(&results, &format, false, None);

    if stats {
        display_provider_stats(&multi_search);
    }

    Ok(())
}

async fn handle_arxiv_search(
    ids: String,
    max_results: u32,
    sort_by: Option<SortByCli>,
    sort_order: Option<SortOrderCli>,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let arxiv = ArxivProvider::new();

    let options = SearchOptions {
        query: "".to_string(), // ArXiv uses id_list instead
        id_list: Some(ids),
        max_results: Some(max_results),
        sort_by: sort_by.map(|s| match s {
            SortByCli::Relevance => SortBy::Relevance,
            SortByCli::SubmittedDate => SortBy::SubmittedDate,
            SortByCli::LastUpdatedDate => SortBy::LastUpdatedDate,
        }),
        sort_order: sort_order.map(|s| match s {
            SortOrderCli::Ascending => SortOrder::Ascending,
            SortOrderCli::Descending => SortOrder::Descending,
        }),
        provider: Box::new(arxiv),
        ..Default::default()
    };

    let results = web_search(options).await?;

    display_results(&results, &format, false, Some("arxiv"));
    Ok(())
}

async fn handle_list_providers() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "Available Search Providers:".bold().blue());
    println!();

    let providers = [
        ("Google", "Requires GOOGLE_API_KEY and GOOGLE_CX"),
        ("Tavily", "Requires TAVILY_API_KEY (AI-powered search)"),
        ("Exa", "Requires EXA_API_KEY (semantic search)"),
        ("SerpAPI", "Requires SERPAPI_API_KEY"),
        ("DuckDuckGo", "No API key required"),
        ("Brave", "Requires BRAVE_API_KEY"),
        ("SearXNG", "Requires SEARXNG_URL"),
        ("ArXiv", "No API key required"),
    ];

    for (name, requirement) in providers {
        let status = check_provider_availability(name).await;
        let status_color = if status { "✅".green() } else { "❌".red() };
        println!("{} {} - {}", status_color, name.bold(), requirement.italic());
    }

    println!();
    println!("{}", "Set environment variables to enable providers:".bold());
    println!("export GOOGLE_API_KEY=your_key");
    println!("export GOOGLE_CX=your_search_engine_id");
    println!("export TAVILY_API_KEY=tvly-dev-your_key");
    println!("export EXA_API_KEY=your_key");
    println!("export SERPAPI_API_KEY=your_key");
    println!("export BRAVE_API_KEY=your_key");
    println!("export SEARXNG_URL=https://your-searxng-instance.com");

    Ok(())
}

async fn create_provider(provider: Provider) -> Result<Box<dyn websearch::types::SearchProvider>, Box<dyn std::error::Error>> {
    match provider {
        Provider::Google => {
            let api_key = env::var("GOOGLE_API_KEY")?;
            let cx = env::var("GOOGLE_CX")?;
            Ok(Box::new(GoogleProvider::new(&api_key, &cx)?))
        }
        Provider::Tavily => {
            let api_key = env::var("TAVILY_API_KEY")?;
            Ok(Box::new(TavilyProvider::new(&api_key)?))
        }
        Provider::Exa => {
            let api_key = env::var("EXA_API_KEY")?;
            Ok(Box::new(ExaProvider::new(&api_key)?))
        }
        Provider::Serpapi => {
            let api_key = env::var("SERPAPI_API_KEY")?;
            Ok(Box::new(SerpApiProvider::new(&api_key)?))
        }
        Provider::Duckduckgo => Ok(Box::new(DuckDuckGoProvider::new())),
        Provider::Brave => {
            let api_key = env::var("BRAVE_API_KEY")?;
            Ok(Box::new(BraveProvider::new(&api_key)?))
        }
        Provider::Searxng => {
            let url = env::var("SEARXNG_URL")?;
            Ok(Box::new(SearxNGProvider::new(&url)?))
        }
        Provider::Arxiv => Ok(Box::new(ArxivProvider::new())),
    }
}

async fn get_available_providers() -> Vec<Provider> {
    let mut available = Vec::new();

    if env::var("GOOGLE_API_KEY").is_ok() && env::var("GOOGLE_CX").is_ok() {
        available.push(Provider::Google);
    }
    if env::var("TAVILY_API_KEY").is_ok() {
        available.push(Provider::Tavily);
    }
    if env::var("EXA_API_KEY").is_ok() {
        available.push(Provider::Exa);
    }
    if env::var("SERPAPI_API_KEY").is_ok() {
        available.push(Provider::Serpapi);
    }
    available.push(Provider::Duckduckgo); // Always available
    if env::var("BRAVE_API_KEY").is_ok() {
        available.push(Provider::Brave);
    }
    if env::var("SEARXNG_URL").is_ok() {
        available.push(Provider::Searxng);
    }
    available.push(Provider::Arxiv); // Always available

    available
}

async fn check_provider_availability(provider_name: &str) -> bool {
    match provider_name {
        "Google" => env::var("GOOGLE_API_KEY").is_ok() && env::var("GOOGLE_CX").is_ok(),
        "Tavily" => env::var("TAVILY_API_KEY").is_ok(),
        "Exa" => env::var("EXA_API_KEY").is_ok(),
        "SerpAPI" => env::var("SERPAPI_API_KEY").is_ok(),
        "DuckDuckGo" => true,
        "Brave" => env::var("BRAVE_API_KEY").is_ok(),
        "SearXNG" => env::var("SEARXNG_URL").is_ok(),
        "ArXiv" => true,
        _ => false,
    }
}

fn display_results(
    results: &[websearch::types::SearchResult],
    format: &OutputFormat,
    show_raw: bool,
    provider: Option<&str>,
) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(results).unwrap());
        }
        OutputFormat::Simple => {
            for (i, result) in results.iter().enumerate() {
                println!("{}. {}", i + 1, result.title);
                println!("   {}", result.url);
                if let Some(snippet) = &result.snippet {
                    println!("   {}", snippet);
                }
                println!();
            }
        }
        OutputFormat::Table => {
            if let Some(provider) = provider {
                println!("{} {}", "Search Results from".bold(), provider.bold().blue());
            } else {
                println!("{}", "Search Results".bold().blue());
            }
            println!("{}", "─".repeat(80).dimmed());

            for (i, result) in results.iter().enumerate() {
                println!("{}. {}", (i + 1).to_string().bold(), result.title.bold());
                println!("   🔗 {}", result.url.blue().underline());

                if let Some(domain) = &result.domain {
                    println!("   🌐 {}", domain.green());
                }

                if let Some(snippet) = &result.snippet {
                    let truncated = if snippet.len() > 200 {
                        format!("{}...", &snippet[..200])
                    } else {
                        snippet.clone()
                    };
                    println!("   📄 {}", truncated.italic());
                }

                if let Some(published_date) = &result.published_date {
                    println!("   📅 {}", published_date.yellow());
                }

                if let Some(provider) = &result.provider {
                    println!("   🔍 Provider: {}", provider.cyan());
                }

                if show_raw {
                    if let Some(raw) = &result.raw {
                        println!("   📊 Raw: {}", serde_json::to_string_pretty(raw).unwrap());
                    }
                }

                println!();
            }

            println!("{} {}", "Total results:".bold(), results.len().to_string().bold());
        }
    }
}

fn display_provider_stats(multi_search: &MultiProviderSearch) {
    let stats = multi_search.get_stats();

    println!();
    println!("{}", "Provider Statistics:".bold().blue());
    println!("{}", "─".repeat(80).dimmed());

    for (provider, stat) in stats {
        println!("{}:", provider.bold());
        println!("  Total requests: {}", stat.total_requests);
        println!("  Successful: {}", stat.successful_requests.to_string().green());
        println!("  Failed: {}", stat.failed_requests.to_string().red());
        println!("  Avg response time: {:.2}ms", stat.avg_response_time_ms);
        if stat.total_requests > 0 {
            let success_rate = (stat.successful_requests as f64 / stat.total_requests as f64) * 100.0;
            println!("  Success rate: {:.1}%", success_rate);
        }
        println!();
    }
}