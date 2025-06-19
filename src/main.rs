use animation_replace_roblox::{StudioParser, animation_uploader};
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let file_path = shellexpand::tilde("~/Documents/Place1.rbxl");

    let parser = match StudioParser::from_rbxl(file_path.as_ref()) {
        Ok(parser) => parser,
        Err(e) => {
            eprintln!("Error loading file: {}", e);
            return;
        }
    };

    let animations = parser.workspace_animations();
    let scripts = parser.animations_in_scripts();

    match animation_uploader::reupload_animation().await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Failed uploading animation {:?}", e)
        }
    }
}
