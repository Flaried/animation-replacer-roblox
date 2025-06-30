use core::time;
use roboat::ClientBuilder;
use roboat::RoboatError;
use roboat::assetdelivery::request_types::AssetBatchPayload;
use roboat::assetdelivery::request_types::AssetBatchResponse;
use roboat::catalog::AssetType;
use roboat::ide::ide_types::NewAnimation;
use roboat::reqwest;
use std::time::Duration;

use crate::StudioParser;

impl StudioParser {
    /// Downloads an existing animation and re-uploads it with default metadata.
    /// Useful for backing up or migrating animations.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Re-upload a single animation
    /// parser.reupload_animation(123456789).await?;
    ///
    /// // Batch process all animations found in scripts
    /// let animation_ids = parser.find_all_animations().await?;
    /// for &id in &animation_ids {
    ///     parser.reupload_animation(id).await?;
    ///     tokio::time::sleep(Duration::from_secs(1)).await; // Rate limiting
    /// }
    /// ```
    ///
    /// Requires roblosecurity cookie. Does not handle rate limiting automatically.
    ///
    pub async fn reupload_animation(&self, asset_id: u64) -> Result<String, RoboatError> {
        let client = ClientBuilder::new()
            // Gonna error if theres no cookie
            .roblosecurity(
                self.roblosecurity
                    .clone()
                    .expect("roblosecurity cookie required"),
            )
            .build();

        let existing_animation = client.fetch_asset_data(asset_id).await?;

        let animation = NewAnimation {
            group_id: None,
            name: "roboatTest".to_string(),
            description: "This is a roboat example".to_string(),
            animation_data: existing_animation,
        };

        let new_asset_id_string = client.upload_new_animation(animation).await?;

        Ok(new_asset_id_string)
    }

    /// Takes in a batch of up to 250 assets and returns all the details to them.
    /// It will return a vector of AssetBatchResponse
    /// Also creates a timeout for the hanging API error from this endpoint.
    /// Requires Cookie
    pub async fn check_asset_info(
        &self,
        asset_id: Vec<AssetBatchPayload>,
    ) -> Result<Option<Vec<AssetBatchResponse>>, RoboatError> {
        // let proxy_url = "http://localhost:8080";
        // let proxy = Proxy::all(proxy_url).expect("Proxy error");
        let timeout_client = reqwest::ClientBuilder::new()
            // .proxy(proxy)
            .timeout(time::Duration::new(3, 0))
            .build()
            .map_err(RoboatError::ReqwestError)?;

        let client = ClientBuilder::new()
            .roblosecurity(
                self.roblosecurity
                    .clone()
                    .expect("roblosecurity cookie required"),
            )
            .reqwest_client(timeout_client)
            .build();
        // Call get_asset_info and handle the result.
        match client.post_asset_metadata_batch(asset_id).await {
            Ok(x) => Ok(Some(x)),
            Err(e) => Err(e),
        }
    }
}

mod internal {
    use roboat::RoboatError;
    use tokio::time::sleep;

    use crate::StudioParser;
    use roboat::assetdelivery::request_types::{AssetBatchPayload, AssetBatchResponse};
    use roboat::catalog::AssetType;
    use tokio::time::Duration;

    impl StudioParser {
        /// Fetches metadata for a list of asset IDs using Roblox's asset batch API.
        ///
        /// # Parameters
        /// - `asset_ids`: A vector of u64 asset IDs to query.
        ///
        /// # Returns
        /// - `Ok(Vec<AssetBatchResponse>)`: A list of animation assets successfully retrieved and filtered.
        /// - `Err(RoboatError)`: If the request fails (e.g., due to timeouts, malformed responses, or network issues).
        ///
        /// # Behavior
        /// - The API is called in batches of 250 assets (the Roblox batch limit).
        /// - Only assets of type `Animation` are retained in the final response.
        /// - Retries failed batches up to 3 times if the error is a timeout or malformed response.
        /// - Skips and logs failed batches after max retries.
        ///
        /// # Notes
        /// - Each batch is retried up to `MAX_RETRIES` times (3 by default) if a timeout or known recoverable error occurs.
        /// - Uses a brief delay (2 seconds) between retry attempts.
        /// - The request ID in the payload is set to the asset ID for easier correlation in responses.
        pub async fn fetch_animation_assets(
            &self,
            asset_ids: Vec<u64>,
        ) -> Result<Vec<AssetBatchResponse>, RoboatError> {
            let mut return_list: Vec<AssetBatchResponse> = Vec::new();
            let batch_size = 250;
            let id_batches = asset_ids.chunks(batch_size);
            const MAX_RETRIES: i32 = 3;

            for batch in id_batches {
                let payloads: Vec<AssetBatchPayload> = batch
                    .iter()
                    .map(|&asset_id| AssetBatchPayload {
                        asset_id: Some(asset_id.to_string()),
                        // Little hack here, the response requestId will be the assetId
                        request_id: Some(asset_id.to_string()),
                        ..Default::default()
                    })
                    .collect();
                let mut attempts = 0;
                loop {
                    match self.check_asset_info(payloads.clone()).await {
                        Ok(Some(mut result)) => {
                            // Keep only animations
                            result.retain(|item| {
                                matches!(item.asset_type, Some(AssetType::Animation))
                            });
                            return_list.append(&mut result);
                            break;
                        }
                        Ok(None) => {
                            // Handle the case where the result is None
                            break;
                        }
                        Err(e) => {
                            eprintln!("Error checking animation types: {:?}", e);
                            if let RoboatError::ReqwestError(ref reqwest_err) = e {
                                if reqwest_err.is_timeout() && attempts < MAX_RETRIES {
                                    attempts += 1;
                                    println!("Timeout, retrying {}/{}...", attempts, MAX_RETRIES);
                                    sleep(Duration::from_secs(2)).await;
                                    continue;
                                }
                            } else if matches!(e, RoboatError::MalformedResponse) {
                                if attempts < MAX_RETRIES {
                                    attempts += 1;
                                    println!(
                                        "Malformed request, retrying {}/{}...",
                                        attempts, MAX_RETRIES
                                    );
                                    sleep(Duration::from_secs(2)).await;
                                    continue;
                                }
                            }
                            return Err(e);
                        }
                    }
                }
            }
            Ok(return_list)
        }
    }
}
