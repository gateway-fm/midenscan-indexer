use crate::db;
use crate::utils;
use anyhow::Result;
use miden_protocol::crypto::utils::Serializable;

pub async fn transaction_handler(
    db_tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    block: miden_protocol::block::ProvenBlock,
) -> Result<()> {
    let mut database_transactions: Vec<db::models::DatabaseTransaction> = Vec::new();
    let mut database_transaction_input_notes: Vec<db::models::DatabaseTransactionInputNote> =
        Vec::new();
    let mut database_transaction_output_notes: Vec<db::models::DatabaseTransactionOutputNote> =
        Vec::new();

    for (block_transaction_index, transaction_header) in
        block.body().transactions().as_slice().iter().enumerate()
    {
        let transaction_index_u32 = u32::try_from(block_transaction_index)?;
        let account_update_id = format!(
            "{}_{}",
            transaction_header.account_id().to_hex(),
            block.header().commitment().to_hex()
        );

        database_transactions.push(db::models::DatabaseTransaction {
            transaction_id: transaction_header.id().as_bytes().to_vec(),

            account_update_id,
            account_bech: utils::format::account_id_to_bech32(&transaction_header.account_id()),
            block_number: block.header().block_num().as_u32(),
            timestamp: block.header().timestamp(),

            transaction_index: transaction_index_u32,
            initial_state_commitment: transaction_header
                .initial_state_commitment()
                .as_bytes()
                .to_vec(),
            final_state_commitment: transaction_header
                .final_state_commitment()
                .as_bytes()
                .to_vec(),

            // {block_number}_{transaction_index}_0
            internal_time: utils::internal_time::get_internal_time(
                block.header().block_num().as_u32(),
                transaction_index_u32,
                0,
            ),
        });

        for input_note in transaction_header.input_notes() {
            let transaction_input_note_id = format!(
                "{}_{}",
                transaction_header.id().to_hex(),
                input_note.nullifier().to_hex()
            );
            database_transaction_input_notes.push(db::models::DatabaseTransactionInputNote {
                transaction_input_note_id,
                transaction_id: transaction_header.id().as_bytes().to_vec(),
                nullifier: input_note.to_bytes().to_vec(),
            })
        }
        for output_note in transaction_header.output_notes() {
            let transaction_output_note_id = format!(
                "{}_{}",
                transaction_header.id().to_hex(),
                output_note.id().to_hex()
            );
            database_transaction_output_notes.push(db::models::DatabaseTransactionOutputNote {
                transaction_output_note_id,
                transaction_id: transaction_header.id().as_bytes().to_vec(),
                note_id: output_note.to_bytes().to_vec(),
            })
        }
    }

    db::transaction::insert_transactions(db_tx, database_transactions).await?;
    db::transaction::insert_transaction_input_notes(db_tx, database_transaction_input_notes)
        .await?;
    db::transaction::insert_transaction_output_notes(db_tx, database_transaction_output_notes)
        .await?;

    Ok(())
}
