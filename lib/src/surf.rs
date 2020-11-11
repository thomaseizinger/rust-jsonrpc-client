use crate::{Response, SendRequest};
use serde::de::DeserializeOwned;
use std::fmt;
use surf::http::Method;
use url::Url;

#[derive(Debug)]
pub struct Error(pub surf::Error);

impl From<surf::Error> for Error {
    fn from(e: surf::Error) -> Self {
        Error(e)
    }
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[async_trait::async_trait]
impl SendRequest for surf::Client {
    type Error = Error;

    async fn send_request<P>(&self, endpoint: Url, body: String) -> Result<Response<P>, Self::Error>
    where
        P: DeserializeOwned,
    {
        let request = surf::Request::builder(Method::Post, endpoint)
            .body(body)
            .header("Content-type", "application/json")
            .build();

        let response = self.send(request).await?.body_json().await?;

        Ok(response)
    }
}
