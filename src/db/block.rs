use super::models;
use anyhow::Result;
use sqlx::types::BigDecimal;

pub async fn insert_block(
    db_tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    block: models::DatabaseBlock,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO block (
        block_commitment,
        block_number,
        version,
        timestamp,
        chain_commitment,
        account_root,
        nullifier_root,
        note_root,
        tx_commitment,
        proof_commitment,
        sub_commitment,
        number_of_transactions
      ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
    )
    .bind(&block.block_commitment)
    .bind(BigDecimal::from(block.block_number))
    .bind(BigDecimal::from(block.version))
    .bind(BigDecimal::from(block.timestamp))
    .bind(&block.chain_commitment)
    .bind(&block.account_root)
    .bind(&block.nullifier_root)
    .bind(&block.note_root)
    .bind(&block.tx_commitment)
    .bind(&block.proof_commitment)
    .bind(&block.sub_commitment)
    .bind(BigDecimal::from(block.number_of_transactions))
    .execute(&mut **db_tx)
    .await?;

    Ok(())
}
