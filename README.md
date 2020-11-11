# No bullshit JSON-RPC client for Rust

## Features

- No boilerplate: Driven by proc-macros
- No client lock-in: Complete freedom over underlying HTTP client
- Lightweight: Only depends on syn and serde
- Async-ready

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
       async fn subtract(&self, subtrahend: i64, minuend: i64) -> i64;
   }
   ```

3. Define your client:

   ```rust
   #[jsonrpc_client::implement(Math)]
   struct Client {
       inner: reqwest::Client,
       base_url: reqwest::Url,
   }
   ```

4. Start using your client!

## Backends

Currently, the client supports several backends, all of them can be activated via a separate feature-flag:

- reqwest
- surf
- isahc

Support for more backends is welcomed.

Unfortunately, not all can be supported.
In particular:

- hreq: Cannot be supported because their `Agent` takes `&mut self` for sending a request which doesn't work with the `SendRequest` trait.
- awc: Cannot be supported because their `Client` does not implement `Send`.
