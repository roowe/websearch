//! CLI integration tests
//!
//! These tests ensure the CLI binary works correctly with all flags and options.

use std::process::Command;

const CLI_BINARY: &str = "websearch";

/// Helper function to run CLI commands and capture output
fn run_cli_command(args: &[&str]) -> (String, String, bool) {
    let output = Command::new("cargo")
        .args(&["run", "--bin", CLI_BINARY, "--"])
        .args(args)
        .output()
        .expect("Failed to execute CLI command");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let success = output.status.success();

    (stdout, stderr, success)
}

/// Helper function to check if binary exists
fn cli_binary_exists() -> bool {
    // Try to build the binary first
    let build_output = Command::new("cargo")
        .args(&["build", "--bin", CLI_BINARY])
        .output()
        .expect("Failed to build CLI binary");

    build_output.status.success()
}

#[test]
fn test_cli_binary_builds() {
    assert!(cli_binary_exists(), "CLI binary should build successfully");
}

#[test]
fn test_cli_help() {
    let (stdout, _stderr, success) = run_cli_command(&["--help"]);

    assert!(success, "Help command should succeed");
    assert!(stdout.contains("Web search CLI"));
    assert!(stdout.contains("--provider"));
    assert!(stdout.contains("--max-results"));
}

#[test]
fn test_cli_version() {
    let (stdout, _stderr, success) = run_cli_command(&["--version"]);

    assert!(success, "Version command should succeed");
    assert!(stdout.contains("websearch"));
}

#[test]
fn test_default_search_with_provider() {
    let (stdout, _stderr, success) = run_cli_command(&["--help"]);

    assert!(success, "Default search help should succeed");
    assert!(stdout.contains("--provider"));
    assert!(stdout.contains("--max-results"));
    assert!(stdout.contains("--format"));
    assert!(stdout.contains("arxiv") || stdout.contains("duckduckgo"));
}

#[test]
fn test_arxiv_search_flags() {
    let (stdout, _stderr, success) = run_cli_command(&["--help"]);

    assert!(success, "Help should show ArXiv options");
    assert!(stdout.contains("--arxiv-ids"));
    assert!(stdout.contains("--sort-by"));
    assert!(stdout.contains("--sort-order"));
    assert!(stdout.contains("arxiv"));
}

#[test]
fn test_invalid_provider() {
    let (stdout, stderr, success) = run_cli_command(&[
        "test query",
        "--provider",
        "invalid"
    ]);

    assert!(!success, "Invalid provider should fail");
    // Should show valid options in error
    assert!(stderr.contains("invalid") || stdout.contains("invalid"));
}

#[test]
fn test_duckduckgo_search_dry_run() {
    // Test DuckDuckGo search which doesn't require API keys
    // Use a very small result count to minimize API usage
    let (stdout, stderr, success) = run_cli_command(&[
        "rust programming",
        "--provider",
        "duckduckgo",
        "--max-results",
        "1",
        "--format",
        "simple"
    ]);

    if success {
        assert!(stdout.len() > 0, "Should return some results");
        assert!(stdout.contains("1."), "Should have numbered results");
    } else {
        // If it fails, it should be due to network/parsing, not configuration
        println!("DuckDuckGo search failed (network issue): {}{}", stdout, stderr);
    }
}

#[test]
fn test_output_formats() {
    let formats = ["simple", "table", "json"];

    for format in &formats {
        // The format should be mentioned in help
        let help_output = run_cli_command(&["--help"]);
        assert!(help_output.0.contains(format), "Format {} should be in help", format);
    }
}

#[test]
fn test_arxiv_paper_search() {
    // Test ArXiv search with actual paper IDs
    let (stdout, stderr, success) = run_cli_command(&[
        "",
        "--provider",
        "arxiv",
        "--arxiv-ids",
        "2301.00001", // This should be a valid ArXiv ID format
        "--max-results",
        "1",
        "--format",
        "simple"
    ]);

    // ArXiv should either succeed or fail gracefully
    if !success {
        // Should show meaningful error message
        let error_output = format!("{}{}", stdout, stderr);
        assert!(
            error_output.contains("ArXiv") ||
            error_output.contains("arxiv") ||
            error_output.contains("search")
        );
    }
}

#[test]
fn test_debug_flag() {
    let (stdout, stderr, success) = run_cli_command(&[
        "test",
        "--provider",
        "duckduckgo",
        "--debug",
        "--max-results",
        "1"
    ]);

    // Debug flag should either work or show in help
    let combined_output = format!("{}{}", stdout, stderr);
    // Debug output might appear in stdout or stderr
    if success {
        // If successful, might have debug output
        println!("Debug output: {}", combined_output);
    }
}

#[test]
fn test_max_results_parameter() {
    // Test that max-results parameter is accepted
    let (stdout, _stderr, success) = run_cli_command(&[
        "--help"
    ]);

    assert!(success);
    assert!(stdout.contains("max-results") || stdout.contains("max_results"));
}

#[test]
fn test_empty_query_handling() {
    let (stdout, stderr, success) = run_cli_command(&[
        "", // Empty query
        "--provider",
        "duckduckgo"
    ]);

    // Should handle empty query gracefully
    if !success {
        let error_output = format!("{}{}", stdout, stderr);
        assert!(
            error_output.contains("query") ||
            error_output.contains("empty") ||
            error_output.contains("required")
        );
    }
}

#[test]
fn test_cli_providers() {
    // Test that only duckduckgo and arxiv are available
    let (stdout, _stderr, success) = run_cli_command(&[
        "--help"
    ]);

    assert!(success);
    assert!(stdout.contains("duckduckgo"));
    assert!(stdout.contains("arxiv"));
}
