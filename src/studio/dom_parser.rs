use crate::{Animation, StudioParser};
use rbx_binary::from_reader;
use rbx_types::Variant;
use regex::Regex;
use roboat::RoboatError;
use roboat::assetdelivery::request_types::AssetBatchResponse;
use std::fs::File;
use std::path::Path;
use ustr::Ustr;

impl StudioParser {
    /// Finds all Animation instances in the workspace and extracts their AnimationIds.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let parser = StudioParser::from_rbxl("~/Desktop/MyPlace.rbxl")?;
    /// let animations = parser.workspace_animations();
    ///
    /// for animation in &animations {
    ///     if let Some(id) = animation.animation_id.strip_prefix("rbxassetid://") {
    ///         println!("Animation ID: {}", id);
    ///     }
    /// }
    /// ```

    pub async fn workspace_animations(&self) -> Result<Vec<AssetBatchResponse>, RoboatError> {
        let re = Regex::new(r"\d+").unwrap();

        let asset_ids: Vec<u64> = self
            .dom
            .descendants()
            .filter(|instance| instance.class == "Animation")
            .filter_map(
                |instance| match instance.properties.get(&Ustr::from("AnimationId")) {
                    Some(Variant::ContentId(content_id)) => re
                        .find(content_id.as_str())
                        .and_then(|mat| mat.as_str().parse::<u64>().ok()),
                    _ => None,
                },
            )
            .collect();

        self.fetch_animation_assets(asset_ids).await
    }

    /// Creates a StudioParser from a .rbxl file. Supports shell expansion (~, environment variables).
    ///
    /// # Examples
    ///
    /// ```rust
    /// let parser = StudioParser::from_rbxl("~/Desktop/MyPlace.rbxl")?;
    /// let scripts = parser.all_scripts();
    /// ```
    pub fn from_rbxl<P: AsRef<Path>>(
        file_path: P,
    ) -> Result<StudioParser, Box<dyn std::error::Error>> {
        let expanded_path = shellexpand::full(file_path.as_ref().to_str().unwrap())?;
        let file = File::open(expanded_path.as_ref())?;
        let dom = from_reader(file)?;
        Ok(StudioParser {
            roblosecurity: None,
            dom,
        })
    }

    /// Creates a builder for fluent configuration with file path and authentication.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let parser = StudioParser::builder()
    ///     .file_path("~/Desktop/MyPlace.rbxl")
    ///     .roblosecurity("your_cookie")
    ///     .build()?;
    /// ```
    pub fn builder() -> StudioParserBuilder {
        StudioParserBuilder::new()
    }
}

/// Builder for creating StudioParser instances with optional authentication.
#[derive(Debug, Default)]
pub struct StudioParserBuilder {
    file_path: Option<String>,
    roblosecurity: Option<String>,
}

impl StudioParserBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn file_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.file_path = Some(path.as_ref().to_string_lossy().to_string());
        self
    }

    /// Sets the Roblosecurity cookie for API authentication.
    /// Required for animation validation and re-uploading features.
    pub fn roblosecurity<S: Into<String>>(mut self, roblosecurity: S) -> Self {
        self.roblosecurity = Some(roblosecurity.into());
        self
    }

    /// Builds the StudioParser. File path is required.
    pub fn build(self) -> Result<StudioParser, Box<dyn std::error::Error>> {
        let file_path = self.file_path.ok_or("File path is required")?;
        let expanded_path = shellexpand::full(&file_path)?;
        let file = File::open(expanded_path.as_ref())?;
        let dom = from_reader(file)?;
        Ok(StudioParser {
            roblosecurity: self.roblosecurity,
            dom,
        })
    }
}
