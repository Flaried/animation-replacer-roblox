use bytes::Bytes;
use roboat::ClientBuilder;
use roboat::RoboatError;
use roboat::assetdelivery::AssetBatchPayload;
use roboat::assetdelivery::AssetBatchResponse;
use roboat::catalog::CreatorType;
use roboat::catalog::Item;
use roboat::catalog::ItemType;
use roboat::ide::ide_types::NewAnimation;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::time;

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

    pub async fn fetch_animation_owner_details(
        &self,
        asset_ids: Vec<u64>,
    ) -> Result<Vec<OwnerInfo>, RoboatError> {
        let mut return_list = Vec::new();
        let batch_size = 150;
        let id_batches = asset_ids.chunks(batch_size);
        let max_retries: u32 = 9;
        let client = ClientBuilder::new()
            .roblosecurity(self.roblosecurity.clone())
            .build();

        for batch in id_batches {
            let mut attempts: u32 = 0;
            loop {
                // Create a Vec of Items for the batch
                let items: Vec<Item> = batch
                    .iter()
                    .map(|&asset_id| Item {
                        item_type: ItemType::Asset,
                        id: asset_id,
                    })
                    .collect();

                // Make a single API call with all 150 items
                match client.item_details(items).await {
                    Ok(results) => {
                        let owner_infos: Vec<OwnerInfo> = results
                            .iter()
                            .map(|details| OwnerInfo {
                                owner_id: details.creator_id,
                                owner_type: details.creator_type,
                            })
                            .collect();
                        return_list.extend(owner_infos);

                        // return_list.append(&mut results);
                        break;
                    }
                    Err(e) => {
                        eprintln!("Error fetching owners: {:?}", e);
                        if attempts < max_retries {
                            attempts += 1;
                            println!("Retrying owner fetch {}/{}...", attempts, max_retries);
                            // timeout_timer += 1;
                            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                            continue;
                        }
                        return Err(e);
                    }
                }
            }
        }
        Ok(return_list)
    }

    /// Fetches animation assets from a list of asset IDs
    ///
    /// * Notes
    /// This function will divide the list into 250 chunks and post to the API
    /// The AssetBatchResponse CAN contain errors (One asset could be private etc.)
    ///
    /// * Returns
    /// Returns all the Asset Responses (Can have errors)
    /// If the roblox cookie is invalid it will return a roboat error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let ids = vec![123456, 789012];
    /// let animations = uploader.fetch_animation_assets(ids).await?;
    /// ```
    pub async fn fetch_animation_assets(
        &self,
        asset_ids: Vec<u64>,
    ) -> Result<Vec<AssetBatchResponse>, RoboatError> {
        use roboat::catalog::AssetType;
        use tokio::time::{Duration, sleep};

        let mut return_list: Vec<AssetBatchResponse> = Vec::new();
        let batch_size = 250;
        let id_batches = asset_ids.chunks(batch_size);
        let mut timeout_timer: u64 = 4;
        let max_retries: u32 = 9;

        for batch in id_batches {
            let payloads: Vec<AssetBatchPayload> = batch
                .iter()
                .map(|&asset_id| AssetBatchPayload {
                    asset_id: Some(asset_id.to_string()),
                    request_id: Some(asset_id.to_string()),
                    ..Default::default()
                })
                .collect();

            let mut attempts: u32 = 0;
            loop {
                match self
                    .check_asset_metadata(payloads.clone(), time::Duration::new(timeout_timer, 0))
                    .await
                {
                    Ok(Some(mut result)) => {
                        result.retain(|item| matches!(item.asset_type, Some(AssetType::Animation)));
                        return_list.append(&mut result);
                        break;
                    }
                    Ok(None) => break,
                    Err(e) => {
                        eprintln!("Error checking animation types: {:?}", e);
                        if let RoboatError::ReqwestError(ref reqwest_err) = e {
                            if reqwest_err.is_timeout() && attempts < max_retries {
                                attempts += 1;
                                println!(
                                    "Request timed out, retrying with higher timeout: attempts {}/{}...",
                                    attempts, max_retries
                                );
                                timeout_timer += 1;
                                sleep(Duration::from_secs(2)).await;
                                continue;
                            }
                        } else if matches!(e, RoboatError::MalformedResponse) {
                            if attempts < max_retries {
                                attempts += 1;
                                println!(
                                    "Malformed request, retrying {}/{}...",
                                    attempts, max_retries
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
                    // Skip this result since we can't map it without a request_id
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
            // return Err(errors.into_iter().next().unwrap());
        }

        Ok(animation_hashmap)
    }
}
mod internal {
    use bytes::Bytes;
    use roboat::ClientBuilder;
    use roboat::RoboatError;
    use roboat::assetdelivery::AssetBatchPayload;
    use roboat::assetdelivery::AssetBatchResponse;
    use tokio::time;

    use crate::animation::uploader::AnimationUploader;

    impl AnimationUploader {
        /// Checks asset metadata for up to 250 assets.
        /// Now it also returns a place made by the creator, for place_id header to upload
        /// animations
        pub async fn check_asset_metadata(
            &self,
            asset_ids: Vec<AssetBatchPayload>,
            timeout_secs: time::Duration,
        ) -> Result<Option<Vec<AssetBatchResponse>>, RoboatError> {
            let timeout_client = reqwest::ClientBuilder::new()
                .timeout(timeout_secs)
                .build()
                .map_err(RoboatError::ReqwestError)?;

            let client = ClientBuilder::new()
                .roblosecurity(self.roblosecurity.clone())
                .reqwest_client(timeout_client)
                .build();

            match client.post_asset_metadata_batch(asset_ids).await {
                Ok(x) => Ok(Some(x)),
                Err(e) => Err(e),
            }
        }

        /// Downloads file bytes from a URL with retry logic.
        ///
        /// # Examples
        ///
        /// ```rust
        /// let bytes = uploader.file_bytes_from_url("https://example.com/file.rbxm".to_string()).await?;
        /// ```
        pub async fn file_bytes_from_url(&self, url: String) -> Result<Bytes, RoboatError> {
            use reqwest::Client;
            use tokio::time::{Duration, timeout};

            let max_retries: usize = 3;
            const TIMEOUT_SECS: u64 = 3;

            let client = Client::new();

            for attempt in 1..=max_retries {
                let result =
                    timeout(Duration::from_secs(TIMEOUT_SECS), client.get(&url).send()).await;

                match result {
                    Ok(Ok(response)) => {
                        let bytes = response.bytes().await.map_err(RoboatError::ReqwestError)?;
                        return Ok(bytes);
                    }
                    Ok(Err(e)) => {
                        if attempt == max_retries {
                            return Err(RoboatError::ReqwestError(e))?;
                        }
                    }
                    Err(e) => {
                        println!("Getting btres from url error: {:?}", e);
                        if attempt == max_retries {
                            return Err(RoboatError::InternalServerError);
                        }
                    }
                }
            }
            unreachable!("Loop should always return")
        }
    }
}
