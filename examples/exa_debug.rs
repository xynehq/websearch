//! Debug Exa API response structure

use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let api_key =
        env::var("EXA_API_KEY").map_err(|_| "EXA_API_KEY environment variable not set")?;

    let client = reqwest::Client::new();

    // Create a simple request to see the response structure
    let request_body = serde_json::json!({
        "query": "test query",
        "max_results": 1,
        "model": "keyword",
        "include_contents": false
    });

    println!("Sending request to Exa API...");
    println!(
        "Request body: {}",
        serde_json::to_string_pretty(&request_body)?
    );

    let response = client
        .post("https://api.exa.ai/search")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {api_key}"))
        .json(&request_body)
        .send()
        .await?;

    let status = response.status();
    println!("Response status: {status}");

    let response_text = response.text().await?;
    println!("Raw response: {response_text}");

    // Try to pretty-print the JSON
    if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&response_text) {
        println!("Parsed JSON:");
        println!("{}", serde_json::to_string_pretty(&json_value)?);
    } else {
        println!("Failed to parse response as JSON");
    }

    Ok(())
}
