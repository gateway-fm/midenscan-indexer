use super::handlers;
use super::seed;
use crate::config::CONFIG;
use crate::db;
use crate::metrics;
use crate::rpc;
use log::{error, info};

use anyhow::Result;

pub async fn start() {
    info!("Starting Indexer...");

    let db = db::Database::new(&CONFIG.postgres_url).await;
    if let Err(e) = seed::seed_standard_components(&db).await {
        error!("Failed to seed standard account components: {}", e);
    }
    if let Err(e) = seed::seed_standard_notes(&db).await {
        error!("Failed to seed standard note scripts: {}", e);
    }

    loop {
        match run_handlers().await {
            Ok(_) => {}
            Err(e) => {
                if let Some(rpc_error) = e.downcast_ref::<rpc::error::RpcError>() {
                    match rpc_error {
                        rpc::error::RpcError::NotFound(block_number) => {
                            info!("Waiting for next block: {}", block_number);
                        }
                        _ => error!("RPC Error: {}", e),
                    }
                } else {
                    error!("Error: {}", e);
                }
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
        }
    }
}

async fn run_handlers() -> Result<()> {
    let rpc: rpc::Rpc = rpc::Rpc::new(&CONFIG.rpc_url);
    let db = db::Database::new(&CONFIG.postgres_url).await;

    db.execute_transaction(|db_tx| {
        Box::pin(async move {
            let now = std::time::Instant::now();

            let db_state_ref = db::state_ref::get_ref(db_tx).await?;
            let block = rpc
                .get_block_by_number_with_timeout((db_state_ref.block_number + 1) as u32)
                .await?;

            handlers::block::block_handler(db_tx, block.clone()).await?;
            handlers::account::account_handler(db_tx, block.clone()).await?;
            handlers::transaction::transaction_handler(db_tx, block.clone()).await?;
            handlers::note::note_handler(db_tx, block.clone()).await?;
            handlers::nullifier::nullifier_handler(db_tx, block.clone()).await?;

            db::state_ref::update_ref(
                db_tx,
                db::models::DatabaseRef {
                    block_commitment: block.header().commitment().as_bytes().to_vec(),
                    // TODO should use u32 instead of i32
                    block_number: block.header().block_num().as_u32() as i32,
                },
            )
            .await?;

            // Update metrics
            let took = now.elapsed();
            let indexed_block_number = block.header().block_num().as_u32();

            metrics::on_block_indexed(indexed_block_number, took);

            info!("Indexed Block: {}, took {:.2?}", indexed_block_number, took);

            Ok(())
        })
    })
    .await?;

    Ok(())
}
