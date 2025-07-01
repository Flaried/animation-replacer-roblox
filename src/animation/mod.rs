use crate::StudioParser;
use roboat::RoboatError;
use roboat::assetdelivery::request_types::AssetBatchResponse;
pub mod uploader;
use crate::animation::uploader::AnimationUploader;

impl StudioParser {
    /// Creates an AnimationUploader from this StudioParser's roblosecurity cookie
    pub fn animation_uploader(&self) -> Result<AnimationUploader, &'static str> {
        match &self.roblosecurity {
            Some(cookie) => Ok(AnimationUploader::new(cookie.clone())),
            None => Err("No roblosecurity cookie set"),
        }
    }

    /// Convenience method to fetch animation assets using the internal cookie
    pub async fn fetch_animation_assets(
        &self,
        asset_ids: Vec<u64>,
    ) -> Result<Vec<AssetBatchResponse>, RoboatError> {
        let uploader = self
            .animation_uploader()
            .map_err(|_| RoboatError::InternalServerError)?;
        uploader.fetch_animation_assets(asset_ids).await
    }
}
