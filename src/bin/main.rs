//! WebSearch CLI - Command-line interface for the websearch SDK
//!
//! A simple CLI tool for searching via DuckDuckGo and ArXiv.

use clap::{Parser, ValueEnum};
use colored::*;
use websearch::{
    providers::*,
    types::{DebugOptions, SafeSearch, SearchOptions, SortBy, SortOrder},
    web_search,
};

#[derive(Parser)]
#[command(name = "websearch")]
#[command(about = "Web search CLI (DuckDuckGo & ArXiv)")]
#[command(version)]
struct Cli {
    /// Search query
    #[arg(value_name = "QUERY")]
    query: Option<String>,

    /// Search provider (duckduckgo or arxiv)
    #[arg(short, long, value_enum, default_value = "duckduckgo")]
    provider: Option<Provider>,

    /// Maximum number of results
    #[arg(short, long, default_value = "10")]
    max_results: Option<u32>,

    /// Language code (e.g., en, es, fr)
    #[arg(short, long)]
    language: Option<String>,

    /// Region code (e.g., US, UK, DE)
    #[arg(short, long)]
    region: Option<String>,

    /// Safe search setting
    #[arg(short, long, value_enum)]
    safe_search: Option<SafeSearchCli>,

    /// ArXiv paper IDs (comma-separated, for ArXiv provider)
    #[arg(long)]
    arxiv_ids: Option<String>,

    /// Sort by field (for ArXiv)
    #[arg(long, value_enum)]
    sort_by: Option<SortByCli>,

    /// Sort order (for ArXiv)
    #[arg(long, value_enum)]
    sort_order: Option<SortOrderCli>,

    /// Enable debug output
    #[arg(short, long)]
    debug: bool,

    /// Show raw provider response
    #[arg(long)]
    raw: bool,

    /// Output format
    #[arg(short, long, value_enum, default_value = "table")]
    format: OutputFormat,
}

#[derive(ValueEnum, Clone, Debug)]
enum Provider {
    Duckduckgo,
    Arxiv,
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

    if let Some(query) = cli.query {
        let provider = cli.provider.unwrap_or(Provider::Duckduckgo);
        let max_results = cli.max_results.unwrap_or(10);

        handle_search(
            query,
            provider,
            max_results,
            cli.language,
            cli.region,
            cli.safe_search,
            cli.arxiv_ids,
            cli.sort_by,
            cli.sort_order,
            cli.debug,
            cli.raw,
            cli.format,
        )
        .await?;
    } else {
        eprintln!("{}", "Error: Search query is required".red());
        eprintln!("Usage: websearch \"your search query\" --provider duckduckgo");
        eprintln!("Try: websearch --help");
        std::process::exit(1);
    }

    Ok(())
}

async fn handle_search(
    query: String,
    provider: Provider,
    max_results: u32,
    language: Option<String>,
    region: Option<String>,
    safe_search: Option<SafeSearchCli>,
    arxiv_ids: Option<String>,
    sort_by: Option<SortByCli>,
    sort_order: Option<SortOrderCli>,
    debug: bool,
    raw: bool,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let provider_name = format!("{:?}", provider).to_lowercase();
    let provider_box = create_provider(provider);

    // For ArXiv, use either query or IDs
    let (search_query, id_list) = if provider_name == "arxiv" {
        if let Some(ids) = arxiv_ids {
            ("".to_string(), Some(ids))
        } else {
            (query.clone(), None)
        }
    } else {
        (query.clone(), None)
    };

    let options = SearchOptions {
        query: search_query,
        id_list,
        max_results: Some(max_results),
        language,
        region,
        safe_search: safe_search.map(|s| match s {
            SafeSearchCli::Off => SafeSearch::Off,
            SafeSearchCli::Moderate => SafeSearch::Moderate,
            SafeSearchCli::Strict => SafeSearch::Strict,
        }),
        sort_by: sort_by.map(|s| match s {
            SortByCli::Relevance => SortBy::Relevance,
            SortByCli::SubmittedDate => SortBy::SubmittedDate,
            SortByCli::LastUpdatedDate => SortBy::LastUpdatedDate,
        }),
        sort_order: sort_order.map(|s| match s {
            SortOrderCli::Ascending => SortOrder::Ascending,
            SortOrderCli::Descending => SortOrder::Descending,
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

    display_results(&results, &format, raw, &provider_name);
    Ok(())
}

fn create_provider(provider: Provider) -> Box<dyn websearch::types::SearchProvider> {
    match provider {
        Provider::Duckduckgo => Box::new(DuckDuckGoProvider::new()),
        Provider::Arxiv => Box::new(ArxivProvider::new()),
    }
}

fn display_results(
    results: &[websearch::types::SearchResult],
    format: &OutputFormat,
    show_raw: bool,
    provider: &str,
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
            println!("{} {}", "Search Results from".bold(), provider.bold().blue());
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
