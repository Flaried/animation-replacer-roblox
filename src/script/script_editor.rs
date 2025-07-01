use crate::StudioParser;
use rbx_types::Variant;
use std::collections::HashMap;
use ustr::Ustr;

impl StudioParser {
    /// Updates animation IDs in script source code using the provided mapping.
    pub fn update_script_animations(&mut self, animation_mapping: &HashMap<String, String>) {
        // Collect script refs first to avoid borrow checker issues
        let script_refs = self.get_script_refs();

        // Now modify each script
        for script_ref in script_refs {
            if let Some(instance) = self.dom.get_by_ref_mut(script_ref) {
                if let Some(Variant::String(source)) =
                    instance.properties.get(&Ustr::from("Source"))
                {
                    let mut new_source = source.clone();

                    // Replace animation IDs in the source code
                    for (old_id, new_id) in animation_mapping {
                        new_source = new_source.replace(old_id, new_id);
                    }

                    // Update the source property
                    instance
                        .properties
                        .insert(Ustr::from("Source"), Variant::String(new_source));
                }
            }
        }
    }
}
