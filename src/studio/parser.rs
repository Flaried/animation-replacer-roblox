use crate::{Animation, StudioParser};
use rbx_binary::from_reader;
use rbx_types::Variant;
use std::fs::File;
use std::path::Path;
use ustr::Ustr;

impl StudioParser {
    pub fn workspace_animations(&self) -> Vec<Animation> {
        let mut animations = Vec::new();

        for instance in self.dom.descendants() {
            if instance.class == "Animation" {
                if let Some(instance_type) = instance.properties.get(&Ustr::from("AnimationId")) {
                    if let Variant::ContentId(content_id) = instance_type {
                        let url = content_id.as_str();
                        if !url.is_empty() {
                            animations.push(Animation {
                                instance,
                                animation_id: url.to_string(),
                            });
                        }
                    }
                }
            }
        }
        animations
    }

    pub fn from_rbxl<P: AsRef<Path>>(
        file_path: P,
    ) -> Result<StudioParser, Box<dyn std::error::Error>> {
        // Expand tilde and environment variables
        let expanded_path = shellexpand::full(file_path.as_ref().to_str().unwrap())?;

        // Open the file using the expanded path
        let file = File::open(expanded_path.as_ref())?;

        let dom = from_reader(file)?;

        Ok(StudioParser { dom })
    }
}
