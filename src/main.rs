use animation_replace_roblox::{StudioParser, animation};
use dotenv::dotenv;
use roboat::ClientBuilder;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let file_path = shellexpand::tilde("~/Documents/Place1.rbxl");

    // let parser = match StudioParser::from_rbxl(file_path.as_ref()) {
    //     Ok(parser) => parser,
    //     Err(e) => {
    //         eprintln!("Error loading file: {}", e);
    //         return;
    //     }
    // };

    let roblox_cookie = std::env::var("ROBLOSECURITY").expect(".ROBLOSECURITY must be set in .env");
    let client = ClientBuilder::new().roblosecurity(roblox_cookie).build();

    // Build the parser with the roboat client
    let parser = match StudioParser::builder()
        .file_path(file_path.as_ref())
        .roblosecurity("1233")
        .build()
    {
        Ok(parser) => parser,
        Err(e) => {
            eprintln!("Error loading file: {}", e);
            return;
        }
    };

    let animations = parser.workspace_animations();
    let scripts = parser.animations_in_scripts();

    // match parser.reupload_animation().await {
    //     Ok(_) => {}
    //     Err(e) => {
    //         eprintln!("Failed uploading animation {:?}", e)
    //     }
    // }
}
