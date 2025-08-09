use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Network {
    Mainnet,
    Testnet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRequest {
    pub tx_hex: String,
    pub network: Network,
}

impl Network {
    pub fn mempool_api_url(&self) -> &'static str {
        match self {
            Network::Mainnet => "https://mempool.space/api",
            Network::Testnet => "https://mempool.space/testnet/api",
        }
    }
}