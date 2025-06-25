use crate::Script;
use crate::ScriptType;
use crate::StudioParser;
// use futures::stream::{self, StreamExt};
use rbx_dom_weak::types::Variant;
use regex::Regex;
use roboat::catalog::AssetType;
// use reqwest::Proxy;
use roboat::ClientBuilder;
use roboat::RoboatError;
use roboat::assetdelivery::request_types::AssetBatchPayload;
use roboat::assetdelivery::request_types::AssetBatchResponse;
use std::collections::HashSet;
use std::time::Duration;
use tokio::time;
use tokio::time::sleep;
use ustr::Ustr;

impl StudioParser {
    /// Returns a vector of AssetBatchResponse (Animation Details from batch API) found in the script
    /// # Notes:
    /// Takes in a script, scans all the IDs into it then has batch_sizes of 250.
    /// It posts 250 Ids at a time to the asset batch API then filters out everything but
    /// animations.
    /// * Requires a cookie
    /// * Batch API does hang sometimes, fixed that with retries and 3 second timeout.
    pub async fn animations_in_script(
        &self,
        script: &Script<'_>,
    ) -> Result<Vec<AssetBatchResponse>, RoboatError> {
        let mut return_list: Vec<AssetBatchResponse> = Vec::new();

        let pattern = Regex::new(r"\d{5,}").unwrap();

        // Extract and deduplicate IDs first
        let unique_ids: HashSet<u64> = pattern
            .find_iter(&script.source)
            .filter_map(|m| m.as_str().parse().ok())
            .collect();

        let unique_id_list: Vec<u64> = unique_ids.into_iter().collect();
        let batch_size = 250;

        // Split the list into batches
        let id_batches = unique_id_list.chunks(batch_size);

        const MAX_RETRIES: i32 = 3;
        for batch in id_batches {
            let payloads: Vec<AssetBatchPayload> = batch
                .iter()
                .map(|&asset_id| AssetBatchPayload {
                    asset_id: Some(asset_id.to_string()),
                    //
                    // Little hack here, the response requestId will be the assetId
                    request_id: Some(asset_id.to_string()),
                    ..Default::default()
                })
                .collect();
            let mut attempts = 0;

            loop {
                match self.check_animation_types(payloads.clone()).await {
                    Ok(Some(mut result)) => {
                        // Keep only animations
                        result.retain(|item| matches!(item.asset_type, Some(AssetType::Animation)));
                        return_list.append(&mut result);

                        break;
                    }
                    Ok(None) => {
                        // Handle the case where the result is None, if necessary.
                        // For now, we simply ignore it.
                        break;
                    }
                    Err(e) => {
                        eprintln!("Error checking animation types: {:?}", e);
                        if let RoboatError::ReqwestError(ref reqwest_err) = e {
                            if reqwest_err.is_timeout() && attempts < MAX_RETRIES {
                                attempts += 1;
                                println!("Timeout, retrying {}/{}...", attempts, MAX_RETRIES);
                                sleep(Duration::from_secs(2)).await; // Optional backoff
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

    /// Takes in a batch of up to 250 assets and returns all the details to them.
    /// It will return a vector of AssetBatchResponse
    /// Also creates a timeout for the hanging API error from this endpoint.
    /// Requires Cookie
    async fn check_animation_types(
        &self,
        asset_id: Vec<AssetBatchPayload>,
    ) -> Result<Option<Vec<AssetBatchResponse>>, RoboatError> {
        // let proxy_url = "http://localhost:8080";
        // let proxy = Proxy::all(proxy_url).expect("Proxy error");
        let debug_client = reqwest::ClientBuilder::new()
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
            .reqwest_client(debug_client)
            .build();
        // Call get_asset_info and handle the result.
        match client.post_asset_metadata_batch(asset_id).await {
            Ok(x) => Ok(Some(x)),
            Err(e) => Err(e),
        }
    }

    /// Gets every script and source code from the .rbxl file
    pub fn all_scripts(&self) -> Vec<Script<'_>> {
        let mut scripts = Vec::new();
        for instance in self.dom.descendants() {
            let script_type = ScriptType::from_class_name(&instance.class);
            if let ScriptType::Script | ScriptType::LocalScript | ScriptType::ModuleScript =
                script_type
            {
                if let Some(Variant::String(source)) =
                    instance.properties.get(&Ustr::from("Source"))
                {
                    scripts.push(Script {
                        instance: instance,
                        source: source.clone(),
                        script_type: script_type,
                    });
                }
            }
        }
        scripts
    }
}

mod internal {}
