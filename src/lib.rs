use rbx_dom_weak::{Instance, WeakDom};

/// A module for uploading animations
pub mod animation;

/// A module for handling parsing and editing studio files
pub mod studio;

/// A module for handling parsing and editing on scripts, in studio files.
pub mod script;

/// A structure to store the Instance and Id of the animation
#[derive(Debug, Clone)]
pub struct Animation<'a> {
    pub instance: &'a Instance,
    pub animation_id: String,
}

impl<'a> Animation<'a> {
    pub fn new(instance: &'a Instance, animation_id: String) -> Self {
        Self {
            instance,
            animation_id,
        }
    }

    pub fn with_info(instance: &'a Instance, animation_id: String) -> Self {
        Self {
            instance,
            animation_id,
        }
    }
}
/// A structure to be impl for the roblox studio DOM
pub struct StudioParser {
    pub roblosecurity: Option<String>,
    pub dom: WeakDom,
}

/// A structure to store the instance, source code and class name of a script
pub struct Script<'a> {
    pub instance: &'a mut Instance,
    pub source: String,
    pub script_type: ScriptType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScriptType {
    Script,
    LocalScript,
    ModuleScript,
    Unknown(String), // fallback for non-standard classes
}
