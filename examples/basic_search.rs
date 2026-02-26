//! Basic search example using the websearch SDK

use websearch::{providers::DuckDuckGoProvider, types::DebugOptions, web_search, SearchOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    // Example with DuckDuckGo (no API key required)
    println!("🦆 Testing DuckDuckGo search...");

    let duckduckgo = DuckDuckGoProvider::new();

    let results = web_search(SearchOptions {
        query: "Rust programming".to_string(),
        max_results: Some(3),
        provider: Box::new(duckduckgo),
        debug: Some(DebugOptions {
            enabled: true,
            log_requests: false,
            log_responses: true,
        }),
        ..Default::default()
    })
    .await?;

    println!("Found {} results:", results.len());
    for (i, result) in results.iter().enumerate() {
        println!("{}. {}", i + 1, result.title);
        println!("   URL: {}", result.url);
        if let Some(snippet) = &result.snippet {
            println!("   {snippet}");
        }
        println!();
    }

    Ok(())
}
