/// A module for uploading animations
pub mod animation_uploader;

/// A module for handling parsing and editing studio files
pub mod studio;

use rbx_dom_weak::{Instance, WeakDom};

pub struct Animation<'a> {
    pub instance: &'a Instance,
    pub animation_id: String,
}

pub struct StudioParser {
    pub dom: WeakDom,
}
