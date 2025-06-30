use animation_replace_roblox::StudioParser;
use dotenv::dotenv;
// use log::{debug, info};

#[tokio::main]
async fn main() {
    env_logger::init();
    dotenv().ok();

    let file_path = shellexpand::tilde("~/Documents/Place1.rbxl");
    let roblox_cookie = std::env::var("ROBLOSECURITY").expect(".ROBLOSECURITY must be set in .env");

    // Build the parser with the roboat client
    let mut parser = match StudioParser::builder()
        .file_path(file_path.as_ref())
        .roblosecurity(roblox_cookie)
        .build()
    {
        Ok(parser) => parser,
        Err(e) => {
            eprintln!("Error loading file: {}", e);
            return;
        }
    };

    let workspace_animations = parser.workspace_animations();
    match workspace_animations.await {
        Ok(animations) => {
            println!("Animations: {:?}", animations);
        }
        Err(e) => {
            eprintln!("Failed to fetch animations: {:?}", e);
        }
    }

    let script_animations = parser.all_animations_in_scripts();

    match script_animations.await {
        Ok(animations) => {
            println!("Animations: {:?}", animations);
        }
        Err(e) => {
            eprintln!("Failed to fetch animations: {:?}", e);
        }
    }
}
