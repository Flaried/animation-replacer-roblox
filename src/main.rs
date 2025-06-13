use animation_replace_roblox::roblox_api::{
    self, assetdelivery,
    roblox_client::{RobloxSession, SessionBuilder},
};
use reqwest::Client;
const ROBLOSECURITY: &str = "your-roblosecurity-token";

#[tokio::main]
async fn main() {
    let session = SessionBuilder::new()
        .roblosecurity("your_roblosecurity_token_here".to_string())
        .build()
        .expect("Failed to build Roblox session");

    match session.pull_animation(123141414).await {
        Ok(body) => println!("{}", body),
        Err(e) => eprintln!("Failed to fetch animation: {}", e),
    }
}
