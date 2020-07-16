use jsonrpc_client::{Id, Request, Response, ResponsePayload, SendRequest};
use serde::de::DeserializeOwned;
use std::any::Any;
use std::cell::Cell;
use std::convert::Infallible;

#[jsonrpc_client::api]
pub trait Math {
    fn subtract(&self, subtrahend: i64, minuend: i64) -> i64;
}

struct Client {
    next_response: Cell<Option<Box<dyn Any>>>,
}

impl Math for Client {}

impl Client {
    fn new() -> Self {
        Self {
            next_response: Cell::new(None),
        }
    }

    fn set_next_response<R>(&self, response: Response<R>)
    where
        R: 'static,
    {
        self.next_response.set(Some(Box::new(response)));
    }
}

impl SendRequest for Client {
    type Error = Infallible;

    fn send_request<Res>(&self, _: Request) -> Result<Response<Res>, Self::Error>
    where
        Res: DeserializeOwned + 'static,
    {
        let response = self.next_response.replace(None).unwrap();
        let response = *response.downcast::<Response<Res>>().unwrap();

        Ok(response)
    }
}

#[test]
fn subtract() {
    let client = Client::new();

    client.set_next_response(Response {
        id: Id::Number(1),
        jsonrpc: "2.0".to_string(),
        payload: ResponsePayload::Result(1i64),
    });

    let result = client.subtract(5, 4).unwrap();

    assert_eq!(result, 1)
}
