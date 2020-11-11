//! A macro-driven JSON-RPC client.
//!
//! This crate offers a macro-driven approach to interacting with JSON-RPC APIs.
//! JSON-RPC itself is pretty lightweight, yet it is pretty verbose if you have to roll a client from scratch on top of say `reqwest`.
//!
//! This crate abstracts away this boilerplate by allowing you to define a trait that resembles the API you want to talk to.
//! At the same time, we give full control the user over which HTTP client they want to use to actually send the request.
//!
//! # Example
//!
//! ```rust,no_run
//! # use anyhow::Result;
//! #[jsonrpc_client::api]
//! pub trait Math {
//!     async fn subtract(&self, subtrahend: i64, minuend: i64) -> i64;
//! }
//!
//! #[jsonrpc_client::implement(Math)]
//! struct Client {
//!     inner: reqwest::Client,
//!     base_url: reqwest::Url,
//! }
//! # impl Client {
//! #     fn new(base_url: String) -> Result<Self> {
//! #        Ok(Self {
//! #            inner: reqwest::Client::new(),
//! #            base_url: base_url.parse()?,
//! #        })
//! #    }
//! # }
//! # #[tokio::main]
//! # async fn main() -> Result<()> {
//!
//! let client = Client::new("http://example-jsonrpc.org/".to_owned())?;
//!
//! client.subtract(10, 5).await?;
//! #
//! #    Ok(())
//! # }
//! ```

#[cfg(feature = "reqwest")]
mod reqwest;

#[cfg(feature = "surf")]
pub mod surf;

#[cfg(feature = "isahc")]
mod isahc;

#[cfg(feature = "awc")]
pub mod awc;

/// Define the API of the JSON-RPC server you want to talk to.
///
/// All methods of this trait must be `async`. Additionally, the trait cannot have other items such as `const` or `type` declarations.
/// You can define the JSON-RPC version through the `version` attribute. For now, all this does is sent the correct version property in the JSON-RPC request.
///
/// # Example
///
/// ```
/// #[jsonrpc_client::api]
/// pub trait Math {
///     async fn subtract(&self, subtrahend: i64, minuend: i64) -> i64;
/// }
/// ```
pub use jsonrpc_client_macro::api;

/// Implement a given API trait on this client.
///
/// The client needs to have at least two fields:
///
/// - the "inner" client that is used to dispatch the request
/// - the "base_url" of the server the request should be sent to
///
/// If these fields are literally named `inner` and `base_url`, then they will be automatically detected by this macro.
/// If you wish to use alternative names, you can use the attributes `#[jsonrpc_client(inner)]` and `#[jsonrpc_client(base_url)]` to mark them accordingly.
///
/// # Example
///
/// ```rust,no_run
/// # use anyhow::Result;
/// # #[jsonrpc_client::api]
/// # pub trait Math {
/// #    async fn subtract(&self, subtrahend: i64, minuend: i64) -> i64;
/// # }
/// #[jsonrpc_client::implement(Math)]
/// struct Client {
///     #[jsonrpc_client(inner)]
///     my_client: reqwest::Client,
///     #[jsonrpc_client(base_url)]
///     url: reqwest::Url,
/// }
/// # impl Client {
/// #     fn new(base_url: String) -> Result<Self> {
/// #        Ok(Self {
/// #            my_client: reqwest::Client::new(),
/// #            url: base_url.parse()?,
/// #        })
/// #    }
/// # }
/// ```
pub use jsonrpc_client_macro::implement;

pub use url::Url;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    error::Error as StdError,
    fmt::{self, Debug},
};

/// The ID of a JSON-RPC request.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum Id {
    Number(i64),
    String(String),
}

/// The JSON-RPC version.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Version {
    #[serde(rename = "1.0")]
    V1,
    #[serde(rename = "2.0")]
    V2,
}

/// A JSON-RPC request.
///
/// Normally, you shouldn't need to interact with this directly. It is used to correctly serialize the request being sent.
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct Request {
    pub id: Id,
    pub jsonrpc: Version,
    pub method: String,
    pub params: Vec<serde_json::Value>,
}

impl Request {
    pub fn new_v1(method: &str, params: Vec<serde_json::Value>) -> Self {
        Self {
            id: Id::Number(0),
            jsonrpc: Version::V1,
            method: method.to_owned(),
            params,
        }
    }

    pub fn new_v2(method: &str, params: Vec<serde_json::Value>) -> Self {
        Self {
            id: Id::Number(0),
            jsonrpc: Version::V2,
            method: method.to_owned(),
            params,
        }
    }
}

/// A JSON-RPC response.
///
/// Normally, you shouldn't need to interact with this directly. It is used to correctly deserialize the response from the server.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Response<P> {
    pub id: Id,
    pub jsonrpc: Option<Version>,
    #[serde(flatten)]
    pub payload: ResponsePayload<P>,
}

impl<P> Response<P> {
    pub fn new_v1_result(id: Id, result: P) -> Self {
        Self {
            id,
            jsonrpc: Some(Version::V1),
            payload: ResponsePayload {
                result: Some(result),
                error: None,
            },
        }
    }

    pub fn new_v2_result(id: Id, result: P) -> Self {
        Self {
            id,
            jsonrpc: Some(Version::V2),
            payload: ResponsePayload {
                result: Some(result),
                error: None,
            },
        }
    }

    pub fn new_v1_error(id: Id, error: JsonRpcError) -> Self {
        Self {
            id,
            jsonrpc: Some(Version::V1),
            payload: ResponsePayload {
                result: None,
                error: Some(error),
            },
        }
    }
    pub fn new_v2_error(id: Id, error: JsonRpcError) -> Self {
        Self {
            id,
            jsonrpc: Some(Version::V2),
            payload: ResponsePayload {
                result: None,
                error: Some(error),
            },
        }
    }
}

#[doc(hidden)]
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub struct ResponsePayload<P> {
    result: Option<P>,
    error: Option<JsonRpcError>,
}

impl<P> From<ResponsePayload<P>> for Result<P, JsonRpcError> {
    fn from(value: ResponsePayload<P>) -> Self {
        match value {
            ResponsePayload {
                error: Some(error),
                result: None,
            } => Err(error),
            ResponsePayload {
                error: None,
                result: Some(ok),
            } => Ok(ok),
            ResponsePayload {
                error: Some(_),
                result: Some(_),
            } => Err(JsonRpcError {
                code: -32603,
                message: "invalid JSON-RPC response, got both `result` and `error`".to_string(),
            }),
            ResponsePayload {
                error: None,
                result: None,
            } => Err(JsonRpcError {
                code: -32603,
                message: "invalid JSON-RPC response, got neither `result` nor `error`".to_string(),
            }),
        }
    }
}

/// A JSON-RPC error.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
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
            Error::Client(inner) => fmt::Display::fmt(inner, f),
            Error::JsonRpc(inner) => fmt::Display::fmt(inner, f),
            Error::Serde(inner) => fmt::Display::fmt(inner, f),
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

impl<C> StdError for Error<C>
where
    C: StdError + 'static,
{
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Client(inner) => Some(inner),
            Error::JsonRpc(inner) => Some(inner),
            Error::Serde(inner) => Some(inner),
        }
    }
}

/// A trait abstracting over how a request is actually sent to a server.
///
/// This trait needs to be implemented on the "inner" client.
///
/// # Example
///
/// ```rust,no_run
/// # use serde::de::DeserializeOwned;
/// # use jsonrpc_client::{Response, SendRequest, Url};
/// struct MyHttpClient;
///
/// #[async_trait::async_trait]
/// impl SendRequest for MyHttpClient {
/// #     type Error = reqwest::Error;
///
///     async fn send_request<P>(&self, endpoint: Url, body: String) -> Result<Response<P>, Self::Error>
///     where
///         P: DeserializeOwned,
///     {
///         // send the given body to the given endpoint and deserialize the response as `Response<P>`
/// #        unimplemented!()
///     }
/// }
///
/// #[jsonrpc_client::api]
/// pub trait Math {
///     async fn subtract(&self, subtrahend: i64, minuend: i64) -> i64;
/// }
///
/// #[jsonrpc_client::implement(Math)]
/// struct Client {
///     inner: MyHttpClient,
///     base_url: Url,
/// }
/// ```
#[async_trait::async_trait]
pub trait SendRequest: 'static {
    type Error: StdError;

    async fn send_request<P>(
        &self,
        endpoint: Url,
        body: String,
    ) -> Result<Response<P>, Self::Error>
    where
        P: DeserializeOwned;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn deserialize_v1_error_response_error_first_result_second() {
        let json = r#"{"error":{"code":-6,"message":"Insufficient funds"},"result":null,"id":0}"#;

        let response = serde_json::from_str::<Response<String>>(json).unwrap();

        assert_eq!(response.id, Id::Number(0));
        assert_eq!(response.jsonrpc, None);
        assert_eq!(
            Result::from(response.payload),
            Err(JsonRpcError {
                code: -6,
                message: "Insufficient funds".to_owned()
            })
        )
    }

    #[test]
    fn deserialize_v1_error_response_result_first_error_second() {
        let json = r#"{"result":null,"error":{"code":-6,"message":"Insufficient funds"},"id":0}"#;

        let response = serde_json::from_str::<Response<String>>(json).unwrap();

        assert_eq!(response.id, Id::Number(0));
        assert_eq!(response.jsonrpc, None);
        assert_eq!(
            Result::from(response.payload),
            Err(JsonRpcError {
                code: -6,
                message: "Insufficient funds".to_owned()
            })
        )
    }

    #[test]
    fn deserialize_v1_error_response_result_first_error_second_unit_type() {
        let json = r#"{"result":null,"error":{"code":-6,"message":"Insufficient funds"},"id":0}"#;

        let response = serde_json::from_str::<Response<()>>(json).unwrap();

        assert_eq!(response.id, Id::Number(0));
        assert_eq!(response.jsonrpc, None);
        assert_eq!(
            Result::from(response.payload),
            Err(JsonRpcError {
                code: -6,
                message: "Insufficient funds".to_owned()
            })
        )
    }

    #[test]
    fn deserialize_v1_error_no_result() {
        let json = r#"{"error":{"code":-6,"message":"Insufficient funds"},"id":0}"#;

        let response = serde_json::from_str::<Response<String>>(json).unwrap();

        assert_eq!(response.id, Id::Number(0));
        assert_eq!(response.jsonrpc, None);
        assert_eq!(
            Result::from(response.payload),
            Err(JsonRpcError {
                code: -6,
                message: "Insufficient funds".to_owned()
            })
        )
    }

    #[test]
    fn deserialize_v2_error_response() {
        let json = r#"{"jsonrpc": "2.0", "error": {"code": -32601, "message": "Method not found"}, "id": "1"}"#;

        let response = serde_json::from_str::<Response<()>>(json).unwrap();

        assert_eq!(response.id, Id::String("1".to_owned()));
        assert_eq!(response.jsonrpc, Some(Version::V2));
        assert_eq!(
            Result::from(response.payload),
            Err(JsonRpcError {
                code: -32601,
                message: "Method not found".to_owned()
            })
        )
    }

    #[test]
    fn deserialize_success_response() {
        let json = r#"{"jsonrpc": "2.0", "result": 19, "id": 1}"#;

        let response = serde_json::from_str::<Response<i32>>(json).unwrap();

        assert_eq!(response.id, Id::Number(1));
        assert_eq!(response.jsonrpc, Some(Version::V2));
        assert_eq!(Result::from(response.payload), Ok(19))
    }

    #[test]
    fn serialize_request() {
        let request = Request::new_v2("subtract", vec![json!(42), json!(23)]);

        let json = serde_json::to_string(&request).unwrap();

        assert_eq!(
            json,
            r#"{"id":0,"jsonrpc":"2.0","method":"subtract","params":[42,23]}"#
        );
    }
}
