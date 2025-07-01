use bytes::Bytes;
use roboat::ClientBuilder;
use roboat::RoboatError;
use roboat::assetdelivery::AssetBatchPayload;
use roboat::assetdelivery::AssetBatchResponse;
use roboat::ide::ide_types::NewAnimation;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Semaphore;

pub struct AnimationUploader {
    pub roblosecurity: String,
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
            name: "roboatTestv2".to_string(),
            description: "This is a roboat example".to_string(),
            animation_data,
        };

        let new_asset_id_string = client.upload_new_animation(animation).await?;
        Ok(new_asset_id_string)
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
        const MAX_RETRIES: i32 = 9;

        for batch in id_batches {
            let payloads: Vec<AssetBatchPayload> = batch
                .iter()
                .map(|&asset_id| AssetBatchPayload {
                    asset_id: Some(asset_id.to_string()),
                    request_id: Some(asset_id.to_string()),
                    ..Default::default()
                })
                .collect();

            let mut attempts = 0;
            loop {
                match self.check_asset_info(payloads.clone()).await {
                    Ok(Some(mut result)) => {
                        result.retain(|item| matches!(item.asset_type, Some(AssetType::Animation)));
                        return_list.append(&mut result);
                        break;
                    }
                    Ok(None) => break,
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
    ) -> Result<HashMap<String, String>, RoboatError> {
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
                let self_arc = Arc::clone(&self);
                let location = location.to_string();
                let request_id = animation.request_id.clone();
                let group_id = group_id.clone();

                let task = tokio::spawn(async move {
                    let _permit = semaphore.acquire().await.unwrap();

                    println!(
                        "Reuploading animation {}/{} ({} remaining)",
                        index + 1,
                        total_animations,
                        total_animations - (index + 1)
                    );

                    let animation_file = self_arc.file_bytes_from_url(location).await?;
                    let new_animation_id =
                        self_arc.upload_animation(animation_file, group_id).await?;

                    Ok::<_, RoboatError>((request_id, new_animation_id))
                });

                tasks.push(task);
            }
        }

        let mut animation_hashmap = HashMap::new();

        for task in tasks {
            match task.await {
                Ok(Ok((Some(request_id), new_animation_id))) => {
                    animation_hashmap.insert(request_id, new_animation_id);
                }
                Ok(Err(e)) => return Err(e),
                Err(_) => return Err(RoboatError::InternalServerError),
                _ => {}
            }
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
        /// Checks asset information for up to 250 assets.
        pub async fn check_asset_info(
            &self,
            asset_id: Vec<AssetBatchPayload>,
        ) -> Result<Option<Vec<AssetBatchResponse>>, RoboatError> {
            let timeout_client = reqwest::ClientBuilder::new()
                .timeout(time::Duration::new(3, 0))
                .build()
                .map_err(RoboatError::ReqwestError)?;

            let client = ClientBuilder::new()
                .roblosecurity(self.roblosecurity.clone())
                .reqwest_client(timeout_client)
                .build();

            match client.post_asset_metadata_batch(asset_id).await {
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

            const MAX_RETRIES: usize = 3;
            const TIMEOUT_SECS: u64 = 1;

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
                        if attempt == MAX_RETRIES {
                            return Err(RoboatError::TooManyRequests);
                        }
                    }
                }
            }

            Err(RoboatError::InternalServerError)
        }
    }
}
