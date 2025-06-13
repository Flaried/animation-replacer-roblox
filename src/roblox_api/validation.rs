use crate::{
    XCSRF_HEADER,
    roblox_api::{errors::RobloxError, roblox_client::RobloxSession},
};
use reqwest::Response;
use serde::{Deserialize, Serialize};

#[allow(missing_docs)]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
struct RobloxErrorResponse {
    pub errors: Vec<RobloxErrorRaw>,
}

#[allow(missing_docs)]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
struct RobloxErrorRaw {
    pub code: u16,
    pub message: String,
}

impl RobloxSession {
    // This function gets called when you get 403; Unauthorized or challenge required.
    async fn process_unauth(resp: Response) -> RobloxError {
        let headers = resp.headers().clone();
        let xcsrf = headers
            .get(XCSRF_HEADER)
            .and_then(|x| x.to_str().ok())
            .map(|s| s.to_string());

        match resp.json::<RobloxErrorResponse>().await {
            Ok(x) => match x.errors.first() {
                Some(error) if error.code == 0 => {
                    xcsrf.map_or(RobloxError::XcsrfNotReturned, RobloxError::InvalidXcsrf)
                }
                Some(error)
                    if error.message != "Challenge is required to authorize the request" =>
                {
                    RobloxError::UnknownRobloxErrorCode {
                        code: error.code,
                        message: error.message.clone(),
                    }
                }
                _ => RobloxError::UnknownStatus403Format,
            },
            Err(_) => xcsrf.map_or(RobloxError::XcsrfNotReturned, RobloxError::InvalidXcsrf),
        }
    }

    pub async fn handle_status(resp: Response) -> Result<Response, RobloxError> {
        let status_code = resp.status().as_u16();

        match status_code {
            200 => Ok(resp),
            400 => Err(Self::process_unauth(resp).await),
            _ => Err(Self::process_unauth(resp).await),
        }
    }
}
