use jsonrpc_client::{Request, Response};

#[jsonrpc_client::api]
pub trait Math {
    fn subtract(&self, subtrahend: i64, minuend: i64) -> i64;
}

struct Client {
    inner: reqwest::blocking::Client,
}

impl jsonrpc_client::SendRequest for Client {
    type Error = reqwest::Error;

    fn send_request(&self, request: Request) -> Result<Response, Self::Error> {
        self.inner
            .post("http://example.org")
            .json(&request)
            .send()?
            .json()
    }
}

impl Math for Client {}

fn main() {}
