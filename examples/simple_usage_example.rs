//! Simple example showing how to use the websearch SDK

use websearch::{providers::ArxivProvider, types::DebugOptions, web_search, SearchOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Simple Usage Example");
    println!("======================\n");

    // Example 1: DuckDuckGo search (no API key required)
    println!("📋 Example 1: DuckDuckGo Search");
    println!("-------------------------------");

    let duckduckgo = websearch::providers::DuckDuckGoProvider::new();

    let results = web_search(SearchOptions {
        query: "machine learning".to_string(),
        max_results: Some(3),
        provider: Box::new(duckduckgo),
        ..Default::default()
    })
    .await?;

    println!("✅ Found {} results:", results.len());
    for (i, result) in results.iter().enumerate() {
        println!("  {}. {}", i + 1, result.title);
        println!("     🔗 {}", result.url);
    }

    println!("\n{}\n", "=".repeat(50));

    // Example 2: ArXiv search (no API key required)
    println!("📋 Example 2: ArXiv Search");
    println!("---------------------------");

    let arxiv = ArxivProvider::new();

    let results = web_search(SearchOptions {
        query: "quantum computing".to_string(),
        max_results: Some(3),
        provider: Box::new(arxiv),
        debug: Some(DebugOptions {
            enabled: false,
            log_requests: false,
            log_responses: false,
        }),
        ..Default::default()
    })
    .await?;

    println!("✅ Found {} ArXiv papers:", results.len());
    for (i, result) in results.iter().enumerate() {
        println!("  {}. {}", i + 1, result.title);
        println!("     🔗 {}", result.url);
        if let Some(date) = &result.published_date {
            println!("     📅 {}", date);
        }
    }

    println!("\n💡 **How to use this library:**");
    println!("================================");
    println!();
    println!("1. Add to Cargo.toml:");
    println!("   [dependencies]");
    println!("   websearch = \"0.1\"");
    println!("   tokio = {{ version = \"1.0\", features = [\"full\"] }}");
    println!();
    println!("2. Use in code:");
    println!("   use websearch::{{providers::*, web_search, SearchOptions}};");
    println!();
    println!("3. Create a provider and search:");
    println!("   let ddg = DuckDuckGoProvider::new();");
    println!("   let results = web_search(SearchOptions {{");
    println!("       query: \"your query\".to_string(),");
    println!("       provider: Box::new(ddg),");
    println!("       ..Default::default()");
    println!("   }}).await?;");

    Ok(())
}
