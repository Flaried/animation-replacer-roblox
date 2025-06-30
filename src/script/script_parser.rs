use crate::Script;
use crate::ScriptType;
use crate::StudioParser;
use rbx_dom_weak::types::Variant;
use regex::Regex;
use roboat::RoboatError;
use roboat::assetdelivery::request_types::AssetBatchResponse;
use std::collections::HashSet;
use ustr::Ustr;

impl StudioParser {
    /// Returns a vector of AssetBatchResponse (Animation Details from batch API) found in the script
    /// # Notes:
    /// Takes in a script, scans all the IDs into it then has batch_sizes of 250.
    /// It posts 250 Ids at a time to the asset batch API then filters out everything but
    /// animations.
    /// * Requires a cookie
    /// * Batch API does hang sometimes, fixed that with retries and 3 second timeout.
    pub async fn all_animations_in_scripts(
        &mut self,
    ) -> Result<Vec<AssetBatchResponse>, RoboatError> {
        println!("Fetching animations in scripts.");
        let scripts = self.all_scripts();
        let pattern = Regex::new(r"\d{5,}").unwrap();

        // Collect and deduplicate all IDs from all scripts
        let mut all_ids: HashSet<u64> = HashSet::new();
        for script in &scripts {
            let ids_in_script = pattern
                .find_iter(&script.source)
                .filter_map(|m| m.as_str().parse::<u64>().ok());
            all_ids.extend(ids_in_script);
        }

        // Convert to Vec and fetch assets
        let id_list: Vec<u64> = all_ids.into_iter().collect();
        println!("Got all animations from script... Sending them to Roblox API");
        self.fetch_animation_assets(id_list).await
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
