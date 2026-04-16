use super::models;

use anyhow::Result;
use num_bigint::BigInt;
use sqlx::{types::BigDecimal, QueryBuilder};

pub async fn insert_or_ignore_accounts(
    db_tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    accounts: Vec<models::DatabaseAccount>,
) -> Result<(), sqlx::Error> {
    if accounts.is_empty() {
        return Ok(());
    }

    let mut query_builder: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new(
        "INSERT INTO account (
                account_bech,
                account_id,
                account_id_prefix,
                account_type,
                storage_mode,
                code,
                code_size,
                code_procedure_roots,
                deployed_at_block_number,
                deployed_at_timestamp,
                deployed_at_updated_account_index,
                deployed_at_internal_time
            ) ",
    );
    query_builder.push_values(accounts, |mut b, account| {
        b.push_bind(account.account_bech)
            .push_bind(account.account_id)
            .push_bind(account.account_id_prefix)
            .push_bind(account.account_type)
            .push_bind(account.storage_mode)
            .push_bind(account.code)
            .push_bind(BigDecimal::from(account.code_size))
            .push_bind(account.code_procedure_roots)
            .push_bind(BigDecimal::from(account.deployed_at_block_number))
            .push_bind(BigDecimal::from(account.deployed_at_timestamp))
            .push_bind(BigDecimal::from(account.deployed_at_updated_account_index))
            .push_bind(BigDecimal::from(BigInt::from(
                account.deployed_at_internal_time,
            )));
    });
    query_builder.push(" ON CONFLICT (account_bech) DO NOTHING");

    let query = query_builder.build();
    query.execute(&mut **db_tx).await?;

    Ok(())
}
