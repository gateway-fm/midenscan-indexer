use std::time::Duration;

use miden_protocol::address::NetworkId;
use once_cell::sync::Lazy;
use prometheus::{HistogramVec, IntGaugeVec};

use crate::config::CONFIG;

const NETWORK_LABEL: &str = "network";
static NETWORK: Lazy<String> = Lazy::new(|| match &CONFIG.miden_network {
    NetworkId::Mainnet => "mainnet".into(),
    NetworkId::Testnet => "testnet".into(),
    NetworkId::Devnet => "devnet".into(),
    NetworkId::Custom(n) => format!("custom_{}", n),
});
static LABELS: Lazy<Vec<&str>> = Lazy::new(|| vec![&NETWORK]);

static LAST_INDEXED_BLOCK: Lazy<IntGaugeVec> = Lazy::new(|| {
    prometheus::register_int_gauge_vec!(
        "midenscan_indexer_last_indexed_block",
        "Last successfully indexed block number",
        &[NETWORK_LABEL]
    )
    .expect("register last_indexed_block")
});

static LATEST_BLOCK: Lazy<IntGaugeVec> = Lazy::new(|| {
    prometheus::register_int_gauge_vec!(
        "midenscan_indexer_latest_block",
        "Latest known chain block number",
        &[NETWORK_LABEL]
    )
    .expect("register latest_block")
});

static BLOCK_PROCESS_TIME: Lazy<HistogramVec> = Lazy::new(|| {
    prometheus::register_histogram_vec!(
        "midenscan_indexer_block_process_time_ms",
        "Time taken to process a block (milliseconds)",
        &[NETWORK_LABEL],
        vec![10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0, 10000.0]
    )
    .expect("register block_process_time_ms")
});

pub fn on_block_indexed(block_number: u32, duration: Duration) {
    LAST_INDEXED_BLOCK
        .with_label_values(&LABELS)
        .set(block_number as i64);
    BLOCK_PROCESS_TIME
        .with_label_values(&LABELS)
        .observe(duration.as_millis() as f64);
}

pub fn set_latest_block(block_number: u32) {
    LATEST_BLOCK
        .with_label_values(&LABELS)
        .set(block_number as i64);
}
