//! Integration tests for the websearch SDK
//!
//! These tests cover edge cases, error handling, and integration between components.

use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use tokio::time::Duration;
use websearch::{error::SearchError, types::*, web_search};

// Mock provider that can be configured for various test scenarios
#[derive(Debug, Clone)]
struct TestProvider {
    name: String,
    behavior: TestProviderBehavior,
    call_count: Arc<Mutex<usize>>,
}

#[derive(Debug, Clone)]
enum TestProviderBehavior {
    Success(Vec<SearchResult>),
    Error(SearchError),
    Slow {
        delay_ms: u64,
        then: Box<TestProviderBehavior>,
    },
}

impl TestProvider {
    fn new(name: &str, behavior: TestProviderBehavior) -> Self {
        Self {
            name: name.to_string(),
            behavior,
            call_count: Arc::new(Mutex::new(0)),
        }
    }

    fn success(name: &str, results: Vec<SearchResult>) -> Self {
        Self::new(name, TestProviderBehavior::Success(results))
    }

    fn error(name: &str, error: SearchError) -> Self {
        Self::new(name, TestProviderBehavior::Error(error))
    }

    fn slow(name: &str, delay_ms: u64, then: TestProviderBehavior) -> Self {
        Self::new(
            name,
            TestProviderBehavior::Slow {
                delay_ms,
                then: Box::new(then),
            },
        )
    }

}

#[async_trait]
impl SearchProvider for TestProvider {
    fn name(&self) -> &str {
        &self.name
    }

    async fn search(&self, _options: &SearchOptions) -> websearch::Result<Vec<SearchResult>> {
        let _current_count = {
            let mut count = self.call_count.lock().unwrap();
            *count += 1;
            *count
        };

        match &self.behavior {
            TestProviderBehavior::Success(results) => Ok(results.clone()),
            TestProviderBehavior::Error(error) => Err(error.clone()),
            TestProviderBehavior::Slow { delay_ms, then } => {
                tokio::time::sleep(Duration::from_millis(*delay_ms)).await;
                match then.as_ref() {
                    TestProviderBehavior::Success(results) => Ok(results.clone()),
                    TestProviderBehavior::Error(error) => Err(error.clone()),
                    _ => Err(SearchError::Other(
                        "Nested slow behavior not supported".to_string(),
                    )),
                }
            }
        }
    }
}

// Helper function to create test search results
fn create_test_results(provider: &str, count: usize) -> Vec<SearchResult> {
    (1..=count)
        .map(|i| SearchResult {
            title: format!("{provider} Result {i}"),
            url: format!("https://{provider}.com/result/{i}"),
            snippet: Some(format!("{provider} content {i}")),
            domain: None,
            published_date: None,
            provider: Some(provider.to_string()),
            raw: None,
        })
        .collect()
}

#[tokio::test]
async fn test_search_with_unicode_query() {
    let results = create_test_results("unicode", 2);
    let provider = TestProvider::success("unicode", results);

    let options = SearchOptions {
        query: "🔍 search emoji 中文 العربية русский".to_string(),
        provider: Box::new(provider),
        ..Default::default()
    };

    let search_results = web_search(options).await.unwrap();
    assert_eq!(search_results.len(), 2);
    assert_eq!(search_results[0].provider, Some("unicode".to_string()));
}

#[tokio::test]
async fn test_search_with_very_long_query() {
    let long_query = "a".repeat(10000); // 10KB query
    let results = create_test_results("long", 1);
    let provider = TestProvider::success("long", results);

    let options = SearchOptions {
        query: long_query,
        provider: Box::new(provider),
        ..Default::default()
    };

    let search_results = web_search(options).await.unwrap();
    assert_eq!(search_results.len(), 1);
}

#[tokio::test]
async fn test_search_with_special_characters() {
    let results = create_test_results("special", 1);
    let provider = TestProvider::success("special", results);

    let options = SearchOptions {
        query: r#"query with "quotes" & <tags> and [brackets] {braces} \backslashes/ & &amp; %20"#
            .to_string(),
        provider: Box::new(provider),
        ..Default::default()
    };

    let search_results = web_search(options).await.unwrap();
    assert_eq!(search_results.len(), 1);
}

#[tokio::test]
async fn test_error_types_comprehensive() {
    let error_cases = vec![
        (
            "http_401",
            SearchError::HttpError {
                status_code: Some(401),
                message: "Unauthorized".to_string(),
                response_body: None,
            },
        ),
        (
            "http_403",
            SearchError::HttpError {
                status_code: Some(403),
                message: "Forbidden".to_string(),
                response_body: None,
            },
        ),
        (
            "http_404",
            SearchError::HttpError {
                status_code: Some(404),
                message: "Not Found".to_string(),
                response_body: None,
            },
        ),
        (
            "http_429",
            SearchError::HttpError {
                status_code: Some(429),
                message: "Too Many Requests".to_string(),
                response_body: None,
            },
        ),
        (
            "http_500",
            SearchError::HttpError {
                status_code: Some(500),
                message: "Internal Server Error".to_string(),
                response_body: None,
            },
        ),
        ("timeout", SearchError::Timeout { timeout_ms: 5000 }),
        (
            "parse_error",
            SearchError::ParseError("Invalid JSON response".to_string()),
        ),
        (
            "other_error",
            SearchError::Other("Custom error message".to_string()),
        ),
    ];

    for (name, error) in error_cases {
        let provider = TestProvider::error(name, error.clone());
        let options = SearchOptions {
            query: "test".to_string(),
            provider: Box::new(provider),
            ..Default::default()
        };

        let result = web_search(options).await;
        assert!(result.is_err(), "Expected error for case: {name}");

        match result.unwrap_err() {
            SearchError::ProviderError(msg) => {
                assert!(
                    msg.contains("failed"),
                    "Error message should mention failure for case: {name}"
                );
            }
            _ => panic!("Expected ProviderError wrapper for case: {name}"),
        }
    }
}

#[tokio::test]
async fn test_edge_case_empty_results() {
    let provider = TestProvider::success("empty", vec![]);

    let options = SearchOptions {
        query: "test".to_string(),
        provider: Box::new(provider),
        ..Default::default()
    };

    let results = web_search(options).await.unwrap();
    assert_eq!(results.len(), 0);
}

#[tokio::test]
async fn test_edge_case_malformed_urls_in_results() {
    let malformed_results = vec![
        SearchResult {
            title: "Valid Result".to_string(),
            url: "https://example.com/valid".to_string(),
            snippet: Some("Valid content".to_string()),
            domain: None,
            published_date: None,
            provider: Some("test".to_string()),
            raw: None,
        },
        SearchResult {
            title: "Invalid URL Result".to_string(),
            url: "not-a-valid-url".to_string(),
            snippet: Some("Invalid URL content".to_string()),
            domain: None,
            published_date: None,
            provider: Some("test".to_string()),
            raw: None,
        },
        SearchResult {
            title: "Empty URL Result".to_string(),
            url: "".to_string(),
            snippet: Some("Empty URL content".to_string()),
            domain: None,
            published_date: None,
            provider: Some("test".to_string()),
            raw: None,
        },
    ];

    let provider = TestProvider::success("malformed", malformed_results);

    let options = SearchOptions {
        query: "test".to_string(),
        provider: Box::new(provider),
        ..Default::default()
    };

    let results = web_search(options).await.unwrap();
    assert_eq!(results.len(), 3); // Should still return all results
    assert_eq!(results[0].url, "https://example.com/valid");
    assert_eq!(results[1].url, "not-a-valid-url");
    assert_eq!(results[2].url, "");
}

#[tokio::test]
async fn test_large_number_of_results() {
    let large_results = create_test_results("large", 1000);
    let provider = TestProvider::success("large", large_results);

    let options = SearchOptions {
        query: "test".to_string(),
        max_results: Some(1000),
        provider: Box::new(provider),
        ..Default::default()
    };

    let results = web_search(options).await.unwrap();
    assert_eq!(results.len(), 1000);

    // Verify first and last results
    assert_eq!(results[0].title, "large Result 1");
    assert_eq!(results[999].title, "large Result 1000");
}

#[tokio::test]
async fn test_memory_usage_with_large_content() {
    // Create results with large content strings
    let large_content_results = vec![SearchResult {
        title: "Large Content Result".to_string(),
        url: "https://example.com/large".to_string(),
        snippet: Some("x".repeat(1_000_000)), // 1MB of content
        domain: None,
        published_date: None,
        provider: Some("large".to_string()),
        raw: None,
    }];

    let provider = TestProvider::success("large", large_content_results);

    let options = SearchOptions {
        query: "test".to_string(),
        provider: Box::new(provider),
        ..Default::default()
    };

    let results = web_search(options).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].snippet.as_ref().unwrap().len(), 1_000_000);
}

#[tokio::test]
async fn test_search_options_validation() {
    let provider = TestProvider::success("test", create_test_results("test", 1));

    // Test with both empty query and no id_list (should fail)
    let invalid_options = SearchOptions {
        query: "".to_string(),
        id_list: None,
        provider: Box::new(provider.clone()),
        ..Default::default()
    };

    let result = web_search(invalid_options).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        SearchError::InvalidInput(msg) => {
            assert!(msg.contains("search query or ID list"));
        }
        _ => panic!("Expected InvalidInput error"),
    }

    // Test with empty query but with id_list (should succeed for arxiv-like providers)
    let valid_options = SearchOptions {
        query: "".to_string(),
        id_list: Some("1234.5678".to_string()),
        provider: Box::new(provider),
        ..Default::default()
    };

    let result = web_search(valid_options).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_debug_logging_does_not_crash() {
    let provider = TestProvider::success("debug", create_test_results("debug", 1));

    let options = SearchOptions {
        query: "debug test".to_string(),
        debug: Some(DebugOptions {
            enabled: true,
            log_requests: false,
            log_responses: false,
        }),
        provider: Box::new(provider),
        ..Default::default()
    };

    // Should not crash even with debug logging enabled
    let results = web_search(options).await.unwrap();
    assert_eq!(results.len(), 1);
}

#[tokio::test]
async fn test_slow_provider() {
    let provider = TestProvider::slow(
        "slow",
        50, // 50ms delay
        TestProviderBehavior::Success(create_test_results("slow", 1)),
    );

    let options = SearchOptions {
        query: "test".to_string(),
        provider: Box::new(provider),
        ..Default::default()
    };

    let results = web_search(options).await.unwrap();
    assert_eq!(results.len(), 1);
}
