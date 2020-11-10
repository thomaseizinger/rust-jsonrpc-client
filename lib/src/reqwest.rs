use crate::{Response, SendRequest};
use reqwest::{header::CONTENT_TYPE, Url};
use serde::de::DeserializeOwned;

#[async_trait::async_trait]
impl SendRequest for reqwest::Client {
    type Error = reqwest::Error;

    async fn send_request<P>(&self, endpoint: Url, body: String) -> Result<Response<P>, Self::Error>
    where
        P: DeserializeOwned,
    {
        Ok(self
            .post(endpoint)
            .header(CONTENT_TYPE, "application/json")
            .body(body)
            .send()
            .await?
            .json()
            .await?)
    }
}
