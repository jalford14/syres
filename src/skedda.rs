use anyhow::{Context, Result};
use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderValue},
};
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::env;
use std::fs;

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

        // Dump /webs response for debugging when SYRES_DEBUG_WEBS=1
        if env::var("SYRES_DEBUG_WEBS").is_ok() {
            if let Ok(s) = serde_json::to_string_pretty(&response_json) {
                let _ = fs::write("webs_debug.json", s);
            }
        }

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

    /// Extract a string ID from a JSON value (handles both string and number).
    fn id_from_value(v: &serde_json::Value) -> Option<String> {
        v.as_str()
            .map(String::from)
            .or_else(|| v.as_i64().map(|n| n.to_string()))
    }

    pub fn fetch_space_ids(&mut self) -> HashMap<String, String> {
        let mut venue_space_ids = HashMap::new();
        let webs_data = self.get_booking_data().unwrap();

        // Top-level "spaces" array: add only leaf spaces (skip items with "spaceIds",
        // which are venue/parent objects). IDs can be string or number in JSON.
        if let Some(items) = webs_data["spaces"].as_array() {
            for item in items {
                if item.get("spaceIds").is_some() {
                    continue; // venue/parent, not a bookable space
                }
                let id = item.get("id").and_then(Self::id_from_value);
                let name = item.get("name").and_then(serde_json::Value::as_str).map(String::from);
                if let (Some(id), Some(name)) = (id, name) {
                    venue_space_ids.insert(id, name);
                }
            }
        }

        // Venue-specific "spaces": some APIs nest spaces under venue["spaces"].
        if let Some(venues) = webs_data["venue"].as_array() {
            for venue in venues {
                if let Some(spaces) = venue["spaces"].as_array() {
                    for sp in spaces {
                        let id = sp.get("id").and_then(Self::id_from_value);
                        let name = sp.get("name").and_then(serde_json::Value::as_str).map(String::from);
                        if let (Some(id), Some(name)) = (id, name) {
                            venue_space_ids.insert(id, name);
                        }
                    }
                }
            }
        }

        self.venue_space_ids = venue_space_ids.clone();
        venue_space_ids
    }

    /// Extract space IDs from a JSON array (handles both numeric and string IDs).
    fn space_ids_from_array(arr: &[serde_json::Value]) -> Vec<String> {
        arr.iter()
            .filter_map(Self::id_from_value)
            .collect()
    }

    pub fn fetch_location_space_ids(&self, selected_location: &str) -> Vec<String> {
        let webs_data = self.get_booking_data().unwrap();

        // Normalize "Virginia-Highland" -> "Virginia Highland" to match Skedda's spaceTags.
        let name_to_match = selected_location.replace('-', " ");

        // 1) Primary: venue[0].spacePresentation.spaceTags — each tag has "name" (location)
        //    and "spaceIds" (array of space IDs). This is the canonical structure for
        //    Switchyards/Skedda.
        if let Some(venues) = webs_data["venue"].as_array() {
            if let Some(venue0) = venues.first() {
                if let Some(space_tags) = venue0["spacePresentation"]["spaceTags"].as_array() {
                    for tag in space_tags {
                        if tag["name"].as_str() == Some(name_to_match.as_str()) {
                            if let Some(ids) = tag["spaceIds"].as_array() {
                                return Self::space_ids_from_array(ids);
                            }
                        }
                    }
                }
            }
        }

        // 2) Fallback: venue array with venue["name"] match, or venue["spaceIds"] / venue["spaces"].
        if let Some(venues) = webs_data["venue"].as_array() {
            for venue in venues {
                if venue["name"].as_str() != Some(selected_location) {
                    continue;
                }
                if let Some(ids) = venue["spaceIds"].as_array() {
                    return Self::space_ids_from_array(ids);
                }
                if let Some(spaces) = venue["spaces"].as_array() {
                    let ids: Vec<String> = spaces
                        .iter()
                        .filter_map(|s| {
                            s.get("id")
                                .and_then(Self::id_from_value)
                                .or_else(|| Self::id_from_value(s))
                        })
                        .collect();
                    if !ids.is_empty() {
                        return ids;
                    }
                }
            }
        }

        // 3) Fallback: top-level "spaces" array, item with name == location and "spaceIds".
        if let Some(items) = webs_data["spaces"].as_array() {
            for item in items {
                if item["name"].as_str() != Some(selected_location) {
                    continue;
                }
                if let Some(ids) = item["spaceIds"].as_array() {
                    return Self::space_ids_from_array(ids);
                }
            }
        }

        Vec::new()
    }
}
