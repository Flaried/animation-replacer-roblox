pub mod request_types;
use crate::XCSRF_HEADER;
use crate::roblox_api::assetdelivery::request_types::{Animation, upload_animation_url};
use crate::roblox_api::{errors::RobloxError, roblox_client::RobloxSession};
use base64::encode;
use reqwest::Response;
use reqwest::header::{COOKIE, USER_AGENT};

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

    pub async fn publish_animation(&self, new_animation: Animation) -> Result<(), RobloxError> {
        match self.publish_animation_internal(new_animation.clone()).await {
            Ok(_) => {
                // Since we expect a successful result with no additional data, just return Ok(())
                Ok(())
            }
            Err(e) => match e {
                RobloxError::InvalidXcsrf(new_xcsrf) => {
                    // Set the new X-CSRF token and retry the request
                    self.set_xcsrf(new_xcsrf).await;

                    // Retry the internal publish call
                    let _retry_response = self.publish_animation_internal(new_animation).await?;

                    // In case the retry succeeds, return Ok(())
                    Ok(())
                }
                _ => Err(e), // Propagate other errors
            },
        }
    }

    async fn publish_animation_internal(
        &self,
        new_animation: Animation,
    ) -> Result<Response, RobloxError> {
        let xcsrf_token = self.read_xcsrf().await;
        let url = upload_animation_url(&new_animation);

        println!("Publishing to URL: {}", url);

        let mut builder = self
            .reqwest_client
            .post(&url)
            .header(USER_AGENT, "Roblox/WinInet")
            // .body(new_animation.animation_data)
            .header(XCSRF_HEADER, xcsrf_token);

        // Check if `animation_data` exists (is Some)
        if let Some(animation_data) = new_animation.animation_data {
            builder = builder.body(encode(animation_data)); // Add animation data to the body
            println!("Joined data")
        } else {
            return Err(RobloxError::MissingAnimationData); // Return an error if data is missing
        }

        // Add authentication cookie if available
        if let Some(cookie_string) = &self.cookie_string {
            builder = builder.header(COOKIE, cookie_string.clone());
            println!("Set cookie {:?}", builder)
        } else {
            return Err(RobloxError::RoblosecurityNotSet);
        }

        let request_result = builder.send().await;
        let response = Self::validate_request_result(request_result).await?;

        Ok(response)
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
