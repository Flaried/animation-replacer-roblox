use crate::Script;
use crate::ScriptType;
use crate::StudioParser;

use rbx_dom_weak::types::Variant;
use ustr::Ustr;

use regex::Regex;

impl StudioParser {
    pub fn animations_in_scripts(&self) {
        let scripts = self.all_scripts();
        let pattern = Regex::new(r"\d{5,}").unwrap();

        // Scan for 5 digits line by line
        for script in scripts {
            for regex_match in pattern.find_iter(&script.source) {
                println!(
                    "Found animation ID in {}: {}",
                    script.script_type.as_str(),
                    regex_match.as_str()
                );
            }
        }

        println!("hi");
    }

    fn all_scripts(&self) -> Vec<Script> {
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
