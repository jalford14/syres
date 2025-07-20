use syres::http_client::SkeddaClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Testing Skedda HTTP Client...");
    
    // Create client
    let client = SkeddaClient::new()?;
    println!("✓ Client created successfully");
    
    // Get CSRF token
    println!("Fetching booking page and extracting CSRF token...");
    let csrf_token = client.get_booking_page().await?;
    println!("✓ CSRF Token: {}", csrf_token);
    
    // Make authenticated request
    println!("Making authenticated request...");
    let response = client.authenticated_get("/booking", &csrf_token).await?;
    println!("✓ Response length: {} characters", response.len());
    
    // Debug cookies
    println!("Checking cookies...");
    let cookie_debug = client.get_cookies_debug().await?;
    println!("✓ Cookie debug: {}", cookie_debug);
    
    println!("Test completed successfully!");
    Ok(())
} 