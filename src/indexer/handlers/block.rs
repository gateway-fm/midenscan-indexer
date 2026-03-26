use crate::db;
use anyhow::Result;
use miden_protocol;

pub async fn block_handler(
    db_tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    block: miden_protocol::block::ProvenBlock,
) -> Result<()> {
    db::block::insert_block(
        db_tx,
        db::models::DatabaseBlock {
            block_commitment: block.header().commitment().as_bytes().to_vec(),
            block_number: block.header().block_num().as_u32(),
            version: block.header().version(),
            timestamp: block.header().timestamp(),

            chain_commitment: block.header().chain_commitment().as_bytes().to_vec(),
            account_root: block.header().account_root().as_bytes().to_vec(),
            nullifier_root: block.header().nullifier_root().as_bytes().to_vec(),
            note_root: block.header().note_root().as_bytes().to_vec(),
            tx_commitment: block.header().tx_commitment().as_bytes().to_vec(),
            proof_commitment: block.header().tx_kernel_commitment().as_bytes().to_vec(),
            sub_commitment: block.header().sub_commitment().as_bytes().to_vec(),

            number_of_transactions: u32::try_from(block.body().transactions().as_slice().len())?,
        },
    )
    .await?;

    Ok(())
}
