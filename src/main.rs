use animation_replace_roblox::animation_uploader;
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();

    match animation_uploader::reupload_animation().await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Failed uploading animation {:?}", e)
        }
    }
}
