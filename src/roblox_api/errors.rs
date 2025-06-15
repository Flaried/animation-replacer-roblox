use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Debug, Default)]
pub enum RobloxError {
    #[default]
    #[error("Roblosecurity Not Set")]
    RoblosecurityNotSet,

    #[error("Unknown Roblox Error Code {code}: {message}")]
    UnknownRobloxErrorCode { code: u16, message: String },

    #[error("Invalid Xcsrf. New Xcsrf Contained In Error.")]
    InvalidXcsrf(String),

    #[error("Missing Xcsrf")]
    XcsrfNotReturned,

    #[error(
        "Unknown Status Code 403 Format. If this occurs often it may be a bug. Please report it to the issues page."
    )]
    UnknownStatus403Format,

    /// Used for any reqwest error that occurs.
    #[error("RequestError {0}")]
    ReqwestError(reqwest::Error),

    #[error("Ratelimited. Sending too many requests")]
    TooManyRequests,

    #[error("Request Failed InternalServerError bad payload")]
    InternalServerError,
}
