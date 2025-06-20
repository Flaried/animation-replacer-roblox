use roboat::ClientBuilder;
use roboat::RoboatError;
use roboat::ide::ide_types::NewAnimation;

use crate::StudioParser;

impl StudioParser {
    pub async fn reupload_animation(&self, asset_id: u64) -> Result<(), RoboatError> {
        let client = ClientBuilder::new()
            // Gonna error if theres no cookie
            .roblosecurity(
                self.roblosecurity
                    .clone()
                    .expect("roblosecurity cookie required"),
            )
            .build();

        let existing_animation = client.fetch_asset_data(asset_id).await?;

        let animation = NewAnimation {
            group_id: None,
            name: "roboatTest".to_string(),
            description: "This is a roboat example".to_string(),
            animation_data: existing_animation,
        };

        client.upload_new_animation(animation).await?;

        println!("Uploaded Animation!");
        Ok(())
    }
}
