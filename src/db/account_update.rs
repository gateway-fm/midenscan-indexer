use super::models;
use crate::utils;

use anyhow::Result;
use num_bigint::BigInt;
use sqlx::{types::BigDecimal, QueryBuilder};

pub async fn insert_account_updates(
    db_tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    account_updates: Vec<models::DatabaseAccountUpdate>,
) -> Result<(), sqlx::Error> {
    if account_updates.is_empty() {
        return Ok(());
    }

    let mut query_builder: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new(
        "INSERT INTO account_update (
                account_update_id,
                account_bech,
                final_state_commitment,
                nonce_delta,
                block_number,
                timestamp,
                updated_account_index,
                internal_time
            ) 
            ",
    );
    query_builder.push_values(account_updates, |mut b, account_update| {
        b.push_bind(account_update.account_update_id)
            .push_bind(account_update.account_bech)
            .push_bind(account_update.final_state_commitment)
            .push_bind(utils::format::convert_option_u64_to_bigdecimal(
                account_update.nonce_delta,
            ))
            .push_bind(BigDecimal::from(account_update.block_number))
            .push_bind(BigDecimal::from(account_update.timestamp))
            .push_bind(BigDecimal::from(account_update.updated_account_index))
            .push_bind(BigDecimal::from(BigInt::from(account_update.internal_time)));
    });

    let query = query_builder.build();
    query.execute(&mut **db_tx).await?;

    Ok(())
}
