use std::sync::Arc;

use animation_replace_roblox::StudioParser;
use animation_replace_roblox::animation::uploader::AnimationUploader;
use dotenv::dotenv;
use roboat::assetdelivery::request_types::AssetBatchResponse;
// use log::{debug, info};

#[tokio::main]
async fn main() {
    dotenv().ok();

    let file_path = shellexpand::tilde("~/Documents/Place1.rbxl");
    let roblox_cookie = std::env::var("ROBLOSECURITY").expect(".ROBLOSECURITY must be set in .env");

    // Build the parser with the roboat client
    let mut parser = match StudioParser::builder()
        .file_path(file_path.as_ref())
        .roblosecurity(&roblox_cookie)
        .build()
    {
        Ok(parser) => parser,
        Err(e) => {
            eprintln!("Error loading file: {}", e);
            return;
        }
    };

    let mut all_animations: Vec<AssetBatchResponse> = Vec::new();
    let workspace_animations = parser.workspace_animations();
    match workspace_animations.await {
        Ok(mut animations) => {
            println!("Animations: {:?}", animations);
            all_animations.append(&mut animations);
        }
        Err(e) => {
            eprintln!("Failed to workspace animations: {:?}", e);
        }
    }

    let script_animations = parser.all_animations_in_scripts();

    match script_animations.await {
        Ok(mut animations) => {
            println!("Animations: {:?}", animations);
            all_animations.append(&mut animations);
        }
        Err(e) => {
            eprintln!("Failed to fetch animations: {:?}", e);
        }
    }

    let uploader = Arc::new(AnimationUploader::new(roblox_cookie));

    match uploader.reupload_all_animations(all_animations).await {
        Ok(animation_mapping) => {
            if let Err(e) = parser.update_script_animations(&animation_mapping) {
                eprintln!("Failed to update script animations: {:?}", e);
            }
        }
        Err(e) => {
            eprintln!("Failed to upload animations: {:?}", e);
        }
    }

    parser.save_to_rbxl("~/Documents/Place2.rbxl").unwrap();
}
