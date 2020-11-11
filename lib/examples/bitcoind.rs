use anyhow::{Context, Result};
use bitcoincore_rpc_json::GetBlockchainInfoResult;
use testcontainers::images::coblox_bitcoincore::BitcoinCore;
use testcontainers::*;

#[jsonrpc_client::api(version = "1.0")]
pub trait BitcoindRpc {
    async fn getblockchaininfo(&self) -> GetBlockchainInfoResult;
}

#[jsonrpc_client::implement(BitcoindRpc)]
struct Client {
    inner: reqwest::Client,
    base_url: jsonrpc_client::Url,
}

impl Client {
    fn new(base_url: String) -> Result<Self> {
        Ok(Self {
            inner: reqwest::Client::new(),
            base_url: base_url.parse()?,
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = clients::Cli::default();
    let container = cli.run(BitcoinCore::default());
    let auth = container.image().auth();

    let client = Client::new(format!(
        "http://{}:{}@localhost:{}",
        auth.username(),
        auth.password(),
        container
            .get_host_port(18443)
            .context("port 18443 was not exposed")?
    ))?;

    let blockchain_info = client.getblockchaininfo().await?;

    println!("{:?}", blockchain_info);

    Ok(())
}
