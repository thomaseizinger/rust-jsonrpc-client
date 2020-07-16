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

1. Define a trait that describes the JSON-RPC API you want to talk to and annotate it with `#[jsonrpc_client::api]`:
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
    impl jsonrpc_client::SendRequest for Client {
        type Error = reqwest::Error;
    
        fn send_request(&self, request: Request) -> Result<Response, Self::Error> {
            self.inner.post("http://example.org").json(&request).send()?.json()
        }
    }
   ```

4. Implement the API trait on your client.
The macro provided default implementations for all functions based on the `SendRequest` functionality.

    ```rust
   impl Math for Client {} 
   ```

5. Start using your client!
