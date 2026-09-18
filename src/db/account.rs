use super::models;

use anyhow::Result;
use num_bigint::BigInt;
use sqlx::{types::BigDecimal, QueryBuilder, Row};

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
                code,
                code_size,
                code_procedure_roots,
                code_commitment,
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
            .push_bind(account.code)
            .push_bind(BigDecimal::from(account.code_size))
            .push_bind(account.code_procedure_roots)
            .push_bind(account.code_commitment)
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

/// A public account indexed before code commitments were recorded.
#[derive(Debug, Clone)]
pub struct AccountMissingCodeCommitment {
    pub account_bech: String,
    pub account_id: Vec<u8>,
    pub deployed_at_block_number: u32,
}

pub async fn select_accounts_missing_code_commitment(
    pool: &sqlx::Pool<sqlx::Postgres>,
) -> Result<Vec<AccountMissingCodeCommitment>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT account_bech, account_id, deployed_at_block_number::BIGINT AS deployed_at_block_number
         FROM account
         WHERE account_type = 'Public' AND code IS NOT NULL AND code_commitment IS NULL
         ORDER BY deployed_at_block_number, account_bech",
    )
    .fetch_all(pool)
    .await?;

    rows.iter()
        .map(|row| {
            let block_number: i64 = row.try_get("deployed_at_block_number")?;
            Ok(AccountMissingCodeCommitment {
                account_bech: row.try_get("account_bech")?,
                account_id: row.try_get("account_id")?,
                deployed_at_block_number: u32::try_from(block_number)
                    .map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
            })
        })
        .collect()
}

/// Records the code commitment of an account that does not have one yet.
/// Returns whether a row was updated.
pub async fn update_account_code_commitment(
    pool: &sqlx::Pool<sqlx::Postgres>,
    account_bech: &str,
    code_commitment: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE account SET code_commitment = $1 WHERE account_bech = $2 AND code_commitment IS NULL",
    )
    .bind(code_commitment)
    .bind(account_bech)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}
