#[jsonrpc_client::api(foo = "bar")]
pub trait Math {
    async fn subtract(&self, subtrahend: i64, minuend: i64) -> i64;
}

fn main() {}
