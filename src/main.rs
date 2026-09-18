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

    // `midenscan-indexer backfill-code-commitments` runs the one-off backfill instead of
    // the indexing loop; it uses the same configuration.
    match std::env::args().nth(1).as_deref() {
        Some("backfill-code-commitments") => {
            if let Err(e) = indexer::backfill::backfill_code_commitments().await {
                log::error!("Backfill failed: {:#}", e);
                std::process::exit(1);
            }
            return;
        }
        Some(command) => {
            eprintln!(
                "unknown command `{}`; usage: midenscan-indexer [backfill-code-commitments]",
                command
            );
            std::process::exit(2);
        }
        None => {}
    }

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
