use crate::AnimationUploader;
use roboat::ClientBuilder;

// impl AnimationUploader {
/// Retrieves the place ID from the first game owned by an asset's creator.
/// Used for adding place_id to header for uploading animatiions
///
/// # Endpoint
/// This method makes multiple API calls:
/// 1. `GET` to asset info endpoint to determine the creator
/// 2. `GET` to `https://games.roblox.com/v2/users/{user_id}/games?limit=50` (for user creators)
/// 3. `GET` to `https://games.roblox.com/v2/groups/{group_id}/gamesv2?limit=100` (for group creators)
///
/// # Notes
/// * Requires a valid `.ROBLOSECURITY` cookie for authentication to fetch asset info.
/// * The games endpoints are public and don't require authentication.
/// * Returns the root place ID of the creator's first game (most recent by default).
/// * Supports both user and group creators.
/// * Useful for determining where to upload related content (like animations) to the same creator's place.
///
/// # Parameters
/// * `asset_id` – The numeric asset ID to look up the creator for
///
/// # Return Value Notes
/// * Returns `u64` representing the place ID of the creator's first game if successful.
/// * The place ID can be used for uploading assets or other operations that require a place context.
///
/// # Errors
/// * Returns `anyhow::Error` for various failure conditions:
///   - Network issues when fetching asset info or games
///   - Invalid or non-existent asset ID
///   - Creator has no games available
///   - Malformed responses from Roblox APIs
///   - Missing or invalid creator information
///   - Authentication issues when fetching asset info
//     pub async fn asset_place_id(&self, asset_id: u64) -> anyhow::Result<u64> {
//         let client = ClientBuilder::new()
//             .roblosecurity(self.roblosecurity.clone())
//             .build();
//         match client.get_asset_info(asset_id).await {
//             Ok(x) => {
//                 let creator = &x.creation_context.creator;
//                 if let Some(group_id) = &creator.group_id {
//                     let id: u64 = group_id.parse().expect("Failed to parse owner: group int");
//
//                     let place_id = self.fetch_group_place(id).await?;
//                     return Ok(place_id);
//                 } else if let Some(user_id) = &creator.user_id {
//                     let id: u64 = user_id.parse().expect("Failed to parse owner: user_id int");
//                     let place_id = self.fetch_user_place(id).await?;
//                     return Ok(place_id);
//                 } else {
//                     Err(anyhow::anyhow!(
//                         "No valid creator found for asset {}",
//                         asset_id
//                     ))
//                 }
//             }
//             Err(e) => Err(anyhow::anyhow!("Failed to get asset info: {}", e)),
//         }
//     }
// }

mod internal {
    use crate::AnimationUploader;
    use roboat::ClientBuilder;

    impl AnimationUploader {
        pub(super) async fn fetch_user_place(&self, user_id: u64) -> anyhow::Result<u64> {
            let client = ClientBuilder::new().build();
            let games_response = client.user_games(user_id).await?;
            if let Some(first_place) = games_response.data.first() {
                Ok(first_place.root_place.id)
            } else {
                Err(anyhow::anyhow!("Couldn't find place for user {}", user_id))
            }
        }

        pub(super) async fn fetch_group_place(&self, group_id: u64) -> anyhow::Result<u64> {
            let client = ClientBuilder::new().build();
            let games_response = client.group_games(group_id).await?;
            if let Some(first_place) = games_response.data.first() {
                Ok(first_place.root_place.id)
            } else {
                Err(anyhow::anyhow!(
                    "Couldn't find place for group {}",
                    group_id
                ))
            }
        }
    }
}
