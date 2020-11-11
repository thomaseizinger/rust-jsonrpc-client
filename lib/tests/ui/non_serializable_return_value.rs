struct Number;

#[jsonrpc_client::api]
pub trait Math {
    async fn subtract(&self, subtrahend: i64, minuend: i64) -> Number;
}

fn main() {}
