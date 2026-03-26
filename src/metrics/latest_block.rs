use anyhow::Result;
use log::error;

use std::time::Duration;
use tokio::time::sleep;

use crate::config;
use crate::metrics;
use crate::rpc::Rpc;

const POLL_INTERVAL: Duration = Duration::from_secs(5);

/// Returns the latest available block number on the connected node using header probes.
async fn get_latest_block_number(rpc: &Rpc) -> Result<u32> {
    let status = rpc.get_status().await;
    match status {
        Ok(s) => match s.store {
            Some(store) => Ok(store.chain_tip),
            None => Err(anyhow::anyhow!("RPC store status error: missing store")),
        },
        Err(e) => Err(anyhow::anyhow!("Could not get latest block number: {}", e)),
    }
}

/// Background task: update `latest_block` gauge every second.
pub async fn run_latest_block_poller() {
    let rpc = Rpc::new(&config::CONFIG.rpc_url);
    loop {
        match get_latest_block_number(&rpc).await {
            Ok(n) => {
                metrics::set_latest_block(n);
            }
            Err(e) => {
                // Log and keep previous metric value
                error!("latest_block poller error: {}", e);
            }
        }
        sleep(POLL_INTERVAL).await;
    }
}
