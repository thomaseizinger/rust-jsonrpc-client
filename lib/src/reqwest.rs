use crate::{Response, SendRequest};
use reqwest::{header::CONTENT_TYPE, Url};
use serde::de::DeserializeOwned;

impl SendRequest for reqwest::blocking::Client {
    type Error = reqwest::Error;

    fn send_request<P>(&self, endpoint: Url, body: String) -> Result<Response<P>, Self::Error>
    where
        P: DeserializeOwned,
    {
        Ok(self
            .post(endpoint)
            .header(CONTENT_TYPE, "application/json")
            .body(body)
            .send()?
            .json()?)
    }
}
