use anyhow::Result;
use miden_node_proto::generated::{
    blockchain::BlockNumber,
    rpc::{api_client::ApiClient, RpcStatus},
};
use miden_protocol::{block::ProvenBlock, utils::Deserializable};
use std::time::Duration;
use tokio::time::timeout;

#[derive(Debug, Clone)]
pub struct Rpc {
    pub rpc_url: String,
}

impl Rpc {
    const TIMEOUT: Duration = Duration::from_secs(20);

    pub fn new(url: &str) -> Self {
        Self {
            rpc_url: url.to_string(),
        }
    }

    pub async fn get_block_by_number_with_timeout(&self, block_num: u32) -> Result<ProvenBlock> {
        match timeout(Self::TIMEOUT, self.get_block_by_number(block_num)).await {
            // ───── finished in time ───────────────────────────────────────────
            Ok(Ok(block)) => Ok(block),

            // ───── call returned an error before 20 s elapsed ─────────────────
            Ok(Err(err)) => Err(err),

            // ───── wall-clock hit 20 s ────────────────────────────────────────
            Err(_) => Err(anyhow::anyhow!("Block data call timeout")),
        }
    }

    pub async fn get_status_with_timeout(&self) -> Result<RpcStatus> {
        match timeout(Self::TIMEOUT, self.get_status()).await {
            // ───── finished in time ───────────────────────────────────────────
            Ok(Ok(status)) => Ok(status),

            // ───── call returned an error before 20 s elapsed ─────────────────
            Ok(Err(err)) => Err(err),

            // ───── wall-clock hit 20 s ────────────────────────────────────────
            Err(_) => Err(anyhow::anyhow!("Status call timeout")),
        }
    }

    pub async fn get_block_by_number(&self, block_num: u32) -> Result<ProvenBlock> {
        let mut rpc_api = ApiClient::connect(self.rpc_url.clone()).await.unwrap();

        let request = BlockNumber { block_num };
        let api_response = rpc_api.get_block_by_number(request).await?.into_inner();

        if let Some(block_data) = api_response.block {
            // Deserialize the block data using miden-objects Deserializer
            match ProvenBlock::read_from_bytes(&block_data) {
                Ok(block) => Ok(block),
                Err(err) => Err(anyhow::anyhow!(format!(
                    "Could not deserialize block data: {}",
                    err
                ))),
            }
        } else {
            Err(anyhow::anyhow!("Block data is missing from the response"))
        }
    }

    pub async fn get_status(&self) -> Result<RpcStatus> {
        let mut rpc_api = ApiClient::connect(self.rpc_url.clone()).await.unwrap();

        let api_response = rpc_api.status(()).await;
        match api_response {
            Ok(status) => Ok(status.into_inner()),
            Err(err) => Err(anyhow::anyhow!(format!(
                "Could not get RPC status: {}",
                err
            ))),
        }
    }
}
