struct Number;

#[jsonrpc_client::api]
pub trait Math {
    async fn subtract(&self, subtrahend: Number, minuend: Number) -> u64;
}

fn main() {}
