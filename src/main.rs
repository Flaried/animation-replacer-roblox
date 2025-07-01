use std::sync::Arc;

use animation_replace_roblox::StudioParser;
use animation_replace_roblox::animation::uploader::AnimationUploader;
use clap::Parser;
use roboat::assetdelivery::request_types::AssetBatchResponse;

#[derive(Parser, Debug)]
struct Args {
    /// .ROBLOSECURITY cookie string [WARNING STRING REQUIRED]
    #[arg(long, short)]
    cookie: String,

    /// file PATH of the .rbxl file [REQUIRED]
    #[arg(long, short)]
    file: String,

    /// Save the copy instead replacing file [AVOID DATA LOSS]
    #[arg(long, short)]
    output: Option<String>,

    /// Required if the the game will be published to a Group [Id of the group]
    #[arg(long, short)]
    group: Option<u64>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let file_path = shellexpand::tilde(&args.file).to_string();

    // Build the parser with the roboat client
    let builder = StudioParser::builder()
        .file_path(file_path)
        .roblosecurity(&args.cookie);

    let mut parser = match builder.build() {
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

    let uploader = Arc::new(AnimationUploader::new(args.cookie));
    match uploader
        .reupload_all_animations(all_animations, args.group.clone())
        .await
    {
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
