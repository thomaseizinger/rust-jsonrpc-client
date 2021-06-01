use crate::{Response, SendRequest, Url};
use serde::de::DeserializeOwned;

#[async_trait::async_trait]
impl SendRequest for ureq::Agent {
    type Error = ureq::Error;

    async fn send_request<P>(&self, endpoint: Url, body: String) -> Result<Response<P>, Self::Error>
    where
        P: DeserializeOwned,
    {
        Ok(self
            .post(&endpoint.to_string())
            .send_json(ureq::json!(&body))?
            .into_json::<Response<P>>()?)
    }
}

impl From<ureq::Error> for crate::Error<ureq::Error> {
    fn from(inner: ureq::Error) -> Self {
        crate::Error::Client(inner)
    }
}
