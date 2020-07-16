use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::error::Error as StdError;
use std::fmt;
use std::fmt::Debug;

pub use jsonrpc_client_macro::*;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum Id {
    Number(i64),
    String(String),
}

#[derive(Serialize, Debug, Clone)]
pub struct Request<'p> {
    pub id: Id,
    pub jsonrpc: String,
    pub method: String,
    pub params: &'p [serde_json::Value],
}

impl<'p> Request<'p> {
    pub fn new(id: Id, method: &str, params: &'p [serde_json::Value]) -> Self {
        Self {
            id,
            jsonrpc: "2.0".to_owned(),
            method: method.to_owned(),
            params,
        }
    }
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Response<T> {
    pub id: Id,
    pub jsonrpc: String,
    #[serde(flatten)]
    pub payload: ResponsePayload<T>,
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ResponsePayload<T> {
    Result(T),
    Error(JsonRpcError),
}

impl<T> From<ResponsePayload<T>> for Result<T, JsonRpcError> {
    fn from(payload: ResponsePayload<T>) -> Self {
        match payload {
            ResponsePayload::Result(result) => Ok(result),
            ResponsePayload::Error(e) => Err(e),
        }
    }
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
}

impl fmt::Display for JsonRpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "JSON-RPC request failed with code {}: {}",
            self.code, self.message
        )
    }
}

impl StdError for JsonRpcError {}

#[derive(Debug)]
pub enum Error<C> {
    Client(C),
    JsonRpc(JsonRpcError),
    Serde(serde_json::Error),
}

impl<C> fmt::Display for Error<C>
where
    C: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Client(client_error) => fmt::Display::fmt(client_error, f),
            Error::JsonRpc(jsonrpc_error) => fmt::Display::fmt(jsonrpc_error, f),
            Error::Serde(serde_error) => fmt::Display::fmt(serde_error, f),
        }
    }
}

impl<C> From<serde_json::Error> for Error<C> {
    fn from(serde_error: serde_json::Error) -> Self {
        Error::Serde(serde_error)
    }
}

impl<C> From<JsonRpcError> for Error<C> {
    fn from(jsonrpc_error: JsonRpcError) -> Self {
        Error::JsonRpc(jsonrpc_error)
    }
}

impl<C> StdError for Error<C> where C: StdError {}

pub trait SendRequest {
    type Error: StdError;

    fn send_request<Res>(&self, request: Request) -> Result<Response<Res>, Self::Error>
    where
        Res: DeserializeOwned + 'static;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn deserialize_error_response() {
        let json = r#"{"jsonrpc": "2.0", "error": {"code": -32601, "message": "Method not found"}, "id": "1"}"#;

        let response = serde_json::from_str::<Response<()>>(json).unwrap();

        assert_eq!(
            response,
            Response {
                id: Id::String("1".to_owned()),
                jsonrpc: "2.0".to_owned(),
                payload: ResponsePayload::Error(JsonRpcError {
                    code: -32601,
                    message: "Method not found".to_owned()
                })
            }
        )
    }

    #[test]
    fn deserialize_success_response() {
        let json = r#"{"jsonrpc": "2.0", "result": 19, "id": 1}"#;

        let response = serde_json::from_str::<Response<u64>>(json).unwrap();

        assert_eq!(
            response,
            Response {
                id: Id::Number(1),
                jsonrpc: "2.0".to_owned(),
                payload: ResponsePayload::Result(19)
            }
        )
    }

    #[test]
    fn serialize_request() {
        let request = Request::new(Id::Number(1), "subtract", vec![json!(42), json!(23)]);

        let json = serde_json::to_string(&request).unwrap();

        assert_eq!(
            json,
            r#"{"id":1,"jsonrpc":"2.0","method":"subtract","params":[42,23]}"#
        )
    }
}
