//! Google API setup test to check API key and discover requirements

use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let api_key =
        env::var("GOOGLE_API_KEY").map_err(|_| "GOOGLE_API_KEY environment variable not set")?;

    println!("🔍 Google Custom Search API Setup Test");
    println!("======================================\n");

    // Test 1: Check API key validity with a simple request (this should fail without CX)
    println!("📋 Test 1: Checking API Key validity");
    println!("------------------------------------");

    let client = reqwest::Client::new();

    // Try the API without CX to see what error we get
    let test_url = format!(
        "https://www.googleapis.com/customsearch/v1?key={api_key}&q=test"
    );

    println!("Testing URL: {}", test_url.replace(&api_key, "***"));

    let response = client.get(&test_url).send().await?;
    let status = response.status();
    let response_text = response.text().await?;

    println!("Response status: {status}");
    println!("Response body: {response_text}");

    // Try to parse the error for helpful information
    if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&response_text) {
        println!("Parsed JSON:");
        println!("{}", serde_json::to_string_pretty(&json_value)?);

        if let Some(error) = json_value.get("error") {
            if let Some(message) = error.get("message") {
                println!("\n🔍 Error message: {message}");
            }
            if let Some(errors) = error.get("errors") {
                println!("🔍 Error details: {errors}");
            }
        }
    }

    println!("\n💡 Next steps:");
    println!("  1. If you see 'Custom search API requires a valid Custom Search Engine ID'");
    println!("     - Go to: https://cse.google.com/cse/");
    println!("     - Create a new Custom Search Engine");
    println!("     - Add the CX ID to your .env file as GOOGLE_CX=your_cx_id");
    println!("  2. If you see authentication errors, check your API key");
    println!("  3. Make sure the Custom Search API is enabled in Google Cloud Console");

    Ok(())
}
