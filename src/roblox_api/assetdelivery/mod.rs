mod request_types;
use crate::XCSRF_HEADER;
use crate::roblox_api::{errors::RobloxError, roblox_client::RobloxSession};
use reqwest::Response;
use reqwest::header::COOKIE;

impl RobloxSession {
    /// Pulls animation data from Roblox API by item ID
    /// Handles XCSRF token refresh automatically on authentication failure
    pub async fn pull_animation(&self, item_id: u64) -> Result<String, RobloxError> {
        match self.pull_animation_internal(item_id).await {
            Ok(response) => response
                .text()
                .await
                .map_err(|e| RobloxError::ReqwestError(e)),
            Err(e) => match e {
                RobloxError::InvalidXcsrf(new_xcsrf) => {
                    self.set_xcsrf(new_xcsrf).await;
                    let response = self.pull_animation_internal(item_id).await?;
                    response
                        .text()
                        .await
                        .map_err(|e| RobloxError::ReqwestError(e))
                }
                _ => Err(e),
            },
        }
    }

    /// Internal method to make the actual HTTP request to fetch animation data
    async fn pull_animation_internal(&self, item_id: u64) -> Result<Response, RobloxError> {
        let xcsrf_token = self.read_xcsrf().await;
        let url = format!("https://assetdelivery.roblox.com/v1/asset/?id={}", item_id);

        // Build the request with required headers
        let mut builder = self
            .reqwest_client
            .get(&url)
            .header(XCSRF_HEADER, xcsrf_token);

        // Add authentication cookie if available
        if let Some(cookie_string) = &self.cookie_string {
            builder = builder.header(COOKIE, cookie_string.clone());
        }

        // Send the request and validate the response
        let request_result = builder.send().await;
        let response = Self::validate_request_result(request_result).await?;

        Ok(response)
    }
}
