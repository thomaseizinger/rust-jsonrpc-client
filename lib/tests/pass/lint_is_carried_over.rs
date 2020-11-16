#![deny(non_snake_case)]

#[jsonrpc_client::api]
pub trait Math {
    #[allow(non_snake_case)]
    async fn subtract(&self, Subtrahend: i64, minuend: i64) -> i64;
}

fn main() {}
