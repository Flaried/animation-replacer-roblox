use animation_replace_roblox::roblox_api::roblox_client::SessionBuilder;
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let roblox_cookie = std::env::var("ROBLOSECURITY").expect(".ROBLOSECURITY must be set in .env");

    let session = SessionBuilder::new()
        .roblosecurity(roblox_cookie.to_string())
        .build()
        .expect("Failed to build Roblox session");

    let result = session.refresh_csrf().await;
    match result {
        Ok(test) => println!("Refreshed XCSRF {:?}", test),
        Err(e) => eprintln!("Failed to refresh xCSRF {}", e),
    }

    match session.pull_animation(128511411359897).await {
        Ok(body) => println!("hi"),
        Err(e) => eprintln!("Failed to fetch animation: {}", e),
    }
}
