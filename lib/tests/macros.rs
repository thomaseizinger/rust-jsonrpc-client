use jsonrpc_client::{Id, Response, SendRequest};
use serde::de::DeserializeOwned;
use serde::export::Formatter;
use serde::Serialize;
use serde_json::json;
use std::cell::Cell;
use std::fmt;
use std::fmt::Display;
use std::ops::Deref;
use url::Url;

#[jsonrpc_client::api]
pub trait Math {
    fn subtract(&self, subtrahend: i64, minuend: i64) -> i64;
}

#[derive(Default)]
pub struct InnerClient {
    next_response: Cell<Option<String>>,
    recorded_request: Cell<Option<String>>,
}

impl InnerClient {
    fn with_next_response<P>(response: Response<P>) -> Self
    where
        P: Serialize,
    {
        Self {
            next_response: Cell::new(Some(serde_json::to_string(&response).unwrap())),
            recorded_request: Cell::new(None),
        }
    }

    fn take_recorded_request(&self) -> String {
        self.recorded_request.take().unwrap()
    }
}

#[derive(Debug)]
pub struct DummyError;

impl Display for DummyError {
    fn fmt(&self, _: &mut Formatter<'_>) -> fmt::Result {
        unimplemented!()
    }
}

impl std::error::Error for DummyError {}

impl SendRequest for InnerClient {
    type Error = DummyError;

    fn send_request<P>(&self, _: Url, request: String) -> Result<Response<P>, Self::Error>
    where
        P: DeserializeOwned,
    {
        self.recorded_request.set(Some(request));
        let response = self.next_response.replace(None).unwrap();

        Ok(serde_json::from_str(&response).unwrap())
    }
}

pub struct ExampleDotOrg(Url);

impl Default for ExampleDotOrg {
    fn default() -> Self {
        Self("http://example.org".parse().unwrap())
    }
}

impl Deref for ExampleDotOrg {
    type Target = Url;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn assert_impls_math<C: SendRequest, T: Math<C>>(_: T) {}

mod derive_on_named_inner {
    use crate::{ExampleDotOrg, InnerClient};

    #[jsonrpc_client::r#impl(super::Math)]
    #[derive(Default)]
    pub struct Client {
        pub inner: InnerClient,
        pub base_url: ExampleDotOrg,
    }
}

mod derive_on_named_inner_multiple_fields {
    use crate::{ExampleDotOrg, InnerClient};

    #[jsonrpc_client::r#impl(super::Math)]
    #[derive(Default)]
    pub struct Client {
        inner: InnerClient,
        base_url: ExampleDotOrg,
        _foobar: u64,
    }
}

// TODO: test for attr on multiple fields
// TODO: test for tagged fields
// TODO: test for two APIs

#[test]
fn test_impls_math_api() {
    assert_impls_math(derive_on_named_inner::Client::default());
    assert_impls_math(derive_on_named_inner_multiple_fields::Client::default());
}

#[test]
fn creates_correct_request() {
    let client = derive_on_named_inner::Client {
        inner: InnerClient::with_next_response(Response::new_v2_result(Id::Number(1), json!(1))),
        ..derive_on_named_inner::Client::default()
    };

    let result = client.subtract(5, 4).unwrap();

    assert_eq!(result, 1);
    assert_eq!(
        client.inner.take_recorded_request(),
        r#"{"id":0,"jsonrpc":"2.0","method":"subtract","params":[5,4]}"#
    );
}
