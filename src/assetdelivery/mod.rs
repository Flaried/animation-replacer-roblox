//
mod request_types;
// pub async fn catalog_search(id: u64) -> Result<(), Box<dyn std::error::Error>> {
//     let base_url = "https://assetdelivery.roblox.com/v1/asset";
//     //
//
//     let url = Url::parse_with_params(base_url, &[("id", id.to_string())])?;
//     // let resp = reqwest::get("https://catalog.roblox.com/v1/search/navigation-menu-items").await?;
//     // let resp_json = resp.json::<Wrapper>().await?;
//     // println!("{:#?}", resp_json);
//     // Ok(())
// }

pub async fn pull_animation(id: u64)
pub async fn push_animation(id: u64) -> Result<(), Box<dyn std::error::Error>> {
    let animation_struct = request_types::Animation {
        id: Some(id),
        title: "My Animation".to_string(),
        description: "This is an animation.".to_string(),
        group_id: None, // Or Some(123456) if needed
    };

    request_types::publish_url(&animation_struct);
    Ok(())
}
