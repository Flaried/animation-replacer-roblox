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

    match session.pull_animation(123141414).await {
        Ok(body) => println!("{}", body),
        Err(e) => eprintln!("Failed to fetch animation: {}", e),
    }
}
