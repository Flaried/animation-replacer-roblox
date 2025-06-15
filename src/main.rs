use animation_replace_roblox::roblox_api::assetdelivery::request_types::Animation;
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

    // let result = session.refresh_csrf().await;
    // match result {
    //     Ok(test) => println!("Refreshed XCSRF {:?}", test),
    //     Err(e) => eprintln!("Failed to refresh xCSRF {}", e),
    // }

    match session.pull_animation(128511411359897).await {
        Ok(animation_data) => {
            let new_animation = Animation {
                id: None,
                title: String::from("Test"),
                description: String::from("test"),
                group_id: None,
                animation_data: Some(String::from(animation_data)),
            };

            match session.publish_animation(new_animation).await {
                Ok(_) => {}
                Err(e) => {
                    // Handle the error if necessary
                    eprintln!("Error: {:?}", e);
                }
            }
        } // Do nothing here
        Err(e) => eprintln!("Failed to fetch animation: {}", e),
    }
}
