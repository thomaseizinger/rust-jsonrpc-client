use jsonrpc_client::{Id, Request, Response, SendRequest};
use serde_json::json;
use std::cell::Cell;
use std::convert::Infallible;

#[jsonrpc_client::api]
pub trait Math {
    fn subtract(&self, subtrahend: i64, minuend: i64) -> i64;
}

struct Client {
    next_response: Cell<Option<Response>>,
    recorded_request: Cell<Option<Request>>,
}

impl Math for Client {}

impl Client {
    fn with_next_response(response: Response) -> Self {
        Self {
            next_response: Cell::new(Some(response)),
            recorded_request: Cell::new(None),
        }
    }

    fn take_recorded_request(&self) -> Request {
        self.recorded_request.take().unwrap()
    }
}

impl SendRequest for Client {
    type Error = Infallible;

    fn send_request(&self, request: Request) -> Result<Response, Self::Error> {
        self.recorded_request.set(Some(request));
        let response = self.next_response.replace(None).unwrap();

        Ok(response)
    }
}

#[test]
fn creates_correct_request() {
    let client = Client::with_next_response(Response::new_v2_result(Id::Number(1), json!(1)));

    let result = client.subtract(5, 4).unwrap();

    assert_eq!(result, 1);
    assert_eq!(
        client.take_recorded_request(),
        Request::new_v2("subtract", vec![json!(5), json!(4)])
    );
}
