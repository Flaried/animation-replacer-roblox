use crate::Script;
use crate::ScriptType;
use crate::StudioParser;
use futures::stream::{self, StreamExt};
use rbx_dom_weak::types::Variant;
use regex::Regex;
use reqwest::Proxy;
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
    // Returns a vector of animation asset IDs found in the script
    pub async fn animations_in_script(
        &self,
        script: &Script<'_>,
    ) -> Result<Vec<AssetBatchResponse>, RoboatError> {
        let mut return_list: Vec<AssetBatchResponse> = Vec::new();

        let pattern = Regex::new(r"\d{5,}").unwrap();
        println!("{:?}", script.source);

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
                .map(|&id| AssetBatchPayload {
                    asset_id: Some(id.to_string()),
                    request_id: Some("0".to_string()),
                    ..Default::default()
                })
                .collect();
            let mut attempts = 0;

            loop {
                match self.check_animation_types(payloads.clone()).await {
                    Ok(Some(mut result)) => {
                        println!("adding to list");
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

        println!("done");
        Ok(return_list)
    }

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
