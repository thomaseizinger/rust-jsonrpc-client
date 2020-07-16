use jsonrpc_client::{Id, Request, Response, ResponsePayload, SendRequest};
use serde_json::json;
use std::cell::Cell;
use std::convert::Infallible;

#[jsonrpc_client::api]
pub trait Math {
    fn subtract(&self, subtrahend: i64, minuend: i64) -> i64;
}

struct Client {
    next_response: Cell<Option<Response>>,
}

impl Math for Client {}

impl Client {
    fn new() -> Self {
        Self {
            next_response: Cell::new(None),
        }
    }

    fn set_next_response(&self, response: Response) {
        self.next_response.set(Some(response));
    }
}

impl SendRequest for Client {
    type Error = Infallible;

    fn send_request(&self, _: Request) -> Result<Response, Self::Error> {
        let response = self.next_response.replace(None).unwrap();

        Ok(response)
    }
}

#[test]
fn subtract() {
    let client = Client::new();

    client.set_next_response(Response {
        id: Id::Number(1),
        jsonrpc: "2.0".to_string(),
        payload: ResponsePayload::Result(json!(1)),
    });

    let result = client.subtract(5, 4).unwrap();

    assert_eq!(result, 1)
}
