use crate::XCSRF_HEADER;
use crate::roblox_api::errors::RobloxError;
use crate::roblox_api::roblox_client::RobloxSession;
use reqwest::header;

impl RobloxSession {
    pub async fn refresh_csrf(&self) -> Result<(), RobloxError> {
        let builder = self
            .reqwest_client
            .post("https://auth.roblox.com/v2/login")
            .header(XCSRF_HEADER, self.read_xcsrf().await);

        // Add the roblosecurity if it exists.
        let builder = match self.read_cookie() {
            Ok(cookie_string) => builder.header(header::COOKIE, cookie_string),
            Err(_) => builder,
        };

        let resp = builder.send().await;

        match Self::validate_request_result(resp).await {
            Ok(_) => Ok(()), // Successful, csrf is valid.
            Err(RobloxError::InvalidXcsrf(xcsrf)) => {
                println!("Hi {:?}", xcsrf);
                self.set_xcsrf(xcsrf).await;
                Ok(()) // CSRF refreshed and set.
            }
            Err(e) => Err(e), // Any other errors are propagated.
        }
    }
}
