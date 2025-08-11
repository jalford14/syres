use anyhow::{Context, Result};
use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderValue},
};
use scraper::{Html, Selector};
use std::collections::HashMap;

pub struct Skedda {
    client: Client,
    base_url: String,
    pub venue_space_ids: HashMap<String, String>,
    pub selected_location_space_ids: Vec<String>,
}

impl Skedda {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .cookie_store(true)
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            base_url: "https://switchyards.skedda.com".to_string(),
            selected_location_space_ids: Vec::new(),
            venue_space_ids: HashMap::new(),
        })
    }

    // TODO: Need to auth and then add the cookies to the jar
    pub fn get_booking_data(&self) -> Result<serde_json::Value> {
        let csrf_token = self.get_booking_page()?;
        let url = format!("{}/webs", self.base_url);
        let mut headers = HeaderMap::new();

        headers.insert(
            "X-Skedda-RequestVerificationToken",
            HeaderValue::from_str(&csrf_token)?,
        );
        headers.insert("Accept", HeaderValue::from_str("application/json")?);

        let response = self
            .client
            .get(&url)
            .headers(headers)
            .send()
            .context("Failed to make request to /webs")?;

        let response_json = response
            .json::<serde_json::Value>()
            .context("Failed to parse JSON response from /webs")?;

        Ok(response_json)
    }

    fn get_booking_page(&self) -> Result<String> {
        let url = format!("{}/booking", self.base_url);
        let response = self
            .client
            .get(&url)
            .send()
            .context("Failed to fetch booking page")?;

        let html_content = response
            .text()
            .context("Failed to get response text")?;

        let token = Skedda::extract_csrf_token(&html_content)?;
        Ok(token)
    }

    fn extract_csrf_token(html_content: &str) -> Result<String> {
        let document = Html::parse_document(html_content);
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

    pub fn fetch_space_ids(&self) -> HashMap<String, String> {
        let mut venue_space_ids = HashMap::new();
        let webs_data = self.get_booking_data().unwrap();

        if let serde_json::Value::Array(items) = &webs_data["spaces"] {
            for item in items {
                if let (Some(id), Some(name)) = (
                    item.get("id").and_then(serde_json::Value::as_str),
                    item.get("name").and_then(serde_json::Value::as_str),
                ) {
                    venue_space_ids.insert(id.to_string(), name.to_string());
                }
            }
        }

        return venue_space_ids;
    }

    pub fn fetch_location_space_ids(&self, selected_location: &str) -> Vec<String> {
        let webs_data = self.get_booking_data();
        let webs_response = &webs_data["venue"][0]["spacePresentation"]["spaceTags"];

        if let serde_json::Value::Array(items) = &webs_data["spaces"] {
            for item in items {
                if let serde_json::Value::Object(obj) = item {
                    if selected_location
                    == obj.unwrap()
                        .get("name")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                    {
                        if let Some(serde_json::Value::Array(space_ids)) = obj.get("spaceIds") {
                            return space_ids
                                .iter()
                                .filter_map(|v| v.as_i64())
                                .map(|n| n.to_string())
                                .collect();
                        }
                    }
                }
            }
        }
    }
}
