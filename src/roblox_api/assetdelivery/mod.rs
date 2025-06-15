//
mod request_types;
use crate::roblox_api::roblox_client::RobloxSession;
use reqwest::header::{COOKIE, HeaderValue};
use std::error::Error;

use crate::XCSRF_HEADER;
impl RobloxSession {
    pub async fn pull_animation(&self, item_id: u64) -> Result<String, Box<dyn Error>> {
        let xcsrf_token = self.xcsrf.read().await.clone();
        let url = format!("https://assetdelivery.roblox.com/v1/asset/?id={}", item_id);

        println!("{:?}", xcsrf_token);

        // Start building the request.
        let mut builder = self
            .reqwest_client
            .get(&url)
            .header(XCSRF_HEADER, HeaderValue::from_str(&xcsrf_token)?);

        // Add the roblosecurity cookie if it exists.
        if let Some(cookie_string) = &self.cookie_string {
            builder = builder.header(COOKIE, cookie_string.clone());
        }

        let request_result = builder.send().await;
        match Self::validate_request_result(request_result).await {
            Ok(response) => {
                // Extract and return the content.
                let content = response.text().await?;
                Ok(content)
            }
            Err(e) => {
                // Handle errors (network errors, status errors, etc.)
                Err(Box::new(e))
            }
        }
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
