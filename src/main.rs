use animation_replace_roblox::{StudioParser, animation};
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let file_path = shellexpand::tilde("~/Documents/Place1.rbxl");

    let roblox_cookie = std::env::var("ROBLOSECURITY").expect(".ROBLOSECURITY must be set in .env");

    // Build the parser with the roboat client
    let parser = match StudioParser::builder()
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

    let animations = parser.workspace_animations();

    let scripts = parser.all_scripts();
    for script in scripts {
        match parser.animations_in_script(&script).await {
            Ok(animations) => {
                println!("{:?}", animations);
            }
            Err(e) => {
                eprintln!("Failed to get workspace animations: {:?}", e);
            }
        };
        // for animation_id in script_animations {
        //     println!("{:?}", animation_id);
        // }
    }

    // match parser.reupload_animation().await {
    //     Ok(_) => {}
    //     Err(e) => {
    //         eprintln!("Failed uploading animation {:?}", e)
    //     }
    // }
}
