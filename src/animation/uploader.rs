use bytes::Bytes;
use roboat::ClientBuilder;
use roboat::RoboatError;
use roboat::assetdelivery::AssetBatchResponse;
use roboat::catalog::CreatorType;
use roboat::ide::ide_types::NewAnimation;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Semaphore;

pub struct AnimationUploader {
    pub roblosecurity: String,
}

#[derive(Debug)]
pub struct AnimationWithPlace {
    pub animation: AssetBatchResponse,
    pub place_id: u64,
}

pub struct OwnerInfo {
    pub owner_id: u64,
    pub owner_type: CreatorType,
}

impl AnimationUploader {
    /// Creates a new AnimationUploader with a roblosecurity cookie.
    pub fn new(roblosecurity: String) -> Self {
        Self { roblosecurity }
    }

    /// Uploads animation data to Roblox.
    ///
    /// * Parameters
    /// Animation_Data: Bytes
    /// Group Id to upload to (Option)
    ///
    /// * Returns
    /// New Asset Id (Sucess)
    /// RoboatError (Failed)
    ///
    /// * Examples
    ///
    /// ```rust
    /// let uploader = AnimationUploader::new(cookie);
    /// let data = std::fs::read("animation.rbxm")?.into();
    /// let id = uploader.upload_animation(data, Some(123456)).await?;
    /// ```
    pub async fn upload_animation(
        &self,
        animation_data: Bytes,
        group_id: Option<u64>,
    ) -> Result<String, RoboatError> {
        let client = ClientBuilder::new()
            .roblosecurity(self.roblosecurity.clone())
            .build();

        let animation = NewAnimation {
            group_id: group_id,
            name: "reuploaded_animation".to_string(),
            description: "This is a example".to_string(),
            animation_data,
        };

        let new_asset_id_string = client.upload_new_animation(animation).await?;
        Ok(new_asset_id_string)
    }

    /// Reuploads animations concurrently with semaphore limiting.
    ///
    /// * Notes
    /// Uses Semaphore for multiproccessing, default it set at 5 semphores
    ///
    ///
    /// # Example
    /// ```rust
    /// let animtions: Vec<AssetBatchResponse> = Vec::New(EXAMPLE)
    /// let uploader = Arc::new(AnimationUploader::new(cookie));
    /// let mapping = uploader.reupload_all_animations(animations, Some(group_id)).await?;
    /// ```
    pub async fn reupload_all_animations(
        self: Arc<Self>,
        animations: Vec<AssetBatchResponse>,
        group_id: Option<u64>,
        task_count: Option<u64>,
    ) -> Result<HashMap<String, String>, RoboatError> {
        let max_concurrent_tasks = task_count.unwrap_or(500);

        let semaphore = Arc::new(Semaphore::new(max_concurrent_tasks as usize));
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
                let self_arc = Arc::clone(&self);
                let location = location.to_string();
                let request_id = animation.request_id.clone();
                let group_id = group_id.clone();

                let task = tokio::spawn(async move {
                    let _permit = semaphore.acquire().await.unwrap();
                    let animation_file = self_arc.file_bytes_from_url(location).await?;

                    // Retry logic for upload_animation
                    let max_upload_retries: usize = 5;
                    let mut last_error = None;

                    for attempt in 1..=max_upload_retries {
                        match self_arc
                            .upload_animation(animation_file.clone(), group_id)
                            .await
                        {
                            Ok(new_animation_id) => {
                                println!(
                                    "Success uploading animation {}/{} ({} remaining)",
                                    index + 1,
                                    total_animations,
                                    total_animations - (index + 1),
                                );
                                return Ok::<_, RoboatError>((request_id, new_animation_id));
                            }

                            Err(e) => {
                                eprintln!(
                                    "Upload attempt {}/{} failed for animation {}: {}",
                                    attempt,
                                    max_upload_retries,
                                    index + 1,
                                    e
                                );
                                last_error = Some(e);

                                // Don't sleep on the last attempt
                                if attempt < max_upload_retries {
                                    tokio::time::sleep(tokio::time::Duration::from_millis(1000))
                                        .await;
                                }
                            }
                        }
                    }

                    // All retries failed
                    Err(last_error.unwrap())
                });

                tasks.push(task);
            }
        }

        let mut animation_hashmap = HashMap::new();
        let mut errors = Vec::new();
        let total_tasks = tasks.len();

        for task in tasks {
            match task.await {
                // Task completed successfully with a result and request_id exists
                Ok(Ok((Some(request_id), new_animation_id))) => {
                    animation_hashmap.insert(request_id, new_animation_id);
                }

                // Handle case where animation_id is None
                Ok(Ok((None, _))) => {
                    eprintln!("Warning: Animation uploader success but no animation_id available");
                }

                // Task completed but your function returned an error
                Ok(Err(e)) => {
                    if matches!(e, RoboatError::BadRequest) {
                        eprintln!(
                            "Animation Upload API failed to respond with errors; Cookie cannot publish animations. \nwith group uploading; make sure the cookie has perms to ALL Asset and Experience permissions\n"
                        )
                    } else {
                        eprintln!("Animation upload failed: {}", e);
                    }
                    errors.push(e);
                }

                // Task panicked or was cancelled
                Err(join_error) => {
                    eprintln!("Task failed to execute: {}", join_error);
                    // Just log the error and continue - don't fail the entire batch
                }
            }
        }

        // Handle collected errors
        if !errors.is_empty() {
            eprintln!(
                "Some uploads failed: {} out of {} tasks\nERROR INFO: {:?}",
                errors.len(),
                total_tasks,
                errors
            );
        }

        Ok(animation_hashmap)
    }

    ///  Gets all the animation file data to re-upload them
    /// * Notes
    /// This func uses caching and hashmaps to handle needing place-id to download assets
    pub async fn fetch_animation_assets(
        &self,
        asset_ids: Vec<u64>,
    ) -> anyhow::Result<Vec<AssetBatchResponse>> {
        let mut cached_places: HashMap<String, u64> = HashMap::new();
        let mut place_id: Option<String> = None;
        let mut animations: Vec<AssetBatchResponse> = Vec::new();
        let batch_size = 250;

        for batch in asset_ids.chunks(batch_size) {
            let batch_animations = self
                .fetch_batch_with_retry(batch, &mut cached_places, &mut place_id)
                .await?;
            animations.extend(batch_animations);
        }

        Ok(animations)
    }
}

mod internal {
    use std::collections::HashMap;

    use roboat::{
        RoboatError,
        assetdelivery::{AssetBatchPayload, AssetBatchResponse},
    };

    use crate::AnimationUploader;

    impl AnimationUploader {
        /// Fetches asset metadata for a batch of asset IDs with automatic retry logic and 403 error handling.
        ///
        /// This function attempts to retrieve asset metadata for the provided asset IDs, handling
        /// error conditions specifically manages 403 permission
        /// errors by dynamically discovering and using place IDs
        ///
        /// # Parameters
        /// * `asset_ids` - A slice of asset IDs to fetch metadata for (typically up to 250 per batch)
        /// * `cached_places` - A mutable reference to a HashMap that caches asset_id -> place_id mappings
        ///                     to avoid redundant API calls across batches
        /// * `place_id` - A mutable reference to an optional place_id that persists across retries and batches.
        ///                When a 403 error occurs, this will be populated with the discovered place_id
        ///
        /// # Returns
        /// * `Ok(Vec<AssetBatchResponse>)` - Successfully fetched and filtered animation assets
        /// * `Err(anyhow::Error)` - Failed after all retry attempts or encountered unrecoverable error
        ///
        /// # Retry Logic
        /// - **Max Retries**: 9 attempts with exponential timeout increase
        /// - **403 Errors**: Automatically discovers place_id from first 403 error and retries
        /// - **Network Errors**: Retries with increased timeout for reqwest timeout/malformed response errors
        /// - **Timeout Progression**: Starts at 4 seconds, increases by 1 second per retry attempt
        ///
        /// # Error Handling
        /// - **403 Forbidden**: Extracts place_id from the failing asset and retries the entire batch
        /// - **Timeout/Network**: Implements backoff strategy with 2-second sleep between attempts
        /// - **Other Errors**: Fails immediately without retry
        ///
        /// # Examples
        /// ```rust
        /// let mut cache = HashMap::new();
        /// let mut place_id = None;
        /// let asset_ids = vec![123456, 789012, 345678];
        ///
        /// let animations = uploader
        ///     .fetch_batch_with_retry(&asset_ids, &mut cache, &mut place_id)
        ///     .await?;
        /// ```
        pub(super) async fn fetch_batch_with_retry(
            &self,
            asset_ids: &[u64],
            cached_places: &mut HashMap<String, u64>,
            place_id: &mut Option<String>,
        ) -> anyhow::Result<Vec<AssetBatchResponse>> {
            use tokio::time::{Duration, sleep};

            const MAX_RETRIES: u32 = 9;
            const INITIAL_TIMEOUT: u64 = 4;

            let mut timeout_seconds = INITIAL_TIMEOUT;
            let mut attempts = 0;

            loop {
                let payloads = self.create_batch_payloads(asset_ids);

                match self
                    .check_asset_metadata(
                        payloads,
                        place_id.clone(),
                        Duration::from_secs(timeout_seconds),
                    )
                    .await
                {
                    Ok(Some(responses)) => {
                        // Check if we got any 403 errors
                        if let Some(new_place_id) = self
                            .handle_first_403_error(&responses, cached_places)
                            .await?
                        {
                            // Found a 403 error and got a new place_id, retry with this place_id
                            *place_id = Some(new_place_id);

                            println!(
                                "Got 403 error, retrying with place_id: {}",
                                place_id.as_ref().unwrap()
                            );
                            continue;
                        }

                        // No 403 errors, filter and return animations
                        return Ok(self.filter_animations(responses));
                    }
                    Ok(None) => {
                        return Ok(Vec::new());
                    }
                    Err(e) => {
                        if self.should_retry(&e, attempts, MAX_RETRIES) {
                            attempts += 1;
                            timeout_seconds += 1;

                            println!(
                                "Request failed, retrying with higher timeout: attempts {}/{} ({})",
                                attempts, MAX_RETRIES, e
                            );

                            sleep(Duration::from_secs(2)).await;
                            continue;
                        }
                        return Err(e);
                    }
                }
            }
        }

        pub(super) async fn handle_first_403_error(
            &self,
            responses: &[AssetBatchResponse],
            cached_places: &mut HashMap<String, u64>,
        ) -> anyhow::Result<Option<String>> {
            // Find the first 403 error
            for response in responses {
                if self.has_403_error(response) {
                    if let Some(request_id_str) = &response.request_id {
                        let asset_id: u64 = request_id_str.parse().map_err(|e| {
                            anyhow::anyhow!("Failed to parse asset ID '{}': {}", request_id_str, e)
                        })?;

                        let place_id = self.get_or_fetch_place_id(asset_id, cached_places).await?;

                        println!("Found place_id: {} for asset: {}", place_id, request_id_str);
                        return Ok(Some(place_id.to_string()));
                    }
                }
            }
            // No 403 errors found
            Ok(None)
        }

        pub(super) async fn get_or_fetch_place_id(
            &self,
            asset_id: u64,
            cached_places: &mut HashMap<String, u64>,
        ) -> anyhow::Result<u64> {
            let cache_key = asset_id.to_string();

            if let Some(&place_id) = cached_places.get(&cache_key) {
                return Ok(place_id);
            }

            let place_id = self.place_id(asset_id, cached_places).await.map_err(|e| {
                anyhow::anyhow!(
                    "Failed to get place id for asset {} error: {}",
                    cache_key,
                    e
                )
            })?;

            cached_places.insert(cache_key, place_id);
            Ok(place_id)
        }
        pub(super) fn create_batch_payloads(&self, asset_ids: &[u64]) -> Vec<AssetBatchPayload> {
            asset_ids
                .iter()
                .map(|&asset_id| AssetBatchPayload {
                    asset_id: Some(asset_id.to_string()),
                    request_id: Some(asset_id.to_string()),
                    ..Default::default()
                })
                .collect()
        }

        pub(super) fn has_403_error(&self, response: &AssetBatchResponse) -> bool {
            response
                .errors
                .as_ref()
                .map(|errors| errors.iter().any(|error| error.code == 403))
                .unwrap_or(false)
        }

        pub(super) fn filter_animations(
            &self,
            responses: Vec<AssetBatchResponse>,
        ) -> Vec<AssetBatchResponse> {
            use roboat::catalog::AssetType;

            responses
                .into_iter()
                .filter(|response| {
                    // Only include responses without errors that are animations
                    response.errors.is_none()
                        && matches!(response.asset_type, Some(AssetType::Animation))
                })
                .collect()
        }

        pub(super) fn should_retry(
            &self,
            error: &anyhow::Error,
            attempts: u32,
            max_retries: u32,
        ) -> bool {
            if attempts >= max_retries {
                return false;
            }

            if let Some(roboat_error) = error.downcast_ref::<RoboatError>() {
                match roboat_error {
                    RoboatError::ReqwestError(reqwest_err) if reqwest_err.is_timeout() => true,
                    RoboatError::MalformedResponse => true,
                    _ => false,
                }
            } else {
                false
            }
        }
    }
}
