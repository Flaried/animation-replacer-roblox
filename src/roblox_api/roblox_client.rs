use crate::roblox_api::errors::RobloxError;
use reqwest::{
    Client as ReqwestClient, Error,
    header::{COOKIE, HeaderMap, HeaderName, HeaderValue},
};

use tokio::sync::RwLock;

#[derive(Debug, Default)]
pub struct RobloxSession {
    pub(crate) cookie_string: Option<HeaderValue>,
    pub(crate) xcsrf: RwLock<String>,
    pub(crate) reqwest_client: reqwest::Client,
}

#[derive(Clone, Debug, Default)]
pub struct SessionBuilder {
    roblosecurity: Option<String>,
    reqwest_client: Option<reqwest::Client>,
}

// https://rust-unofficial.github.io/patterns/patterns/creational/builder.html
impl SessionBuilder {
    // Self would be SessionBuilder
    pub fn new() -> Self {
        Self::default()
    }

    pub fn roblosecurity(mut self, roblosecurity: String) -> Self {
        self.roblosecurity = Some(roblosecurity);
        self
    }

    pub fn build(self) -> Result<RobloxSession, RobloxError> {
        let cookie_string = match self.roblosecurity {
            Some(roblosecurity) => Some(RobloxSession::create_cookie_string_header(&roblosecurity)),
            None => None,
        };

        let reqwest_client = self
            .reqwest_client
            .unwrap_or_else(|| reqwest::Client::new());

        Ok(RobloxSession {
            cookie_string,
            xcsrf: RwLock::new(String::new()),
            reqwest_client,
        })
    }
}

impl RobloxSession {
    pub(crate) async fn read_xcsrf(&self) -> String {
        self.xcsrf.read().await.clone()
    }

    pub(crate) fn read_cookie(&self) -> Result<HeaderValue, RobloxError> {
        let cookie_string_opt = &self.cookie_string;

        match cookie_string_opt {
            Some(cookie) => Ok(cookie.clone()),
            None => Err(RobloxError::RoblosecurityNotSet),
        }
    }

    pub(crate) async fn set_xcsrf(&self, xcsrf: String) {
        *self.xcsrf.write().await = xcsrf;
    }

    fn create_cookie_string_header(roblosecurity: &str) -> HeaderValue {
        let mut header = HeaderValue::from_str(&format!(".ROBLOSECURITY={}", roblosecurity))
            .expect("Invalid roblosecurity characters.");
        header.set_sensitive(true);
        header
    }
}
