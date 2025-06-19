use crate::{Animation, StudioParser};
use rbx_binary::from_reader;
use rbx_types::Variant;
use std::fs::File;
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

    pub fn from_rbxl(file_name: String) -> Result<StudioParser, Box<dyn std::error::Error>> {
        let file = File::open(file_name)?;
        let dom = from_reader(file)?;

        Ok(StudioParser { dom })
    }
}

