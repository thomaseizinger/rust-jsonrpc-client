#[jsonrpc_client::api]
pub trait Math {
    fn subtract(&self, subtrahend: i64, minuend: i64) -> i64;
}

fn main() {}
