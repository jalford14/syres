use std::collections::HashMap;
use reqwest::{Client, header::{HeaderMap, HeaderValue}};
use scraper::{Html, Selector};
use anyhow::{Result, Context};

pub struct SkeddaClient {
    client: Client,
    base_url: String,
}

impl SkeddaClient {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            base_url: "https://switchyards.skedda.com".to_string(),
        })
    }

    /// Fetches the booking page and extracts the CSRF token
    pub async fn get_booking_page(&self) -> Result<String> {
        let url = format!("{}/booking", self.base_url);
        
        let response = self.client
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
        let selectors = [
            "input[name='__RequestVerificationToken']",
            "input[name='X-Skedda-RequestVerificationToken']",
            "meta[name='csrf-token']",
            "meta[name='X-Skedda-RequestVerificationToken']",
        ];

        for selector_str in &selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(element) = document.select(&selector).next() {
                    if let Some(token) = element.value().attr("value") {
                        return Ok(token.to_string());
                    }
                }
            }
        }

        // If not found in input fields, try to find it in the page content
        // Look for patterns like "X-Skedda-RequestVerificationToken: <token>"
        let token_patterns = [
            "X-Skedda-RequestVerificationToken",
            "__RequestVerificationToken",
        ];

        for pattern in &token_patterns {
            if let Some(start) = html_content.find(pattern) {
                let after_pattern = &html_content[start..];
                if let Some(token_start) = after_pattern.find('"') {
                    let token_content = &after_pattern[token_start + 1..];
                    if let Some(token_end) = token_content.find('"') {
                        return Ok(token_content[..token_end].to_string());
                    }
                }
            }
        }

        Err(anyhow::anyhow!("CSRF token not found in HTML content"))
    }

    /// Makes an authenticated GET request with CSRF token
    /// Cookies are automatically handled by reqwest's cookie store
    pub async fn authenticated_get(
        &self, 
        endpoint: &str, 
        csrf_token: &str
    ) -> Result<String> {
        let url = format!("{}{}", self.base_url, endpoint);
        
        // Build headers with CSRF token
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Skedda-RequestVerificationToken",
            HeaderValue::from_str(csrf_token)?
        );

        let response = self.client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .context("Failed to make authenticated request")?;

        let response_text = response
            .text()
            .await
            .context("Failed to get response text")?;

        Ok(response_text)
    }

    /// Makes an authenticated POST request with CSRF token
    /// Cookies are automatically handled by reqwest's cookie store
    pub async fn authenticated_post(
        &self,
        endpoint: &str,
        csrf_token: &str,
        form_data: &HashMap<String, String>
    ) -> Result<String> {
        let url = format!("{}{}", self.base_url, endpoint);
        
        // Build headers with CSRF token
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Skedda-RequestVerificationToken",
            HeaderValue::from_str(csrf_token)?
        );

        let response = self.client
            .post(&url)
            .headers(headers)
            .form(form_data)
            .send()
            .await
            .context("Failed to make authenticated POST request")?;

        let response_text = response
            .text()
            .await
            .context("Failed to get response text")?;

        Ok(response_text)
    }

    /// Makes an authenticated GET request to the /webs endpoint and returns JSON
    /// This endpoint provides booking data for Switchyards locations
    pub async fn get_webs_data(&self, csrf_token: &str) -> Result<serde_json::Value> {
        let url = format!("{}/webs", self.base_url);
        
        // Build headers with CSRF token
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Skedda-RequestVerificationToken",
            HeaderValue::from_str(csrf_token)?
        );
        headers.insert(
            "Accept",
            HeaderValue::from_str("application/json")?
        );
        headers.insert(
            "Referer",
            HeaderValue::from_str(&format!("{}/booking", self.base_url))?
        );

        println!("Making request to: {}", url);
        println!("Headers: {:?}", headers);

        let response = self.client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .context("Failed to make authenticated request to /webs")?;

        println!("Response status: {}", response.status());
        println!("Response headers: {:?}", response.headers());

        let response_json = response
            .json::<serde_json::Value>()
            .await
            .context("Failed to parse JSON response from /webs")?;

        Ok(response_json)
    }

    /// Gets booking data from the /webs endpoint with proper session handling
    /// This method ensures the CSRF token and security cookie are properly synchronized
    pub async fn get_booking_data(&self) -> Result<serde_json::Value> {
        // Step 1: Get the booking page to establish session and get CSRF token
        let booking_url = format!("{}/booking", self.base_url);
        let booking_response = self.client
            .get(&booking_url)
            .send()
            .await
            .context("Failed to fetch booking page")?;

        // Extract all cookies from the response headers first
        let mut all_cookies = Vec::new();
        for (name, value) in booking_response.headers() {
            if name.as_str().to_lowercase() == "set-cookie" {
                let cookie_str = value.to_str().unwrap_or("");
                if let Some(cookie_value) = cookie_str.split(';').next() {
                    all_cookies.push(cookie_value.to_string());
                    println!("Found cookie: {}", cookie_value);
                }
            }
        }

        let html_content = booking_response
            .text()
            .await
            .context("Failed to get booking page text")?;

        // Extract CSRF token
        let csrf_token = self.extract_csrf_token(&html_content)?;
        println!("Extracted CSRF token: {}", csrf_token);

        // Step 2: Make request to /webs with CSRF token and all cookies
        let webs_url = format!("{}/webs", self.base_url);
        
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Skedda-RequestVerificationToken",
            HeaderValue::from_str(&csrf_token)?
        );
        headers.insert(
            "Accept",
            HeaderValue::from_str("application/json")?
        );
        headers.insert(
            "Referer",
            HeaderValue::from_str(&booking_url)?
        );

        // Add all cookies if found
        if !all_cookies.is_empty() {
            let cookie_string = all_cookies.join("; ");
            headers.insert(
                "Cookie",
                HeaderValue::from_str(&cookie_string)?
            );
            println!("Added cookies to request: {}", cookie_string);
        } else {
            println!("Warning: No cookies found!");
        }

        println!("Making request to: {}", webs_url);
        println!("Headers: {:?}", headers);

        let webs_response = self.client
            .get(&webs_url)
            .headers(headers)
            .send()
            .await
            .context("Failed to make request to /webs")?;

        println!("Response status: {}", webs_response.status());
        println!("Response headers: {:?}", webs_response.headers());

        let response_json = webs_response
            .json::<serde_json::Value>()
            .await
            .context("Failed to parse JSON response from /webs")?;

        Ok(response_json)
    }

    /// Gets booking data using a predefined cookie string (for testing)
    pub async fn get_booking_data_with_cookies(&self, cookie_string: &str) -> Result<serde_json::Value> {
        // Step 1: Get the booking page to establish session and get CSRF token
        let booking_url = format!("{}/booking", self.base_url);
        let booking_response = self.client
            .get(&booking_url)
            .send()
            .await
            .context("Failed to fetch booking page")?;

        let html_content = booking_response
            .text()
            .await
            .context("Failed to get booking page text")?;

        // Extract CSRF token
        let csrf_token = self.extract_csrf_token(&html_content)?;
        println!("Extracted CSRF token: {}", csrf_token);

        // Step 2: Make request to /webs with CSRF token and provided cookies
        let webs_url = format!("{}/webs", self.base_url);
        
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Skedda-RequestVerificationToken",
            HeaderValue::from_str(&csrf_token)?
        );
        headers.insert(
            "Accept",
            HeaderValue::from_str("application/json")?
        );
        headers.insert(
            "Referer",
            HeaderValue::from_str(&booking_url)?
        );
        headers.insert(
            "Cookie",
            HeaderValue::from_str(cookie_string)?
        );

        println!("Making request to: {}", webs_url);
        println!("Using provided cookies: {}", cookie_string);

        let webs_response = self.client
            .get(&webs_url)
            .headers(headers)
            .send()
            .await
            .context("Failed to make request to /webs")?;

        println!("Response status: {}", webs_response.status());

        let response_json = webs_response
            .json::<serde_json::Value>()
            .await
            .context("Failed to parse JSON response from /webs")?;

        Ok(response_json)
    }

    /// Gets booking data using the security cookie from the same session as the CSRF token
    pub async fn get_booking_data_synchronized(&self) -> Result<serde_json::Value> {
        // Step 1: Get the booking page to establish session and get CSRF token
        let booking_url = format!("{}/booking", self.base_url);
        let booking_response = self.client
            .get(&booking_url)
            .send()
            .await
            .context("Failed to fetch booking page")?;

        // Extract security cookie from the response headers first
        let mut security_cookie = None;
        for (name, value) in booking_response.headers() {
            if name.as_str().to_lowercase() == "set-cookie" {
                let cookie_str = value.to_str().unwrap_or("");
                if cookie_str.contains("X-Skedda-RequestVerificationCookie=") {
                    if let Some(cookie_value) = cookie_str.split(';').next() {
                        security_cookie = Some(cookie_value.to_string());
                        println!("Found security cookie: {}", cookie_value);
                        break;
                    }
                }
            }
        }

        let html_content = booking_response
            .text()
            .await
            .context("Failed to get booking page text")?;

        // Extract CSRF token
        let csrf_token = self.extract_csrf_token(&html_content)?;
        println!("Extracted CSRF token: {}", csrf_token);

        // Step 2: Make request to /webs with matching CSRF token and security cookie
        let webs_url = format!("{}/webs", self.base_url);
        
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Skedda-RequestVerificationToken",
            HeaderValue::from_str(&csrf_token)?
        );
        headers.insert(
            "Accept",
            HeaderValue::from_str("application/json")?
        );
        headers.insert(
            "Referer",
            HeaderValue::from_str(&booking_url)?
        );

        // Add the security cookie if found
        if let Some(cookie) = security_cookie {
            headers.insert(
                "Cookie",
                HeaderValue::from_str(&cookie)?
            );
            println!("Added matching security cookie: {}", cookie);
        } else {
            println!("Warning: No security cookie found!");
        }

        println!("Making request to: {}", webs_url);
        println!("Headers: {:?}", headers);

        let webs_response = self.client
            .get(&webs_url)
            .headers(headers)
            .send()
            .await
            .context("Failed to make request to /webs")?;

        println!("Response status: {}", webs_response.status());
        println!("Response headers: {:?}", webs_response.headers());

        let response_json = webs_response
            .json::<serde_json::Value>()
            .await
            .context("Failed to parse JSON response from /webs")?;

        Ok(response_json)
    }

    /// Gets the current cookies as a string (for debugging)
    pub async fn get_cookies_debug(&self) -> Result<String> {
        // This is a simplified way to see what cookies are stored
        // In a real implementation, you might want to access the cookie jar directly
        let response = self.client
            .get(&format!("{}/booking", self.base_url))
            .send()
            .await
            .context("Failed to get cookies")?;
        
        let mut debug_info = format!("Response status: {}", response.status());
        
        // Check for set-cookie headers
        if let Some(cookie_header) = response.headers().get("set-cookie") {
            debug_info.push_str(&format!("\nSet-Cookie header: {:?}", cookie_header));
        } else {
            debug_info.push_str("\nNo Set-Cookie header found");
        }
        
        // Check all headers for debugging
        debug_info.push_str(&format!("\nAll headers: {:?}", response.headers()));
        
        Ok(debug_info)
    }

    /// Gets detailed cookie information from the cookie jar
    pub async fn get_detailed_cookies(&self) -> Result<String> {
        // Try to access the cookie jar directly
        let response = self.client
            .get(&format!("{}/booking", self.base_url))
            .send()
            .await
            .context("Failed to get detailed cookies")?;
        
        let mut debug_info = String::new();
        
        // Check response headers
        debug_info.push_str(&format!("Response status: {}\n", response.status()));
        
        for (name, value) in response.headers() {
            if name.as_str().to_lowercase().contains("cookie") {
                debug_info.push_str(&format!("Cookie header {}: {:?}\n", name, value));
            }
        }
        
        // Check for set-cookie headers specifically
        for (name, value) in response.headers() {
            if name.as_str().to_lowercase() == "set-cookie" {
                debug_info.push_str(&format!("Set-Cookie: {:?}\n", value));
            }
        }
        
        Ok(debug_info)
    }
}

/// Example usage of the SkeddaClient
pub async fn example_usage() -> Result<()> {
    println!("Creating Skedda client...");
    let client = SkeddaClient::new()?;
    
    println!("Fetching booking page and extracting CSRF token...");
    let csrf_token = client.get_booking_page().await?;
    
    println!("CSRF Token: {}", csrf_token);
    
    // Example of making an authenticated request
    println!("Making authenticated request...");
    let response = client.authenticated_get("/booking", &csrf_token).await?;
    println!("Response length: {} characters", response.len());
    
    // Debug cookies
    println!("Checking cookies...");
    let cookie_debug = client.get_cookies_debug().await?;
    println!("Cookie debug: {}", cookie_debug);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_booking_page() {
        let client = SkeddaClient::new().unwrap();
        let result = client.get_booking_page().await;
        assert!(result.is_ok());
        
        let token = result.unwrap();
        assert!(!token.is_empty());
        println!("Token: {}", token);
    }

    #[tokio::test]
    async fn test_get_webs_data() {
        let client = SkeddaClient::new().unwrap();
        
        // First get the CSRF token and establish session
        println!("Step 1: Getting CSRF token and establishing session...");
        let csrf_token = client.get_booking_page().await.unwrap();
        assert!(!csrf_token.is_empty());
        println!("CSRF Token: {}", csrf_token);
        
        // Check cookies after getting the token
        println!("Step 1.5: Checking cookies...");
        let cookie_debug = client.get_detailed_cookies().await.unwrap();
        println!("Cookie debug: {}", cookie_debug);
        
        // Wait a moment to ensure session is established
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        // Then test the /webs endpoint with the same client instance
        println!("Step 2: Making request to /webs endpoint...");
        let result = client.get_webs_data(&csrf_token).await;
        assert!(result.is_ok());
        
        let webs_data = result.unwrap();
        println!("Webs data: {}", serde_json::to_string_pretty(&webs_data).unwrap());
        
        // Verify it's valid JSON and has some structure
        assert!(webs_data.is_object() || webs_data.is_array());
    }

    #[tokio::test]
    async fn test_get_booking_data() {
        let client = SkeddaClient::new().unwrap();
        
        println!("Testing get_booking_data method...");
        let result = client.get_booking_data().await;
        assert!(result.is_ok());
        
        let booking_data = result.unwrap();
        println!("Booking data: {}", serde_json::to_string_pretty(&booking_data).unwrap());
        
        // Verify it's valid JSON and has some structure
        assert!(booking_data.is_object() || booking_data.is_array());
    }

    #[tokio::test]
    async fn test_get_booking_data_with_provided_cookies() {
        let client = SkeddaClient::new().unwrap();
        
        // Use the exact cookie string from Chrome console
        let cookie_string = "X-Skedda-RequestVerificationCookie=CfDJ8F_bXW5rHYNLk5J7KH88V5Gg2lJTv4khEqHCEPhD2hFsGddy8gUFZ9BXyvPWmnp1ud0o9FOfZW-LCtV2o0lLvs4sclTo4ZtPiw2Zh-rRrfAYS2ff0WANN7waYi9uPQQu1ezlg5wOT8Oy2q70jlwu2Zc; ai_user=50lQnyEMBluoImgy+BtkGj|2025-07-19T00:20:10.798Z; _gcl_au=1.1.1051100356.1752885172; _ga=GA1.1.1248482734.1752885172; _clck=f1sf03%7C2%7Cfxq%7C0%7C2026; _reb2buid=df71a34d-2774-486e-a8bb-6418a396e892-1752885171972; signals-sdk-user-id=f8415c04-86f5-4267-a9d3-a77e15bd2b4b; _reb2bgeo=%7B%22city%22%3A%22Decatur%22%2C%22country%22%3A%22United%20States%22%2C%22countryCode%22%3A%22US%22%2C%22hosting%22%3Afalse%2C%22isp%22%3A%22AT%26T%20Enterprises%2C%20LLC%22%2C%22lat%22%3A33.7408%2C%22proxy%22%3Afalse%2C%22region%22%3A%22GA%22%2C%22regionName%22%3A%22Georgia%22%2C%22status%22%3A%22success%22%2C%22timezone%22%3A%22America%2FNew_York%22%2C%22zip%22%3A%2230032%22%7D; _reb2bresolve=1; _li_dcdm_c=.skedda.com; _lc2_fpi=8cb92928f695--01k0g1j61my0sbcvk2tx0tgv5j; _lc2_fpi_js=8cb92928f695--01k0g1j61my0sbcvk2tx0tgv5j; _reb2bli=YzBhYjRGNLKQHLZ56QdmM2E1NzRlNjBhNjljNDJlY2MyYmNjMWFmZGM=; _reb2bsha=ZDM1NTg5ZWM3YWVhOTRGNLKQHLZ56QljZTc0YzdlNWI3NzMyYjZiNjY0ZDA2N2ZmNGU0YmIxNmRjNDFmNTliMDc2OGUxMDdmYQ==; _reb2btd=YzBhYjdmMRGNLKQHLZ56Q2E1NzRlNjBhNjljNDJlY2MyYmNjMWFmZGM=; __hstc=182930681.9ad3ea6ea5385c7f07608b3babb93a60.1752885173423.1752885173423.1752885173423.1; hubspotutk=9ad3ea6ea5385c7f07608b3babb93a60; __hssrc=1; _li_ss=CgA; _hjSessionUser_3724443=eyJpZCI6ImQ3NTVjM2JkLWNkZjUtNTNmMS1iOTkzLTIwNTFmOWRiMDlmZSIsImNyZWF0ZWQiOjE3NTI4ODUyMTYwNjgsImV4aXN0aW5nIjpmYWxzZX0=; _hp2_id.2650392129=%7B%22userId%22%3A%221624342213995285%22%2C%22pageviewId%22%3A%224548528993726990%22%2C%22sessionId%22%3A%224647607226084480%22%2C%22identity%22%3Anull%2C%22trackerVersion%22%3A%224.0%22%7D; _reb2bref=https://www.skedda.com/integrations; _uetvid=e78b0ac0643711f0a1454f34977ad412; _ga_PEFFMNLGCY=GS2.1.s1752888096$o2$g0$t1752888096$j60$l0$h0; ai_session=wkInruu5k85jkpr7W6hh/e|1752974297985|1752974948582";
        
        println!("Testing get_booking_data_with_cookies method...");
        let result = client.get_booking_data_with_cookies(cookie_string).await;
        assert!(result.is_ok());
        
        let booking_data = result.unwrap();
        println!("Booking data with provided cookies: {}", serde_json::to_string_pretty(&booking_data).unwrap());
        
        // Verify it's valid JSON and has some structure
        assert!(booking_data.is_object() || booking_data.is_array());
    }
} 