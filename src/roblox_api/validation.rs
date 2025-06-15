use crate::{
    XCSRF_HEADER,
    roblox_api::{errors::RobloxError, roblox_client::RobloxSession},
};
use reqwest::{
    Response,
    header::{self, HeaderMap},
};
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
    // This function gets called when you get a 403 Unauthorized or challenge required.
    async fn process_unauth(resp: Response) -> RobloxError {
        let headers = resp.headers().clone();
        let xcsrf = headers
            .get(XCSRF_HEADER)
            .and_then(|x| x.to_str().ok())
            .map(|s| s.to_string());

        match resp.json::<RobloxErrorResponse>().await {
            Ok(x) => match x.errors.first() {
                Some(error) if error.code == 0 => {
                    println!("{:?}", x);
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
            Err(_) => {
                let xcsrf = headers
                    .get(XCSRF_HEADER)
                    .map(|x| x.to_str().unwrap().to_string());
                match xcsrf {
                    Some(x) => RobloxError::InvalidXcsrf(x),
                    None => RobloxError::XcsrfNotReturned,
                }
            }
        }
    }

    pub async fn handle_status(resp: Response) -> Result<Response, RobloxError> {
        let status_code = resp.status().as_u16();
        println!("Trying to handle: {:?}", status_code);

        // TODO: Add more status codes
        match status_code {
            200 => Ok(resp),                                    // Success!
            400 | 403 => Err(Self::process_unauth(resp).await), // Unauthorized or challenge
            429 => Err(RobloxError::TooManyRequests),
            500 => Err(RobloxError::InternalServerError),
            _ => Err(RobloxError::UnknownRobloxErrorCode {
                code: status_code,
                message: "".to_string(),
            }), // Handle other statuses similarly
        }
    }

    pub(crate) async fn validate_request_result(
        request_result: Result<Response, reqwest::Error>,
    ) -> Result<Response, RobloxError> {
        match request_result {
            Ok(response) => Self::handle_status(response).await,
            Err(e) => Err(RobloxError::ReqwestError(e)),
        }
    }
}
