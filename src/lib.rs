use rbx_dom_weak::{Instance, WeakDom};

/// A module for uploading animations
pub mod animation;

/// A module for handling parsing and editing studio files
pub mod studio;

/// A structure to store the Instance and Id of the animation
pub struct Animation<'a> {
    pub instance: &'a Instance,
    pub animation_id: String,
}

/// A structure to be impl for the roblox studio DOM
pub struct StudioParser {
    pub dom: WeakDom,
}

// A structure to store the instance, source code and class name of a script
pub struct Script<'a> {
    pub instance: &'a Instance,
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
