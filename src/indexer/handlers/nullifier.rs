use crate::db;
use crate::utils;
use anyhow::Result;
use miden_protocol::utils::Serializable;

pub async fn nullifier_handler(
    db_tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    block: miden_protocol::block::ProvenBlock,
) -> Result<()> {
    let mut database_nullifiers: Vec<db::models::DatabaseNullifier> = Vec::new();
    for (block_nullifier_index_usize, nullifier) in
        block.body().created_nullifiers().iter().enumerate()
    {
        // DEVNOTE: don't expect for there to be that many nullifiers in a single block
        let block_nullifier_index = u32::try_from(block_nullifier_index_usize)?;
        database_nullifiers.push(db::models::DatabaseNullifier {
            nullifier: nullifier.to_bytes().to_vec(),
            nullifier_index: block_nullifier_index,
            block_number: block.header().block_num().as_u32(),
            timestamp: block.header().timestamp(),
            // {block_number}_{nullifier_index}_{}
            internal_time: utils::internal_time::get_internal_time(
                block.header().block_num().as_u32(),
                block_nullifier_index,
                0,
            ),
        });
    }
    db::nullifier::insert_nullifiers(db_tx, database_nullifiers).await?;

    Ok(())
}
