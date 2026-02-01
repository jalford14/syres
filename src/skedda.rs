use anyhow::{Context, Result};
use chrono::NaiveTime;
use reqwest::{
    blocking::Client,
    cookie::Jar,
    header::{HeaderMap, HeaderValue},
};
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::sync::Arc;

/// A time slot that is available to book.
#[derive(Debug, Clone)]
pub struct AvailableSlot {
    pub start: String,
    pub end: String,
}

/// A single 15-minute time increment within an available slot.
#[derive(Debug, Clone)]
pub struct TimeIncrement {
    pub time: NaiveTime,
    pub block_index: usize,
}

pub struct Skedda {
    client: Client,
    #[allow(dead_code)] // held to keep the Arc alive for the client
    cookie_jar: Arc<Jar>,
    base_url: String,
    pub venue_space_ids: HashMap<String, String>,
    pub selected_location_space_ids: Vec<String>,
    pub authenticated: bool,
    // Caching
    cached_webs_data: Option<serde_json::Value>,
    cached_bookings: HashMap<String, Vec<serde_json::Value>>,
    cached_csrf_token: Option<String>,
    pub venue_id: Option<String>,
}

impl Skedda {
    pub fn new() -> Result<Self> {
        let jar = Arc::new(Jar::default());
        let client = Client::builder()
            .cookie_provider(jar.clone())
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            cookie_jar: jar,
            base_url: "https://switchyards.skedda.com".to_string(),
            selected_location_space_ids: Vec::new(),
            venue_space_ids: HashMap::new(),
            authenticated: false,
            cached_webs_data: None,
            cached_bookings: HashMap::new(),
            cached_csrf_token: None,
            venue_id: None,
        })
    }

    /// Authenticate with Skedda using a session cookie.
    ///
    /// Sets the X-Skedda-ApplicationCookie on the jar and verifies it
    /// by fetching the booking page.
    pub fn authenticate_with_cookie(&mut self, cookie_value: &str) -> Result<()> {
        let url = self
            .base_url
            .parse::<reqwest::Url>()
            .context("Failed to parse base URL")?;
        self.cookie_jar.add_cookie_str(
            &format!("X-Skedda-ApplicationCookie={cookie_value}"),
            &url,
        );

        // Verify the cookie works by fetching the booking page
        let booking_url = format!("{}/booking", self.base_url);
        let response = self
            .client
            .get(&booking_url)
            .send()
            .context("Failed to verify session cookie")?;

        let final_url = response.url().to_string();
        if final_url.contains("login") {
            anyhow::bail!("Session cookie is invalid or expired");
        }

        if !response.status().is_success() {
            anyhow::bail!(
                "Session cookie verification failed with status {}",
                response.status()
            );
        }

        self.authenticated = true;
        Ok(())
    }

    /// Authenticate with Skedda using username/password.
    ///
    /// Flow:
    /// 1. GET app.skedda.com/account/login — obtains CSRF cookie + token
    /// 2. POST app.skedda.com/logins — sends credentials with CSRF header
    pub fn authenticate(&mut self, username: &str, password: &str) -> Result<()> {
        // Step 1: Fetch login page to get CSRF cookie + token
        let login_page_url = "https://app.skedda.com/account/login";
        let page_response = self
            .client
            .get(login_page_url)
            .send()
            .context("Failed to fetch login page")?;

        let html = page_response
            .text()
            .context("Failed to read login page")?;

        let csrf_token = Self::extract_csrf_token(&html)
            .context("Failed to extract CSRF token from login page")?;

        // Step 2: POST credentials with CSRF token
        let login_url = "https://app.skedda.com/logins";
        let body = serde_json::json!({
            "login": {
                "username": username,
                "password": password,
                "rememberMe": false,
                "arbitraryerrors": null
            }
        });

        let response = self
            .client
            .post(login_url)
            .header("Content-Type", "application/json")
            .header("X-Skedda-RequestVerificationToken", &csrf_token)
            .body(serde_json::to_string(&body)?)
            .send()
            .context("Failed to send login request")?;

        let status = response.status();

        if status.is_success() {
            self.authenticated = true;
            return Ok(());
        }

        let response_text = response.text().context("Failed to read login response")?;

        // Try to parse Skedda error format
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response_text) {
            if let Some(errors) = json["errors"].as_array() {
                let messages: Vec<&str> = errors
                    .iter()
                    .filter_map(|e| e["detail"].as_str())
                    .collect();
                if !messages.is_empty() {
                    anyhow::bail!("{}", messages.join("; "));
                }
            }
        }
        anyhow::bail!("Login failed with status {}", status);
    }

    pub fn get_booking_data(&mut self) -> Result<serde_json::Value> {
        if let Some(ref cached) = self.cached_webs_data {
            return Ok(cached.clone());
        }
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

        self.cached_webs_data = Some(response_json.clone());
        Ok(response_json)
    }

    fn get_booking_page(&mut self) -> Result<String> {
        if let Some(ref token) = self.cached_csrf_token {
            return Ok(token.clone());
        }
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
        self.cached_csrf_token = Some(token.clone());
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

        if let Some(items) = webs_data["spaces"].as_array() {
            for item in items {
                if item.get("spaceIds").is_some() {
                    continue;
                }
                let id = item.get("id").and_then(Self::id_from_value);
                let name = item
                    .get("name")
                    .and_then(serde_json::Value::as_str)
                    .map(String::from);
                if let (Some(id), Some(name)) = (id, name) {
                    venue_space_ids.insert(id, name);
                }
            }
        }

        if let Some(venues) = webs_data["venue"].as_array() {
            if let Some(venue0) = venues.first() {
                if let Some(id) = venue0.get("id").and_then(Self::id_from_value) {
                    self.venue_id = Some(id);
                }
            }
            for venue in venues {
                if let Some(spaces) = venue["spaces"].as_array() {
                    for sp in spaces {
                        let id = sp.get("id").and_then(Self::id_from_value);
                        let name = sp
                            .get("name")
                            .and_then(serde_json::Value::as_str)
                            .map(String::from);
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
        arr.iter().filter_map(Self::id_from_value).collect()
    }

    pub fn fetch_location_space_ids(&mut self, selected_location: &str) -> Vec<String> {
        let webs_data = self.get_booking_data().unwrap();

        let name_to_match = selected_location.replace('-', " ");

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

    /// Fetch bookings for a given date.
    pub fn fetch_bookings(&mut self, date: &str) -> Result<Vec<serde_json::Value>> {
        if let Some(cached) = self.cached_bookings.get(date) {
            return Ok(cached.clone());
        }
        let csrf_token = self.get_booking_page()?;
        let url = format!("{}/bookingslists", self.base_url);

        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Skedda-RequestVerificationToken",
            HeaderValue::from_str(&csrf_token)?,
        );
        headers.insert("Accept", HeaderValue::from_str("application/json")?);

        let start = format!("{date}T00:00:00");
        let end = format!("{date}T23:59:59");

        let response = self
            .client
            .get(&url)
            .headers(headers)
            .query(&[("start", &start), ("end", &end)])
            .send()
            .context("Failed to request bookings")?;

        if !response.status().is_success() {
            anyhow::bail!("Bookings API returned {}", response.status());
        }

        let json: serde_json::Value = response
            .json()
            .context("Failed to parse bookings JSON")?;

        if env::var("SYRES_DEBUG_BOOKINGS").is_ok() {
            if let Ok(s) = serde_json::to_string_pretty(&json) {
                let _ = fs::write("bookings_debug.json", s);
            }
        }

        let bookings = json["bookings"]
            .as_array()
            .cloned()
            .unwrap_or_default();

        self.cached_bookings.insert(date.to_string(), bookings.clone());
        Ok(bookings)
    }

    /// Create a booking via POST /bookings.
    pub fn create_booking(
        &mut self,
        space_id: &str,
        date: &str,
        start_time: &NaiveTime,
        end_time: &NaiveTime,
        title: &str,
    ) -> Result<()> {
        let csrf_token = self.get_booking_page()?;
        let url = format!("{}/bookings", self.base_url);

        let venue_id: i64 = self
            .venue_id
            .as_ref()
            .context("No venue ID available")?
            .parse()
            .context("Invalid venue ID")?;

        let space_id_int: i64 = space_id.parse().context("Invalid space ID")?;

        let start = format!("{}T{}", date, start_time.format("%H:%M:%S"));
        let end = format!("{}T{}", date, end_time.format("%H:%M:%S"));

        let body = serde_json::json!({
            "booking": {
                "start": start,
                "end": end,
                "title": title,
                "venue": venue_id,
                "spaces": [space_id_int],
                "type": 1,
                "price": 0
            }
        });

        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Skedda-RequestVerificationToken",
            HeaderValue::from_str(&csrf_token)?,
        );
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .body(serde_json::to_string(&body)?)
            .send()
            .context("Failed to create booking")?;

        if response.status().is_success() {
            // Invalidate bookings cache for this date
            self.cached_bookings.remove(date);
            return Ok(());
        }

        let response_text = response
            .text()
            .context("Failed to read booking response")?;

        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response_text) {
            if let Some(errors) = json["errors"].as_array() {
                let messages: Vec<&str> = errors
                    .iter()
                    .filter_map(|e| e["detail"].as_str())
                    .collect();
                if !messages.is_empty() {
                    anyhow::bail!("{}", messages.join("; "));
                }
            }
        }
        anyhow::bail!("Booking failed: {}", response_text);
    }

    /// Calculate available time slots for a space by finding gaps in bookings.
    pub fn calculate_availability(
        space_id: &str,
        _date: &str,
        bookings: &[serde_json::Value],
    ) -> Vec<AvailableSlot> {
        let open = NaiveTime::from_hms_opt(6, 0, 0).unwrap();
        let close = NaiveTime::from_hms_opt(22, 0, 0).unwrap();

        // Filter bookings to target space and parse start/end times
        let mut intervals: Vec<(NaiveTime, NaiveTime)> = Vec::new();
        for booking in bookings {
            // Check if booking belongs to this space
            let belongs = if let Some(ids) = booking["spaceIds"].as_array() {
                ids.iter()
                    .any(|v| Self::id_from_value(v).as_deref() == Some(space_id))
            } else if let Some(id) = booking.get("spaceId").and_then(Self::id_from_value) {
                id == space_id
            } else {
                false
            };
            if !belongs {
                continue;
            }

            let start_str = booking["start"].as_str().unwrap_or_default();
            let end_str = booking["end"].as_str().unwrap_or_default();

            // Parse time portion from datetime string (e.g. "2025-01-15T09:00:00")
            let start_time = start_str
                .split('T')
                .nth(1)
                .and_then(|t| NaiveTime::parse_from_str(t, "%H:%M:%S").ok());
            let end_time = end_str
                .split('T')
                .nth(1)
                .and_then(|t| NaiveTime::parse_from_str(t, "%H:%M:%S").ok());

            if let (Some(s), Some(e)) = (start_time, end_time) {
                intervals.push((s, e));
            }
        }

        // Sort by start time
        intervals.sort_by_key(|&(s, _)| s);

        // Merge overlapping intervals
        let mut merged: Vec<(NaiveTime, NaiveTime)> = Vec::new();
        for (s, e) in intervals {
            if let Some(last) = merged.last_mut() {
                if s <= last.1 {
                    last.1 = last.1.max(e);
                    continue;
                }
            }
            merged.push((s, e));
        }

        // Find gaps within operating hours
        let mut slots = Vec::new();
        let mut cursor = open;

        for (s, e) in &merged {
            let s = (*s).max(open);
            let e = (*e).min(close);
            if s > cursor {
                slots.push(AvailableSlot {
                    start: cursor.format("%H:%M").to_string(),
                    end: s.format("%H:%M").to_string(),
                });
            }
            cursor = cursor.max(e);
        }

        if cursor < close {
            slots.push(AvailableSlot {
                start: cursor.format("%H:%M").to_string(),
                end: close.format("%H:%M").to_string(),
            });
        }

        slots
    }
}

/// Break available slots into 15-minute increments, each tagged with its block index.
pub fn generate_time_increments(slots: &[AvailableSlot]) -> Vec<TimeIncrement> {
    let mut increments = Vec::new();
    for (block_index, slot) in slots.iter().enumerate() {
        let start = NaiveTime::parse_from_str(&slot.start, "%H:%M")
            .or_else(|_| NaiveTime::parse_from_str(&slot.start, "%H:%M:%S"))
            .unwrap_or_else(|_| NaiveTime::from_hms_opt(0, 0, 0).unwrap());
        let end = NaiveTime::parse_from_str(&slot.end, "%H:%M")
            .or_else(|_| NaiveTime::parse_from_str(&slot.end, "%H:%M:%S"))
            .unwrap_or_else(|_| NaiveTime::from_hms_opt(0, 0, 0).unwrap());

        let mut cursor = start;
        let fifteen_min = chrono::TimeDelta::minutes(15);
        while cursor + fifteen_min <= end {
            increments.push(TimeIncrement {
                time: cursor,
                block_index,
            });
            cursor += fifteen_min;
        }
    }
    increments
}
