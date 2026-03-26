use super::models;
use anyhow::Result;
use num_bigint::BigInt;
use sqlx::{types::BigDecimal, QueryBuilder};

pub async fn insert_transactions(
    db_tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    transactions: Vec<models::DatabaseTransaction>,
) -> Result<(), sqlx::Error> {
    if transactions.is_empty() {
        return Ok(());
    }

    let mut query_builder: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new(
        "INSERT INTO transaction (
                transaction_id,
                account_update_id,
                account_bech,
                block_number,
                timestamp,
                transaction_index,
                initial_state_commitment,
                final_state_commitment,
                internal_time
            )",
    );
    query_builder.push_values(transactions, |mut b, transaction| {
        b.push_bind(transaction.transaction_id)
            .push_bind(transaction.account_update_id)
            .push_bind(transaction.account_bech)
            .push_bind(BigDecimal::from(transaction.block_number))
            .push_bind(BigDecimal::from(transaction.timestamp))
            .push_bind(BigDecimal::from(transaction.transaction_index))
            .push_bind(transaction.initial_state_commitment)
            .push_bind(transaction.final_state_commitment)
            .push_bind(BigDecimal::from(BigInt::from(transaction.internal_time)));
    });

    let query = query_builder.build();
    query.execute(&mut **db_tx).await?;

    Ok(())
}

pub async fn insert_transaction_input_notes(
    db_tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    transaction_input_notes: Vec<models::DatabaseTransactionInputNote>,
) -> Result<(), sqlx::Error> {
    if transaction_input_notes.is_empty() {
        return Ok(());
    }

    let mut query_builder: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new(
        "INSERT INTO transaction_input_note (
                transaction_input_note_id,
                transaction_id,
                nullifier
            )",
    );
    query_builder.push_values(transaction_input_notes, |mut b, transaction_input_note| {
        b.push_bind(transaction_input_note.transaction_input_note_id)
            .push_bind(transaction_input_note.transaction_id)
            .push_bind(transaction_input_note.nullifier);
    });

    let query = query_builder.build();
    query.execute(&mut **db_tx).await?;

    Ok(())
}

pub async fn insert_transaction_output_notes(
    db_tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    transaction_output_notes: Vec<models::DatabaseTransactionOutputNote>,
) -> Result<(), sqlx::Error> {
    if transaction_output_notes.is_empty() {
        return Ok(());
    }

    let mut query_builder: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new(
        "INSERT INTO transaction_output_note (
                transaction_output_note_id,
                transaction_id,
                note_id
            )",
    );
    query_builder.push_values(
        transaction_output_notes,
        |mut b, transaction_output_note| {
            b.push_bind(transaction_output_note.transaction_output_note_id)
                .push_bind(transaction_output_note.transaction_id)
                .push_bind(transaction_output_note.note_id);
        },
    );

    let query = query_builder.build();
    query.execute(&mut **db_tx).await?;

    Ok(())
}
