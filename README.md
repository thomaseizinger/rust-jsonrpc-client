# No bullshit JSON-RPC client for Rust

## Features

- Driven by proc-macros
- Complete freedom over underlying HTTP client
- Sync or async, it is up to you
- Lightweight: Only depends on syn and serde

## How does it work?

1. Define a trait that mimics the JSON-RPC API you want to talk to and annotate it with `#[jsonrpc_client::api]`:
    ```rust
    #[jsonrpc_client::api]
    pub trait Math {
        fn subtract(&self, subtrahend: i64, minuend: i64) -> i64;
    }
    ```
      
2. Define your client:
    
    ```rust
    struct Client {
        inner: reqwest::Client
    }
    ```
   
3. Implement the `jsonrpc_client::SendRequest` trait for your client.
For most backends, this should almost be a oneliner.

    ```rust
    
   ```
