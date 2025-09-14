//! HTTP utilities for making requests to search APIs

use crate::error::{SearchError, SearchResult};
use reqwest::{Client, Response};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::time::Duration;
use url::Url;

/// HTTP client wrapper with search-specific functionality
#[derive(Debug, Clone)]
pub struct HttpClient {
    client: Client,
    default_timeout: Duration,
}

impl HttpClient {
    /// Create a new HTTP client with default settings
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent("search-sdk-rust/0.0.1")
                .build()
                .expect("Failed to create HTTP client"),
            default_timeout: Duration::from_millis(15000),
        }
    }

    /// Create a new HTTP client with custom timeout
    pub fn with_timeout(timeout_ms: u64) -> Self {
        Self {
            client: Client::builder()
                .user_agent("search-sdk-rust/0.0.1")
                .timeout(Duration::from_millis(timeout_ms))
                .build()
                .expect("Failed to create HTTP client"),
            default_timeout: Duration::from_millis(timeout_ms),
        }
    }

    /// Make a GET request and deserialize the JSON response
    pub async fn get_json<T>(&self, url: &str) -> SearchResult<T>
    where
        T: DeserializeOwned,
    {
        let response = self
            .client
            .get(url)
            .timeout(self.default_timeout)
            .send()
            .await?;

        self.handle_response_json(response).await
    }

    /// Make a GET request with headers and deserialize the JSON response
    pub async fn get_json_with_headers<T>(
        &self,
        url: &str,
        headers: HashMap<String, String>,
    ) -> SearchResult<T>
    where
        T: DeserializeOwned,
    {
        let mut request = self.client.get(url).timeout(self.default_timeout);

        for (key, value) in headers {
            request = request.header(key, value);
        }

        let response = request.send().await?;
        self.handle_response_json(response).await
    }

    /// Make a GET request and return the response as text
    pub async fn get_text(&self, url: &str) -> SearchResult<String> {
        let response = self
            .client
            .get(url)
            .timeout(self.default_timeout)
            .send()
            .await?;

        self.handle_response_text(response).await
    }

    /// Make a GET request with headers and return the response as text
    pub async fn get_text_with_headers(
        &self,
        url: &str,
        headers: HashMap<String, String>,
    ) -> SearchResult<String> {
        let mut request = self.client.get(url).timeout(self.default_timeout);

        for (key, value) in headers {
            request = request.header(key, value);
        }

        let response = request.send().await?;
        self.handle_response_text(response).await
    }

    /// Make a POST request with form data and deserialize the JSON response
    pub async fn post_form_json<T>(
        &self,
        url: &str,
        form_data: HashMap<String, String>,
    ) -> SearchResult<T>
    where
        T: DeserializeOwned,
    {
        let response = self
            .client
            .post(url)
            .timeout(self.default_timeout)
            .form(&form_data)
            .send()
            .await?;

        self.handle_response_json(response).await
    }

    /// Make a POST request with form data and return response as text
    pub async fn post_form_text(
        &self,
        url: &str,
        form_data: HashMap<String, String>,
    ) -> SearchResult<String> {
        let response = self
            .client
            .post(url)
            .timeout(self.default_timeout)
            .form(&form_data)
            .send()
            .await?;

        self.handle_response_text(response).await
    }

    /// Make a POST request with form data, headers and return response as text
    pub async fn post_form_text_with_headers(
        &self,
        url: &str,
        form_data: HashMap<String, String>,
        headers: HashMap<String, String>,
    ) -> SearchResult<String> {
        let mut request = self
            .client
            .post(url)
            .timeout(self.default_timeout)
            .form(&form_data);

        for (key, value) in headers {
            request = request.header(key, value);
        }

        let response = request.send().await?;
        self.handle_response_text(response).await
    }

    /// Handle HTTP response and deserialize as JSON
    async fn handle_response_json<T>(&self, response: Response) -> SearchResult<T>
    where
        T: DeserializeOwned,
    {
        let status = response.status();

        if status.is_success() {
            let json = response.json::<T>().await?;
            Ok(json)
        } else {
            let status_code = status.as_u16();
            let response_body = response.text().await.ok();

            Err(SearchError::HttpError {
                message: format!("Request failed with status: {status}"),
                status_code: Some(status_code),
                response_body,
            })
        }
    }

    /// Handle HTTP response and return as text
    async fn handle_response_text(&self, response: Response) -> SearchResult<String> {
        let status = response.status();

        if status.is_success() {
            let text = response.text().await?;
            Ok(text)
        } else {
            let status_code = status.as_u16();
            let response_body = response.text().await.ok();

            Err(SearchError::HttpError {
                message: format!("Request failed with status: {status}"),
                status_code: Some(status_code),
                response_body,
            })
        }
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Build a URL with query parameters
pub fn build_url(base_url: &str, params: HashMap<String, String>) -> SearchResult<String> {
    let mut url = Url::parse(base_url)?;

    for (key, value) in params {
        url.query_pairs_mut().append_pair(&key, &value);
    }

    Ok(url.to_string())
}

/// Extract domain from a URL
pub fn extract_domain(url: &str) -> Option<String> {
    Url::parse(url)
        .ok()
        .and_then(|parsed| parsed.host_str().map(|host| host.to_string()))
}

/// Normalize text by removing excess whitespace
pub fn normalize_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Normalize URL by ensuring it has a proper scheme
pub fn normalize_url(url: &str) -> String {
    if url.starts_with("//") {
        format!("https:{url}")
    } else if !url.starts_with("http://") && !url.starts_with("https://") {
        format!("https://{url}")
    } else {
        url.to_string()
    }
}
