use crate::StudioParser;
use rbx_binary::to_writer;
use rbx_types::Variant;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use ustr::Ustr;

impl StudioParser {
    /// Gets all animation instances in the file, scans their animationId then replaces them if a
    /// new one is provided.
    ///
    pub fn update_game_animations(&mut self, animation_mapping: &HashMap<String, String>) {
        let animation_instances_referent: Vec<_> = self
            .dom
            .descendants()
            .filter(|instance| {
                let instance_class_str = instance.class.as_str();
                let is_animation = matches!(instance_class_str, "Animation");
                is_animation
            })
            .map(|instance| instance.referent())
            .collect();
        // println!(
        //     "Going through workplaces: {:?}",
        //     animation_instances_referent.len()
        // );

        let animation_id_key = Ustr::from("AnimationId");
        for animation_ref in animation_instances_referent {
            if let Some(instance) = self.dom.get_by_ref_mut(animation_ref) {
                if let Some(Variant::ContentId(content_id)) =
                    instance.properties.get(&animation_id_key)
                {
                    let raw = content_id.as_str();
                    let trimmed_id = raw.strip_prefix("rbxassetid://").unwrap_or(raw);
                    if let Some(new_id) = animation_mapping.get(trimmed_id) {
                        // Replace the AnimationId with the new one
                        let rbxasset = format!("rbxassetid://{}", new_id);
                        instance.properties.insert(
                            animation_id_key,
                            Variant::ContentId(rbxasset.clone().into()),
                        );
                    }
                }
            }
        }
    }

    /// Saves the DOM to a .rbxl file.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let parser = StudioParser::builder()
    ///     .file_path("input.rbxl")
    ///     .build()?;
    /// parser.save_to_rbxl("output.rbxl")?;
    /// ```
    pub fn save_to_rbxl<P: AsRef<Path>>(&self, file_path: P) -> Result<(), anyhow::Error> {
        let expanded_path = shellexpand::full(file_path.as_ref().to_str().unwrap())?;
        let file = File::create(expanded_path.as_ref())?;

        // Get the children of the root instead of the root
        let root_children = self.dom.get_by_ref(self.dom.root_ref()).unwrap().children();

        to_writer(file, &self.dom, root_children)?;
        Ok(())
    }
}
