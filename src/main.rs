pub mod config;
pub mod db;
pub mod http;
pub mod indexer;
pub mod metrics;
pub mod rpc;
pub mod utils;

#[tokio::main()]
async fn main() {
    // dev convenience, load variables from a `.env` file if it exists.
    let _ = dotenvy::dotenv();

    // Initialize logger for SQLx and other logs.
    // If RUST_LOG is not set, default to "info".
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // start HTTP server in the background
    tokio::spawn(async move {
        http::server::run().await;
    });

    // start latest-block metrics poller in the background
    tokio::spawn(async move {
        metrics::latest_block::run_latest_block_poller().await;
    });

    // start the indexer (blocking loop)
    indexer::start().await;
}
