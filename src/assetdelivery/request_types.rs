use url::form_urlencoded;

pub struct Animation {
    pub id: Option<u64>,
    pub title: String,
    pub description: String,
    pub group_id: Option<u64>,
}
pub fn asset_delivery_url(animation: &Animation) -> String {
    format!(
        "https://assetdelivery.roblox.com/v1/asset/?id={:?}",
        animation.id,
    )
}

pub fn upload_animation_url(animation: &Animation) -> String {
    let mut url = String::from("https://www.roblox.com/ide/publish/uploadnewanimation");

    // Start building query parameters with URL encoding
    let mut query = form_urlencoded::Serializer::new(String::new());
    query.append_pair("assetTypeName", "Animation");
    query.append_pair("name", &animation.title);
    query.append_pair("description", &animation.description);
    query.append_pair("AllID", "1");
    query.append_pair("ispublic", "False");
    query.append_pair("allowComments", "True");
    query.append_pair("isGamesAsset", "False");

    // Noob rust NOTE: "If group_id is Some(x), then bind x to id and run the block."
    if let Some(id) = animation.group_id {
        query.append_pair("groupId", &id.to_string());
    }

    url.push('?');
    url.push_str(&query.finish());

    url
}
