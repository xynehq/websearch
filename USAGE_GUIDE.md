# 🔍 WebSearch SDK Usage Guide

This guide shows how other repositories can use this web search SDK in different ways.

## 🚀 Option 1: Published Crate (Recommended)

### Step 1: Publish to crates.io

```bash
# Prepare for publishing
cargo login  # Enter your crates.io token
cargo publish --dry-run  # Test the package
cargo publish  # Publish to crates.io
```

### Step 2: Use in Other Projects

```toml
# In another project's Cargo.toml
[dependencies]
websearch = "0.0.1"
tokio = { version = "1.0", features = ["full"] }
```

```rust
// In another project's main.rs
use websearch::{
    providers::{GoogleProvider, TavilyProvider, ExaProvider, DuckDuckGoProvider},
    multi_provider::{MultiProviderSearch, MultiProviderConfig, MultiProviderStrategy},
    types::SearchOptions,
    web_search,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Single provider usage with Tavily AI search
    let tavily = TavilyProvider::new_advanced("tvly-dev-your_api_key")?
        .with_answer(true)
        .with_images(false);

    let results = web_search(SearchOptions {
        query: "latest AI developments 2024".to_string(),
        max_results: Some(5),
        provider: Box::new(tavily),
        ..Default::default()
    }).await?;

    // Or use Google Custom Search
    let google = GoogleProvider::new("your_api_key", "your_cx")?;
    let results = web_search(SearchOptions {
        query: "rust programming".to_string(),
        max_results: Some(5),
        provider: Box::new(google),
        ..Default::default()
    }).await?;

    // Multi-provider usage - combining AI and traditional search
    let mut config = MultiProviderConfig::new(MultiProviderStrategy::Aggregate);
    config = config
        .add_provider(Box::new(TavilyProvider::new("tvly-dev-key")?))
        .add_provider(Box::new(GoogleProvider::new("google_key", "cx")?))
        .add_provider(Box::new(DuckDuckGoProvider::new()));

    let mut multi_search = MultiProviderSearch::new(config);
    let results = multi_search.search(&SearchOptionsMulti {
        query: "machine learning".to_string(),
        max_results: Some(10),
        ..Default::default()
    }).await?;

    Ok(())
}
```

## 📦 Option 2: Git Dependency

### Direct Git Usage

```toml
# In another project's Cargo.toml
[dependencies]
search-sdk-rust = { git = "https://github.com/your-username/search-sdk", branch = "main" }
```

### Specific Commit/Tag

```toml
[dependencies]
search-sdk-rust = { git = "https://github.com/your-username/search-sdk", tag = "v0.1.0" }
# or
search-sdk-rust = { git = "https://github.com/your-username/search-sdk", rev = "abc123" }
```

## 🔧 Option 3: Local Path (Development)

```toml
# In another project's Cargo.toml
[dependencies]
search-sdk-rust = { path = "../search-sdk" }
```

## 🌐 Option 4: Workspace (Monorepo)

### Create a workspace structure:

```
my-project/
├── Cargo.toml          # Workspace root
├── search-sdk/         # This search SDK
│   ├── Cargo.toml
│   └── src/
└── my-app/            # Your application
    ├── Cargo.toml
    └── src/
```

### Workspace Cargo.toml:

```toml
[workspace]
members = ["search-sdk", "my-app"]

[workspace.dependencies]
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
```

### App Cargo.toml:

```toml
[dependencies]
search-sdk-rust = { path = "../search-sdk" }
tokio.workspace = true
```

## 🔌 Option 5: Feature Flags

### Update Cargo.toml for optional features:

```toml
[features]
default = ["google", "exa"]
google = []
exa = []
tavily = []
serpapi = []
multi-provider = []
all = ["google", "exa", "tavily", "serpapi", "multi-provider"]
```

### Usage with specific features:

```toml
# Only Google provider
websearch = { version = "0.0.1", features = ["google"] }

# All providers
websearch = { version = "0.0.1", features = ["all"] }
```

## 📚 Complete Example Project

### project/Cargo.toml
```toml
[package]
name = "my-search-app"
version = "0.1.0"
edition = "2021"

[dependencies]
websearch = "0.0.1"
tokio = { version = "1.0", features = ["full"] }
env_logger = "0.10"
clap = { version = "4.0", features = ["derive"] }
```

### project/src/main.rs
```rust
use websearch::{
    providers::{GoogleProvider, ExaProvider},
    multi_provider::{MultiProviderSearch, MultiProviderConfig, MultiProviderStrategy, SearchOptionsMulti},
};
use clap::Parser;
use std::env;

#[derive(Parser)]
#[command(name = "search")]
#[command(about = "Multi-provider search tool")]
struct Args {
    /// Search query
    query: String,

    /// Maximum results
    #[arg(short, long, default_value = "10")]
    max_results: usize,

    /// Provider (google, exa, multi)
    #[arg(short, long, default_value = "multi")]
    provider: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let args = Args::parse();

    match args.provider.as_str() {
        "google" => {
            let google = GoogleProvider::new(
                &env::var("GOOGLE_API_KEY")?,
                &env::var("GOOGLE_CX")?
            )?;

            let results = google.search(&websearch::types::SearchOptions {
                query: args.query,
                max_results: Some(args.max_results as u32),
                provider: Box::new(google),
                ..Default::default()
            }).await?;

            print_results(results);
        },
        "exa" => {
            let exa = ExaProvider::new(&env::var("EXA_API_KEY")?)?;
            // Similar implementation
        },
        "multi" => {
            let mut config = MultiProviderConfig::new(MultiProviderStrategy::Aggregate);

            if let (Ok(key), Ok(cx)) = (env::var("GOOGLE_API_KEY"), env::var("GOOGLE_CX")) {
                config = config.add_provider(Box::new(GoogleProvider::new(&key, &cx)?));
            }

            if let Ok(key) = env::var("EXA_API_KEY") {
                config = config.add_provider(Box::new(ExaProvider::new(&key)?));
            }

            let mut multi_search = MultiProviderSearch::new(config);
            let results = multi_search.search(&SearchOptionsMulti {
                query: args.query,
                max_results: Some(args.max_results as u32),
                ..Default::default()
            }).await?;

            print_results(results);
        },
        _ => println!("Unknown provider: {}", args.provider),
    }

    Ok(())
}

fn print_results(results: Vec<websearch::types::SearchResult>) {
    for (i, result) in results.iter().enumerate() {
        println!("{}. {}", i + 1, result.title);
        println!("   🔗 {}", result.url);
        if let Some(snippet) = &result.snippet {
            println!("   📄 {}", snippet);
        }
        println!();
    }
}
```

### Usage:
```bash
# Set environment variables
export GOOGLE_API_KEY="your_key"
export GOOGLE_CX="your_cx"
export EXA_API_KEY="your_key"

# Run searches
cargo run -- "rust programming" --max-results 5 --provider google
cargo run -- "machine learning" --provider multi
```

## 🔐 Environment Setup

Create `.env` file in the using project:

```bash
# Google Custom Search
GOOGLE_API_KEY=your_google_api_key
GOOGLE_CX=your_custom_search_engine_id

# Tavily AI Search (Recommended for AI/LLM applications)
TAVILY_API_KEY=tvly-dev-your_api_key

# SerpAPI (Google, Bing, Yahoo)
SERPAPI_API_KEY=your_serpapi_key

# Exa Semantic Search
EXA_API_KEY=your_exa_api_key

# DuckDuckGo (No API key required)
# Brave, SearXNG, ArXiv (No API keys required)
```

### 🤖 Tavily AI Search - Best for AI Applications

Tavily is specifically optimized for AI and LLM applications:

```rust
use websearch::{web_search, providers::TavilyProvider, types::SearchOptions};

// Basic AI-powered search
let tavily = TavilyProvider::new("tvly-dev-your_api_key")?;

// Advanced search with answer generation
let tavily_advanced = TavilyProvider::new_advanced("tvly-dev-your_api_key")?
    .with_answer(true)     // Get AI-generated answers
    .with_images(true)     // Include relevant images
    .with_search_depth("advanced")?; // More comprehensive results

let results = web_search(SearchOptions {
    query: "latest developments in quantum computing 2024".to_string(),
    max_results: Some(5),
    provider: Box::new(tavily_advanced),
    ..Default::default()
}).await?;

// Results include AI-optimized content and relevance scores
for result in results {
    println!("{}: {}", result.title, result.url);
    if let Some(snippet) = result.snippet {
        println!("  {}", snippet);
    }
    // Access Tavily-specific data
    if let Some(raw) = &result.raw {
        if let Some(score) = raw.get("score") {
            println!("  Relevance: {}", score);
        }
    }
}
```

## 📖 Documentation

Generate and publish docs:

```bash
cargo doc --open  # Generate local docs
cargo publish     # Docs auto-published to docs.rs
```

Access at: `https://docs.rs/websearch/`

## 🧪 Testing in Other Projects

```rust
#[cfg(test)]
mod tests {
    use websearch::providers::*;

    #[tokio::test]
    async fn test_google_search() {
        // Test code
    }
}
```

## 📦 Distribution Options Summary

| Method | Pros | Cons | Best For |
|--------|------|------|----------|
| **crates.io** | Easy to use, versioned, cached | Requires publishing | Public projects |
| **Git dependency** | Direct from source, private repos | Slower builds | Development, private |
| **Local path** | Fast iteration | Must be local | Active development |
| **Workspace** | Shared dependencies | Complex setup | Monorepos |

Choose the method that best fits your project's needs!