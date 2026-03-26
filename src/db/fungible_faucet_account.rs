use super::models;
use crate::utils;

use anyhow::Result;
use sqlx::QueryBuilder;

pub async fn insert_fungible_faucet_accounts(
    db_tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    fungible_faucet_accounts: Vec<models::DatabaseFungibleFaucetAccount>,
) -> Result<(), sqlx::Error> {
    if fungible_faucet_accounts.is_empty() {
        return Ok(());
    }

    let mut query_builder: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new(
        "INSERT INTO fungible_faucet_account (
            account_bech,
            faucet_id_prefix,
            symbol,
            decimals,
            max_supply
        ) ",
    );
    query_builder.push_values(fungible_faucet_accounts, |mut b, nullifier| {
        b.push_bind(nullifier.account_bech)
            .push_bind(nullifier.faucet_id_prefix)
            .push_bind(nullifier.symbol)
            .push_bind(utils::format::convert_option_u8_to_bigdecimal(
                nullifier.decimals,
            ))
            .push_bind(utils::format::convert_option_u64_to_bigdecimal(
                nullifier.max_supply,
            ));
    });

    query_builder.push(
        " ON CONFLICT (account_bech) DO UPDATE SET 
            symbol = EXCLUDED.symbol,
            decimals = EXCLUDED.decimals,
            max_supply = EXCLUDED.max_supply",
    );

    let query = query_builder.build();
    query.execute(&mut **db_tx).await?;

    Ok(())
}
