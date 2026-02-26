# WebSearch

A simple Rust library and CLI tool for web search via DuckDuckGo and ArXiv. No API keys required.

## Features

- **Zero Setup**: Works immediately with DuckDuckGo and ArXiv - no API keys needed
- **Dual Purpose**: Use as a CLI tool or integrate as a Rust library
- **Simple & Fast**: Minimal dependencies, fast async performance
- **Type Safe**: Full Rust type safety with comprehensive error handling

## Installation

```bash
# Install CLI tool
cargo install --git https://github.com/roowe/websearch.git

# Or add to your Cargo.toml
[dependencies]
websearch = "0.1"
tokio = { version = "1.0", features = ["full"] }
```

## CLI Usage

```bash
# Search with DuckDuckGo (default)
websearch "rust programming"

# Search with ArXiv for academic papers
websearch "quantum computing" --provider arxiv

# Get specific ArXiv papers by ID
websearch "" --provider arxiv --arxiv-ids "2301.00001,2301.00002"

# Control output
websearch "machine learning" --max-results 5 --format json

# Available options
websearch --help
```

### CLI Options

| Option | Description | Default |
|--------|-------------|---------|
| `--provider` | Search provider: `duckduckgo` or `arxiv` | `duckduckgo` |
| `--max-results` | Maximum number of results | `10` |
| `--format` | Output format: `table`, `json`, `simple` | `table` |
| `--arxiv-ids` | ArXiv paper IDs (comma-separated) | - |
| `--sort-by` | ArXiv sort: `relevance`, `submitted-date`, `last-updated-date` | - |
| `--sort-order` | ArXiv order: `ascending`, `descending` | - |
| `--debug` | Enable debug output | - |

## Library Usage

```rust
use websearch::{web_search, providers::DuckDuckGoProvider, SearchOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let results = web_search(SearchOptions {
        query: "rust programming".to_string(),
        max_results: Some(5),
        provider: Box::new(DuckDuckGoProvider::new()),
        ..Default::default()
    }).await?;

    for result in results {
        println!("{}: {}", result.title, result.url);
    }

    Ok(())
}
```

### ArXiv Example

```rust
use websearch::{web_search, providers::ArxivProvider, SearchOptions};

let results = web_search(SearchOptions {
    query: "machine learning".to_string(),
    max_results: Some(10),
    provider: Box::new(ArxivProvider::new()),
    ..Default::default()
}).await?;
```

## Search Result Format

```rust
pub struct SearchResult {
    pub url: String,              // Result URL
    pub title: String,            // Page title
    pub snippet: Option<String>,  // Description/excerpt
    pub domain: Option<String>,   // Source domain
    pub published_date: Option<String>, // Publication date (ArXiv)
    pub provider: Option<String>, // Provider name
}
```

## Error Handling

```rust
use websearch::error::SearchError;

match web_search(options).await {
    Ok(results) => println!("Found {} results", results.len()),
    Err(SearchError::HttpError { status_code, message, .. }) => {
        eprintln!("HTTP error {:?}: {}", status_code, message);
    }
    Err(e) => eprintln!("Search failed: {}", e),
}
```

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Run example
cargo run --example basic_search

# Install locally
cargo install --path .
```

## Supported Providers

| Provider | API Key | Description |
|----------|---------|-------------|
| **DuckDuckGo** | No | General web search via HTML scraping |
| **ArXiv** | No | Academic papers and research preprints |

## License

MIT
