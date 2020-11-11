use crate::{Response, SendRequest, Url};
use isahc::{
    http::{
        header::{HeaderValue, CONTENT_TYPE},
        Request,
    },
    ResponseExt,
};
use serde::de::DeserializeOwned;

#[async_trait::async_trait]
impl SendRequest for isahc::HttpClient {
    type Error = isahc::Error;

    async fn send_request<P>(&self, endpoint: Url, body: String) -> Result<Response<P>, Self::Error>
    where
        P: DeserializeOwned,
    {
        let mut request = Request::post(endpoint.to_string()).body(body)?;
        request
            .headers_mut()
            .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let response = self
            .send_async(request)
            .await?
            .json()
            .map_err(|e| isahc::Error::ResponseBodyError(Some(e.to_string())))?;

        Ok(response)
    }
}
