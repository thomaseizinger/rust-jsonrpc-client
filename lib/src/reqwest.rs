use crate::{Response, SendRequest, Url};
use reqwest::header::CONTENT_TYPE;
use serde::de::DeserializeOwned;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    Reqwest(reqwest::Error),
    Decode(serde_json::Error),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Reqwest(inner) => Some(inner),
            Error::Decode(inner) => Some(inner),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Reqwest(e) => write!(f, "{}", e),
            Error::Decode(e) => write!(f, "{}", e),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(inner: reqwest::Error) -> Self {
        Error::Reqwest(inner)
    }
}

impl From<serde_json::Error> for Error {
    fn from(inner: serde_json::Error) -> Self {
        Error::Decode(inner)
    }
}

#[async_trait::async_trait]
impl SendRequest for reqwest::Client {
    type Error = Error;

    async fn send_request<P>(&self, endpoint: Url, body: String) -> Result<Response<P>, Self::Error>
    where
        P: DeserializeOwned,
    {
        #[cfg(feature = "log")]
        {
            log::debug!("POST {} {}", endpoint, body);
        }

        let response = self
            .post(endpoint)
            .header(CONTENT_TYPE, "application/json")
            .body(body)
            .send()
            .await?;

        #[cfg(feature = "log")]
        {
            let text = response.text().await?;
            log::debug!("<-- {}", text);

            return Ok(serde_json::from_str(&text)?);
        }

        #[cfg(not(feature = "log"))]
        {
            Ok(response.json().await?)
        }
    }
}
