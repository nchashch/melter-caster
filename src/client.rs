use anyhow::Result;
use jsonrpsee::core::Error;
use jsonrpsee::http_client::{HeaderMap, HttpClient, HttpClientBuilder};
use jsonrpsee::proc_macros::rpc;

pub fn new_client() -> Result<HttpClient> {
    let mut headers = HeaderMap::new();
    let auth = format!("{}:{}", "user", "password");
    let header_value = format!("Basic {}", base64::encode(auth)).parse()?;
    headers.insert("authorization", header_value);
    let zcash = HttpClientBuilder::default()
        .set_headers(headers)
        .build("http://127.0.0.1:19000")?;
    Ok(zcash)
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Transaction {
    confirmations: usize,
}

#[rpc(client)]
pub trait Zcash {
    #[method(name = "getnewaddress")]
    async fn getnewaddress(&self, address_type: Option<&str>) -> Result<String, Error>;
    #[method(name = "gettransaction")]
    async fn gettransaction(
        &self,
        txid: bitcoin::Txid,
        include_watchonly: Option<bool>,
    ) -> Result<Transaction, Error>;
    #[method(name = "z_getnewaddress")]
    async fn z_getnewaddress(&self) -> Result<String, Error>;
}
