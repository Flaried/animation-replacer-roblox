use roboat::ClientBuilder;
use roboat::RoboatError;
use roboat::ide::ide_types::NewAnimation;

pub async fn reupload_animation() -> Result<(), RoboatError> {
    let roblox_cookie = std::env::var("ROBLOSECURITY").expect(".ROBLOSECURITY must be set in .env");
    let client = ClientBuilder::new().roblosecurity(roblox_cookie).build();

    let asset_id = 128511411359897;
    let existing_animation = client.fetch_asset_data(asset_id).await?;

    let animation = NewAnimation {
        group_id: None,
        name: "roboatTest".to_string(),
        description: "This is a roboat example".to_string(),
        animation_data: existing_animation,
    };

    client.upload_new_animation(animation).await?;

    println!("Uploaded Animation!");
    Ok(())
}
