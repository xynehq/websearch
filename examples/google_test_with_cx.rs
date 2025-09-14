//! Google API test with a test Custom Search Engine ID

use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let api_key =
        env::var("GOOGLE_API_KEY").map_err(|_| "GOOGLE_API_KEY environment variable not set")?;

    println!("🔍 Google Custom Search API Test with CX");
    println!("=========================================\n");

    // For testing, I'll use a test CX ID - this might not work but will help us understand the response
    // You can create your own at https://cse.google.com/cse/
    let test_cx = "017576662512468239146:omuauf_lfve"; // This is a common test CX ID (might be invalid)

    println!("📋 Testing with CX ID: {test_cx}");
    println!("-------------------------------------");

    let client = reqwest::Client::new();

    let test_url = format!(
        "https://www.googleapis.com/customsearch/v1?key={api_key}&cx={test_cx}&q=rust programming"
    );

    println!("Testing URL: {}", test_url.replace(&api_key, "***"));

    let response = client.get(&test_url).send().await?;
    let status = response.status();
    let response_text = response.text().await?;

    println!("Response status: {status}");

    if status.is_success() {
        println!("✅ Success! Google API is working");

        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&response_text) {
            if let Some(items) = json_value.get("items") {
                if let Some(items_array) = items.as_array() {
                    println!("Found {} search results", items_array.len());

                    for (i, item) in items_array.iter().take(3).enumerate() {
                        if let (Some(title), Some(link)) = (item.get("title"), item.get("link")) {
                            println!("{}. {} - {}", i + 1, title, link);
                        }
                    }
                }
            }
        }
    } else {
        println!("❌ Request failed");

        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&response_text) {
            println!("Error response:");
            println!("{}", serde_json::to_string_pretty(&json_value)?);

            if let Some(error) = json_value.get("error") {
                if let Some(message) = error.get("message") {
                    println!("\n🔍 Error message: {message}");

                    if message
                        .as_str()
                        .unwrap_or("")
                        .contains("Invalid Custom Search Engine ID")
                    {
                        println!("\n💡 The CX ID is invalid. You need to:");
                        println!("   1. Go to https://cse.google.com/cse/");
                        println!("   2. Click 'Add' to create a new search engine");
                        println!(
                            "   3. Add '*' as the site to search (for searching the whole web)"
                        );
                        println!("   4. Get your CX ID from the search engine setup");
                        println!("   5. Add it to your .env file as GOOGLE_CX=your_cx_id");
                    }
                }
            }
        } else {
            println!("Raw response: {response_text}");
        }
    }

    Ok(())
}
