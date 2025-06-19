use animation_replace_roblox::{StudioParser, animation_uploader};
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let parser = match StudioParser::from_rbxl("test.rbxl".to_string()) {
        Ok(parser) => parser,
        Err(e) => {
            eprintln!("Error loading file: {}", e);
            return;
        }
    };

    parser.workspace_animations();

    match animation_uploader::reupload_animation().await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Failed uploading animation {:?}", e)
        }
    }
}
