use crate::AnimationUploader;
use crate::StudioParser;
use roboat::RoboatError;
use roboat::assetdelivery::AssetBatchResponse;

pub mod uploader;

// Implement uploader code into the studio struct
impl StudioParser {
    pub fn animation_uploader(&self) -> Result<AnimationUploader, RoboatError> {
        match &self.roblosecurity {
            Some(cookie) => Ok(AnimationUploader::new(cookie.clone())),
            None => Err(RoboatError::InvalidRoblosecurity),
        }
    }

    pub async fn fetch_animation_assets(
        &self,
        asset_ids: Vec<u64>,
    ) -> Result<Vec<AssetBatchResponse>, RoboatError> {
        let uploader = self.animation_uploader()?;
        uploader.fetch_animation_assets(asset_ids).await
    }
}
