use animation_replace_roblox::assetdelivery;
#[tokio::main]
async fn main() {
    // Call the basic function and handle the Result
    if let Err(e) = assetdelivery::catalog_search().await {
        eprintln!("Error: {}", e);
    }
}
