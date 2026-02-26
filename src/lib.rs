//! # WebSearch - Rust Implementation
//!
//! A simple Rust SDK for web search via DuckDuckGo and ArXiv.
//!
//! ## Quick Start
//!
//! ```rust
//! use websearch::{web_search, providers::DuckDuckGoProvider, SearchOptions};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let duckduckgo = DuckDuckGoProvider::new();
//!
//!     let results = web_search(SearchOptions {
//!         query: "Rust programming language".to_string(),
//!         max_results: Some(5),
//!         provider: Box::new(duckduckgo),
//!         ..Default::default()
//!     }).await?;
//!
//!     for result in results {
//!         println!("{}: {}", result.title, result.url);
//!     }
//!
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod providers;
pub mod types;
pub mod utils;

// Re-export common types
pub use error::{SearchError, SearchResult as Result};
pub use types::{DebugOptions, SearchOptions, SearchProvider, SearchResult};

/// Main search function that queries a web search provider and returns standardized results
///
/// # Arguments
///
/// * `options` - Search options including provider, query and other parameters
///
/// # Returns
///
/// A vector of search results or an error
///
/// # Examples
///
/// ```rust
/// use websearch::{web_search, providers::DuckDuckGoProvider, SearchOptions};
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let provider = DuckDuckGoProvider::new();
/// let results = web_search(SearchOptions {
///     query: "rust programming".to_string(),
///     provider: Box::new(provider),
///     ..Default::default()
/// }).await?;
/// # Ok(())
/// # }
/// ```
pub async fn web_search(options: SearchOptions) -> Result<Vec<SearchResult>> {
    use error::SearchError;
    use utils::debug;

    // Validate required options
    if options.query.is_empty() && options.id_list.is_none() {
        return Err(SearchError::InvalidInput(
            "A search query or ID list (for Arxiv) is required".to_string(),
        ));
    }

    // Log search parameters if debugging is enabled
    debug::log(
        &options.debug,
        "Performing search",
        &format!(
            "provider: {}, query: {}",
            options.provider.name(),
            options.query
        ),
    );

    // Perform the search
    match options.provider.search(&options).await {
        Ok(results) => {
            debug::log_response(
                &options.debug,
                &format!("Received {} results", results.len()),
            );
            Ok(results)
        }
        Err(error) => {
            let troubleshooting = get_troubleshooting_info(options.provider.name(), &error);
            let detailed_error = format!(
                "Search with provider '{}' failed: {}\n\nTroubleshooting: {}",
                options.provider.name(),
                error,
                troubleshooting
            );

            debug::log(&options.debug, "Search error", &detailed_error);
            Err(SearchError::ProviderError(detailed_error))
        }
    }
}

/// Get provider-specific troubleshooting information based on error
fn get_troubleshooting_info(provider_name: &str, error: &SearchError) -> String {
    // Common troubleshooting based on error type
    match error {
        SearchError::HttpError {
            status_code: Some(401 | 403),
            ..
        } => {
            "This is likely an authentication issue. Check your API key and make sure it's valid and has the correct permissions.".to_string()
        }
        SearchError::HttpError {
            status_code: Some(400),
            ..
        } => {
            "This is likely due to invalid request parameters. Check your query and other search options.".to_string()
        }
        SearchError::HttpError {
            status_code: Some(429),
            ..
        } => {
            "You've exceeded the rate limit for this API. Try again later or reduce your request frequency.".to_string()
        }
        SearchError::HttpError {
            status_code: Some(500..=599),
            ..
        } => {
            "The search provider is experiencing server issues. Try again later.".to_string()
        }
        _ => {
            // Provider-specific troubleshooting
            match provider_name {
                "duckduckgo" => "You may be making too many requests to DuckDuckGo. Try adding a delay between requests or reduce your request frequency.".to_string(),
                "arxiv" => "ArXiv may be temporarily unavailable. Try again later or reduce your request frequency.".to_string(),
                _ => format!("Check your {provider_name} configuration and make sure your search request is valid."),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use async_trait::async_trait;

    // Mock provider for testing
    #[derive(Debug)]
    struct MockProvider {
        name: String,
        should_error: bool,
        error_type: Option<SearchError>,
        results: Vec<SearchResult>,
    }

    impl MockProvider {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                should_error: false,
                error_type: None,
                results: vec![
                    SearchResult {
                        title: "Test Result 1".to_string(),
                        url: "https://example.com/1".to_string(),
                        snippet: Some("Test content 1".to_string()),
                        domain: None,
                        published_date: None,
                        provider: Some(name.to_string()),
                        raw: None,
                    },
                    SearchResult {
                        title: "Test Result 2".to_string(),
                        url: "https://example.com/2".to_string(),
                        snippet: Some("Test content 2".to_string()),
                        domain: None,
                        published_date: None,
                        provider: Some(name.to_string()),
                        raw: None,
                    },
                ],
            }
        }

        fn with_error(mut self, error: SearchError) -> Self {
            self.should_error = true;
            self.error_type = Some(error);
            self
        }

        fn with_results(mut self, results: Vec<SearchResult>) -> Self {
            self.results = results;
            self
        }
    }

    #[async_trait]
    impl SearchProvider for MockProvider {
        fn name(&self) -> &str {
            &self.name
        }

        async fn search(&self, _options: &SearchOptions) -> Result<Vec<SearchResult>> {
            if self.should_error {
                Err(self
                    .error_type
                    .clone()
                    .unwrap_or(SearchError::Other("Mock error".to_string())))
            } else {
                Ok(self.results.clone())
            }
        }
    }

    #[tokio::test]
    async fn test_web_search_success() {
        let provider = MockProvider::new("test");
        let options = SearchOptions {
            query: "test query".to_string(),
            provider: Box::new(provider),
            ..Default::default()
        };

        let results = web_search(options).await.unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "Test Result 1");
        assert_eq!(results[0].url, "https://example.com/1");
        assert_eq!(results[0].provider, Some("test".to_string()));
    }

    #[tokio::test]
    async fn test_web_search_empty_query() {
        let provider = MockProvider::new("test");
        let options = SearchOptions {
            query: "".to_string(),
            provider: Box::new(provider),
            ..Default::default()
        };

        let result = web_search(options).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            SearchError::InvalidInput(msg) => {
                assert!(msg.contains("search query or ID list"));
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[tokio::test]
    async fn test_web_search_provider_error() {
        let provider = MockProvider::new("test").with_error(SearchError::HttpError {
            status_code: Some(401),
            message: "Unauthorized".to_string(),
            response_body: None,
        });
        let options = SearchOptions {
            query: "test query".to_string(),
            provider: Box::new(provider),
            ..Default::default()
        };

        let result = web_search(options).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            SearchError::ProviderError(msg) => {
                assert!(msg.contains("failed"));
                assert!(msg.contains("authentication issue"));
            }
            _ => panic!("Expected ProviderError"),
        }
    }

    #[tokio::test]
    async fn test_troubleshooting_info_http_errors() {
        let test_cases = vec![
            (
                SearchError::HttpError {
                    status_code: Some(401),
                    message: "Unauthorized".to_string(),
                    response_body: None,
                },
                "authentication issue",
            ),
            (
                SearchError::HttpError {
                    status_code: Some(403),
                    message: "Forbidden".to_string(),
                    response_body: None,
                },
                "authentication issue",
            ),
            (
                SearchError::HttpError {
                    status_code: Some(400),
                    message: "Bad Request".to_string(),
                    response_body: None,
                },
                "invalid request parameters",
            ),
            (
                SearchError::HttpError {
                    status_code: Some(429),
                    message: "Too Many Requests".to_string(),
                    response_body: None,
                },
                "rate limit",
            ),
            (
                SearchError::HttpError {
                    status_code: Some(500),
                    message: "Internal Server Error".to_string(),
                    response_body: None,
                },
                "server issues",
            ),
        ];

        for (error, expected_text) in test_cases {
            let info = get_troubleshooting_info("test", &error);
            assert!(
                info.to_lowercase().contains(expected_text),
                "Expected '{info}' to contain '{expected_text}'"
            );
        }
    }

    #[tokio::test]
    async fn test_web_search_with_arxiv_id_list() {
        let provider = MockProvider::new("arxiv");
        let options = SearchOptions {
            query: "".to_string(), // Empty query is OK for arxiv with id_list
            id_list: Some("1234.5678,2345.6789".to_string()),
            provider: Box::new(provider),
            ..Default::default()
        };

        let results = web_search(options).await.unwrap();
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_web_search_max_results() {
        let results = vec![
            SearchResult {
                title: "Result 1".to_string(),
                url: "https://example.com/1".to_string(),
                snippet: Some("Content 1".to_string()),
                domain: None,
                published_date: None,
                provider: Some("test".to_string()),
                raw: None,
            },
            SearchResult {
                title: "Result 2".to_string(),
                url: "https://example.com/2".to_string(),
                snippet: Some("Content 2".to_string()),
                domain: None,
                published_date: None,
                provider: Some("test".to_string()),
                raw: None,
            },
            SearchResult {
                title: "Result 3".to_string(),
                url: "https://example.com/3".to_string(),
                snippet: Some("Content 3".to_string()),
                domain: None,
                published_date: None,
                provider: Some("test".to_string()),
                raw: None,
            },
        ];

        let provider = MockProvider::new("test").with_results(results);
        let options = SearchOptions {
            query: "test".to_string(),
            max_results: Some(2),
            provider: Box::new(provider),
            ..Default::default()
        };

        let search_results = web_search(options).await.unwrap();
        assert!(search_results.len() >= 2);
    }
}
