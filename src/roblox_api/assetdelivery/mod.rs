//
mod request_types;
use reqwest::header::HeaderValue;
use std::error::Error;

use crate::{
    XCSRF_HEADER,
    roblox_api::{
        roblox_client::{self, RobloxSession},
        validation,
    },
};
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

impl RobloxSession {
    pub async fn pull_animation(&self, item_id: u64) -> Result<String, Box<dyn Error>> {
        let xcsrf_token = self.xcsrf.read().await.clone();
        let url = format!("https://assetdelivery.roblox.com/v1/asset/?id={}", item_id);

        let response = self
            .reqwest_client
            .get(&url)
            .header(XCSRF_HEADER, HeaderValue::from_str(&xcsrf_token)?)
            .send()
            .await?;

        let response = Self::handle_status(response).await?;

        let content = response.text().await?;
        Ok(content)
    }
}

// pub async fn pull_animation(id: u64) -> Result<(), Error> {
//     let assetdeliveryurl = request_types::asset_delivery_url(id);
//
//     let resp = reqwest::get(assetdeliveryurl).await?;
//     let text = resp.text().await?; // Get response body as text
//
//     println!("Response Text:\n{}", text);
//     Ok(())
// }
// pub async fn push_animation(id: u64) -> Result<(), Box<dyn std::error::Error>> {
//     let animation_struct = request_types::Animation {
//         id: Some(id),
//         title: "My Animation".to_string(),
//         description: "This is an animation.".to_string(),
//         group_id: None, // Or Some(123456) if needed
//     };
//
//     let upload_url = request_types::upload_animation_url(&animation_struct);
//
//     Ok(())
// }
