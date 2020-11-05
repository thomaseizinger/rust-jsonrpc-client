# No bullshit JSON-RPC client for Rust

## Features

- No boilerplate: Driven by proc-macros
- No client lock-in: Complete freedom over underlying HTTP client
- Flexible: Sync or async, it is up to you
- Lightweight: Only depends on syn and serde

## How does it work?

We take a trait as input to a proc-macro and output another one that has default implementations for all the functions.
This allows us to take away all the boilerplate of making JSON-RPC calls and you get to define a nice interface at the same time!

## How do I use it?

1. Depend on `jsonrpc_client`:

    ```toml
   [dependencies]
   jsonrpc_client = { version = "*", features = ["reqwest"] } 
   ```

2. Define a trait that describes the JSON-RPC API you want to talk to and annotate it with `#[jsonrpc_client::api]`:
    ```rust
    #[jsonrpc_client::api]
    pub trait Math {
        fn subtract(&self, subtrahend: i64, minuend: i64) -> i64;
    }
    ```

3. Define your client:
    
    ```rust
    #[jsonrpc_client::implement(Math)]
    struct Client {
        inner: reqwest::Client,
        base_url: reqwest::Url
    }
    ```
   
4. Start using your client!
