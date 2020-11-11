use crate::{Response, SendRequest};
use awc::{
    error::{JsonPayloadError, SendRequestError},
    http::header::CONTENT_TYPE,
};
use serde::de::DeserializeOwned;
use std::fmt;
use url::Url;

#[derive(Debug)]
pub enum Error {
    SendRequest(SendRequestError),
    DecodeJson(JsonPayloadError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::SendRequest(inner) => fmt::Display::fmt(inner, f),
            Error::DecodeJson(inner) => fmt::Display::fmt(inner, f),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::SendRequest(inner) => Some(inner),
            Error::DecodeJson(inner) => Some(inner),
        }
    }
}

impl From<SendRequestError> for Error {
    fn from(e: SendRequestError) -> Self {
        Error::SendRequest(e)
    }
}

impl From<JsonPayloadError> for Error {
    fn from(e: JsonPayloadError) -> Self {
        Error::DecodeJson(e)
    }
}

#[async_trait::async_trait]
impl SendRequest for awc::Client {
    type Error = Error;

    async fn send_request<P>(&self, endpoint: Url, body: String) -> Result<Response<P>, Self::Error>
    where
        P: DeserializeOwned,
    {
        let response = self
            .post(endpoint.to_string())
            .header(CONTENT_TYPE, "application/json")
            .send_body(body)
            .await?
            .json()
            .await?;

        Ok(response)
    }
}
