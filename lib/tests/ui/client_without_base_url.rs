use jsonrpc_client::{Error, Response, SendRequest, Url};
use serde::de::DeserializeOwned;
use std::fmt;

#[jsonrpc_client::api]
pub trait Math {
    async fn subtract(&self, subtrahend: i64, minuend: i64) -> i64;
}

struct InnerClient;

#[derive(Debug)]
pub struct DummyError;

impl fmt::Display for DummyError {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        unimplemented!()
    }
}

impl std::error::Error for DummyError {}

#[async_trait::async_trait]
impl SendRequest for InnerClient {
    type Error = DummyError;

    async fn send_request<P>(&self, endpoint: Url, body: String) -> Result<Response<P>, Self::Error>
    where
        P: DeserializeOwned,
    {
        unimplemented!()
    }
}

impl From<DummyError> for Error<DummyError> {
    fn from(inner: DummyError) -> Self {
        Self::Client(inner)
    }
}

#[jsonrpc_client::implement(Math)]
pub struct Client {
    inner: InnerClient,
}

fn main() {}
