use anyhow::Result;

#[jsonrpc_client::api]
pub trait Math {
    fn subtract(&self, subtrahend: i64, minuend: i64) -> i64;
}

#[jsonrpc_client::implement(Math)]
struct Client {
    inner: reqwest::blocking::Client,
    base_url: reqwest::Url,
}

impl Client {
    fn new(base_url: String) -> Result<Self> {
        Ok(Self {
            inner: reqwest::blocking::Client::new(),
            base_url: base_url.parse()?,
        })
    }
}

fn main() -> Result<()> {
    let client = Client::new("http://example-jsonrpc.org/".to_owned())?;

    let _ = client.subtract(10, 5)?;

    Ok(())
}
