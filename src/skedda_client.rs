use anyhow::{Context, Result};
use reqwest::{
    Client,
    header::{HeaderMap, HeaderValue},
};
use scraper::{Html, Selector};

pub struct SkeddaClient {
    client: Client,
    base_url: String,
}

impl SkeddaClient {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .cookie_store(true)
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            base_url: "https://switchyards.skedda.com".to_string(),
        })
    }

    /// Fetches the booking page and extracts the CSRF token
    /// This establishes the session and cookies automatically via reqwest's cookie store
    pub async fn get_booking_page(&self) -> Result<String> {
        let url = format!("{}/booking", self.base_url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch booking page")?;

        let html_content = response
            .text()
            .await
            .context("Failed to get response text")?;

        // Parse HTML to find the CSRF token
        let token = self.extract_csrf_token(&html_content)?;

        Ok(token)
    }

    /// Extracts the CSRF token from HTML content
    fn extract_csrf_token(&self, html_content: &str) -> Result<String> {
        let document = Html::parse_document(html_content);

        // Try multiple selectors to find the CSRF token
        let selectors = ["input[name='__RequestVerificationToken']"];

        for selector_str in &selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(element) = document.select(&selector).next() {
                    if let Some(token) = element.value().attr("value") {
                        return Ok(token.to_string());
                    }
                }
            }
        }

        Err(anyhow::anyhow!("CSRF token not found in HTML content"))
    }

    // TODO: Need to auth and then add the cookies to the jar
    pub async fn get_booking_data(&self) -> Result<serde_json::Value> {
        // Step 1: Get the booking page to establish session and get CSRF token
        let csrf_token = self.get_booking_page().await?;
        println!("Extracted CSRF token: {}", csrf_token);

        // Step 2: Make request to /webs with CSRF token
        // Cookies are automatically included by reqwest's cookie store
        let url = format!("{}/webs", self.base_url);

        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Skedda-RequestVerificationToken",
            HeaderValue::from_str(&csrf_token)?,
        );
        headers.insert("Accept", HeaderValue::from_str("application/json")?);

        println!("Making request to: {}", url);

        let response = self
            .client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .context("Failed to make request to /webs")?;

        println!("Response status: {}", response.status());

        let response_json = response
            .json::<serde_json::Value>()
            .await
            .context("Failed to parse JSON response from /webs")?;

        Ok(response_json)
    }
}
