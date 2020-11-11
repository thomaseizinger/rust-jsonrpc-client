use jsonrpc_client::{Response, SendRequest, Url};
use serde::de::DeserializeOwned;

#[jsonrpc_client::api]
pub trait Math {
    async fn subtract(&self, subtrahend: i64, minuend: i64) -> i64;
}

struct InnerClient;

#[async_trait::async_trait]
impl SendRequest for InnerClient {
    type Error = std::io::Error;

    async fn send_request<P>(&self, endpoint: Url, body: String) -> Result<Response<P>, Self::Error>
    where
        P: DeserializeOwned,
    {
        unimplemented!()
    }
}

#[jsonrpc_client::implement(Math)]
pub struct Client {
    inner_client: InnerClient,
}

fn main() {}
