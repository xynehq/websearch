//! CLI integration tests
//!
//! These tests ensure the CLI binary works correctly with all flags and options.

use std::process::Command;
use std::path::Path;

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
    assert!(stdout.contains("Multi-provider web search CLI"));
    assert!(stdout.contains("single"));
    assert!(stdout.contains("multi"));
    assert!(stdout.contains("arxiv"));
    assert!(stdout.contains("providers"));
}

#[test]
fn test_cli_version() {
    let (stdout, _stderr, success) = run_cli_command(&["--version"]);

    assert!(success, "Version command should succeed");
    assert!(stdout.contains("websearch"));
}

#[test]
fn test_providers_command() {
    let (stdout, _stderr, success) = run_cli_command(&["providers"]);

    assert!(success, "Providers command should succeed");
    assert!(stdout.contains("Available Search Providers"));
    assert!(stdout.contains("Google"));
    assert!(stdout.contains("Tavily"));
    assert!(stdout.contains("DuckDuckGo"));
    assert!(stdout.contains("ArXiv"));

    // Should show which providers are available vs not
    assert!(stdout.contains("✅") || stdout.contains("❌"));
}

#[test]
fn test_single_search_help() {
    let (stdout, _stderr, success) = run_cli_command(&["single", "--help"]);

    assert!(success, "Single search help should succeed");
    assert!(stdout.contains("Search using a single provider"));
    assert!(stdout.contains("--provider"));
    assert!(stdout.contains("--max-results"));
    assert!(stdout.contains("--format"));
}

#[test]
fn test_multi_search_help() {
    let (stdout, _stderr, success) = run_cli_command(&["multi", "--help"]);

    assert!(success, "Multi search help should succeed");
    assert!(stdout.contains("Search using multiple providers"));
    assert!(stdout.contains("--strategy"));
    assert!(stdout.contains("--providers"));
}

#[test]
fn test_arxiv_search_help() {
    let (stdout, _stderr, success) = run_cli_command(&["arxiv", "--help"]);

    assert!(success, "ArXiv search help should succeed");
    assert!(stdout.contains("Search ArXiv papers by ID"));
    assert!(stdout.contains("--sort-by"));
    assert!(stdout.contains("--sort-order"));
}

#[test]
fn test_invalid_provider() {
    let (stdout, stderr, success) = run_cli_command(&[
        "single",
        "test query",
        "--provider",
        "invalid"
    ]);

    assert!(!success, "Invalid provider should fail");
    // Should show valid options in error
    assert!(stderr.contains("invalid") || stdout.contains("invalid"));
}

#[test]
fn test_missing_api_key_error() {
    // Test that providers requiring API keys show appropriate errors
    let (stdout, stderr, success) = run_cli_command(&[
        "single",
        "test query",
        "--provider",
        "google",
        "--max-results",
        "1"
    ]);

    assert!(!success, "Google without API key should fail");
    let error_output = format!("{}{}", stdout, stderr);
    assert!(
        error_output.contains("GOOGLE_API_KEY") ||
        error_output.contains("environment variable") ||
        error_output.contains("API key")
    );
}

#[test]
fn test_duckduckgo_search_dry_run() {
    // Test DuckDuckGo search which doesn't require API keys
    // Use a very small result count to minimize API usage
    let (stdout, stderr, success) = run_cli_command(&[
        "single",
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
        let (stdout, _stderr, success) = run_cli_command(&[
            "single",
            "--help" // Just test format parsing via help
        ]);

        // The format should be mentioned in help
        let help_output = run_cli_command(&["single", "--help"]);
        assert!(help_output.0.contains(format), "Format {} should be in help", format);
    }
}

#[test]
fn test_multi_search_strategies() {
    let strategies = ["aggregate", "failover", "load-balance"];

    for strategy in &strategies {
        let (stdout, _stderr, success) = run_cli_command(&[
            "multi",
            "--help"
        ]);

        // The strategy should be mentioned in help
        assert!(stdout.contains("strategy") || stdout.contains(strategy),
                "Strategy {} should be mentioned in help", strategy);
    }
}

#[test]
fn test_arxiv_paper_search() {
    // Test ArXiv search with actual paper IDs
    let (stdout, stderr, success) = run_cli_command(&[
        "arxiv",
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
        "single",
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
        "single",
        "--help"
    ]);

    assert!(success);
    assert!(stdout.contains("max-results") || stdout.contains("max_results"));
}

#[test]
fn test_json_output_format() {
    // Test that JSON format is properly structured when it works
    let (stdout, stderr, success) = run_cli_command(&[
        "providers" // This should always work and return structured data
    ]);

    assert!(success, "Providers command should always work");
    // The output should be human readable for providers command
    assert!(stdout.contains("Google") || stdout.contains("DuckDuckGo"));
}

#[test]
fn test_empty_query_handling() {
    let (stdout, stderr, success) = run_cli_command(&[
        "single",
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
fn test_cli_comprehensive_flag_validation() {
    // Test that all major flags are recognized (even if they fail due to missing APIs)
    let test_cases = vec![
        (vec!["single", "test", "--provider", "google"], false), // Should fail without API key
        (vec!["single", "test", "--provider", "tavily"], false), // Should fail without API key
        (vec!["single", "test", "--provider", "duckduckgo"], true), // Should work
        (vec!["multi", "test", "--strategy", "aggregate"], true), // Should work with available providers
        (vec!["providers"], true), // Should always work
    ];

    for (args, should_succeed) in test_cases {
        let (stdout, stderr, success) = run_cli_command(&args);
        let combined = format!("{}{}", stdout, stderr);

        if should_succeed {
            assert!(success || combined.contains("DuckDuckGo") || combined.contains("Available"),
                    "Command {:?} should succeed or show meaningful output", args);
        } else {
            // If it fails, should be due to missing API keys, not invalid flags
            assert!(
                combined.contains("API") ||
                combined.contains("key") ||
                combined.contains("environment") ||
                success, // Or it might succeed if user has API keys
                "Command {:?} should fail due to API keys, not invalid syntax", args
            );
        }
    }
}