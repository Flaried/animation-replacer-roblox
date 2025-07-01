use bytes::Bytes;
use core::time;
use roboat::ClientBuilder;
use roboat::RoboatError;
use roboat::assetdelivery::request_types::AssetBatchPayload;
use roboat::assetdelivery::request_types::AssetBatchResponse;
use roboat::ide::ide_types::NewAnimation;
use roboat::reqwest;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Semaphore;

pub struct AnimationUploader {
    pub roblosecurity: String,
}
impl AnimationUploader {
    pub fn new(roblosecurity: String) -> Self {
        Self { roblosecurity }
    }
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
    pub async fn upload_animation(&self, animation_data: Bytes) -> Result<String, RoboatError> {
        let client = ClientBuilder::new()
            .roblosecurity(self.roblosecurity.clone())
            .build();

        let animation = NewAnimation {
            group_id: None,
            name: "roboatTest630".to_string(),
            description: "This is a roboat example".to_string(),
            animation_data,
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
                self.roblosecurity.clone(), // .expect("roblosecurity cookie required"),
            )
            .reqwest_client(timeout_client)
            .build();
        // Call get_asset_info and handle the result.
        match client.post_asset_metadata_batch(asset_id).await {
            Ok(x) => Ok(Some(x)),
            Err(e) => Err(e),
        }
    }

    /// Reuploads a collection of animations by downloading them from their existing locations
    /// and uploading them as new assets.
    ///
    /// This function processes animations concurrently with a maximum of 5 simultaneous uploads
    /// to avoid overwhelming the server. It maintains a mapping between original animation IDs
    /// and their newly uploaded counterparts.
    ///
    /// # Arguments
    ///
    /// * `animations` - A vector of `AssetBatchResponse` objects containing animation metadata
    ///                  and download locations. Each animation should have at least one location
    ///                  in its `locations` field.
    ///
    /// # Returns
    ///
    /// * `Ok(HashMap<String, String>)` - A mapping where keys are original animation request IDs
    ///                                   and values are the new animation IDs after reupload
    /// * `Err(RoboatError)` - Returns an error if:
    ///   - Any animation download fails
    ///   - Any animation upload fails  
    ///   - A tokio task panics during execution
    ///
    /// # Behavior
    ///
    /// - Skips animations that don't have a valid location URL
    /// - Processes animations concurrently with a semaphore limiting to 5 simultaneous operations
    /// - Prints progress information to stdout showing current progress and remaining items
    /// - Only includes animations with valid `request_id` values in the returned mapping
    ///
    /// # Examples
    ///
    /// ```rust
    /// let animations = vec![
    ///     AssetBatchResponse {
    ///         request_id: Some("anim_123".to_string()),
    ///         locations: Some(vec![
    ///             AssetLocation {
    ///                 location: Some("https://example.com/animation1.rbxm".to_string()),
    ///             }
    ///         ]),
    ///     },
    ///     // ... more animations
    /// ];
    ///
    /// let client = Arc::new(MyClient::new());
    /// let result = client.reupload_all_animations(animations).await?;
    ///
    /// // result now contains mapping: {"anim_123" => "new_anim_456"}
    /// ```
    ///
    /// # Concurrency
    ///
    /// This function uses a semaphore to limit concurrent operations to 5 simultaneous uploads.
    /// This prevents overwhelming the target server while still providing reasonable parallelism
    /// for faster processing of large animation batches.
    ///
    /// # Error Handling
    ///
    /// - Individual animation processing errors are propagated immediately, stopping all processing
    /// - Tokio task join errors are converted to `RoboatError::InternalServerError`
    /// - Animations without valid locations or request IDs are silently skipped
    pub async fn reupload_all_animations(
        self: Arc<Self>, // Change from &self to Arc<Self>
        animations: Vec<AssetBatchResponse>,
    ) -> Result<HashMap<String, String>, RoboatError> {
        // Setup 5 Semaphore permits
        // NOTE: maybe set this as an arg
        let semaphore = Arc::new(Semaphore::new(5));
        let mut tasks = Vec::new();
        let total_animations = animations.len();

        for (index, animation) in animations.into_iter().enumerate() {
            let location_string = animation
                .locations
                .as_ref()
                .and_then(|locs| locs.first())
                .and_then(|loc| loc.location.as_ref());

            if let Some(location) = location_string {
                let semaphore = semaphore.clone();
                let self_arc = Arc::clone(&self); // Each task gets access to the client methods
                let location = location.to_string();
                let request_id = animation.request_id.clone();

                // The task goes into Tokio's scheduler queue and will run when a thread is available
                let task = tokio::spawn(async move {
                    // Blocks the thread until theres an active permit
                    let _permit = semaphore.acquire().await.unwrap();

                    println!(
                        "Processing animation {}/{} ({} remaining)",
                        index + 1,
                        total_animations,
                        total_animations - (index + 1)
                    );

                    // Only 5 of these operations happen simultaneously across ALL tasks
                    let animation_file = self_arc.file_bytes_from_url(location).await?;
                    let new_animation_id = self_arc.upload_animation(animation_file).await?;

                    Ok::<_, RoboatError>((request_id, new_animation_id))
                });

                tasks.push(task);
            }
        }

        let mut animation_hashmap = HashMap::new();

        // Collect all the tasks results
        for task in tasks {
            match task.await {
                Ok(Ok((Some(request_id), new_animation_id))) => {
                    animation_hashmap.insert(request_id, new_animation_id);
                }
                Ok(Err(e)) => return Err(e),
                Err(_) => return Err(RoboatError::InternalServerError),
                _ => {} // Skip if request_id is None
            }
        }

        Ok(animation_hashmap)
    }
}

mod internal {
    use bytes::Bytes;
    use roboat::{RoboatError, reqwest};
    use tokio::time::sleep;

    use crate::animation::AnimationUploader;
    use roboat::assetdelivery::request_types::{AssetBatchPayload, AssetBatchResponse};
    use roboat::catalog::AssetType;
    use tokio::time::Duration;

    use reqwest::Client;
    use tokio::time::timeout;

    const MAX_RETRIES: usize = 3;
    const TIMEOUT_SECS: u64 = 1;

    impl AnimationUploader {
        /// Downloads bytes from a roblox location URL
        pub async fn file_bytes_from_url(&self, url: String) -> Result<Bytes, RoboatError> {
            let client = Client::new();

            for attempt in 1..=MAX_RETRIES {
                let result =
                    timeout(Duration::from_secs(TIMEOUT_SECS), client.get(&url).send()).await;

                match result {
                    Ok(Ok(response)) => {
                        let bytes = response.bytes().await.map_err(RoboatError::ReqwestError)?;
                        return Ok(bytes);
                    }
                    Ok(Err(e)) => {
                        if attempt == MAX_RETRIES {
                            return Err(RoboatError::ReqwestError(e));
                        }
                    }
                    Err(_) => {
                        // Timeout hit
                        if attempt == MAX_RETRIES {
                            return Err(RoboatError::TooManyRequests);
                        }
                    }
                }
            }

            Err(RoboatError::InternalServerError) // fallback error if all retries fail
        }
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
