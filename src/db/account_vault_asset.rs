use super::models;

use anyhow::Result;
use sqlx::{types::BigDecimal, QueryBuilder};

pub async fn insert_or_add_account_vault_assets(
    db_tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    account_vault_assets_added: Vec<models::DatabaseAccountVaultAsset>,
) -> Result<(), sqlx::Error> {
    if account_vault_assets_added.is_empty() {
        return Ok(());
    }

    let mut query_builder: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new(
        "INSERT INTO account_vault_asset (
              account_vault_asset_id,
              account_bech,
              faucet_id_prefix,
              amount
          ) 
          ",
    );
    query_builder.push_values(account_vault_assets_added, |mut b, account_vault_asset| {
        b.push_bind(account_vault_asset.account_vault_asset_id)
            .push_bind(account_vault_asset.account_bech)
            .push_bind(account_vault_asset.faucet_id_prefix)
            .push_bind(BigDecimal::from(account_vault_asset.amount));
    });

    query_builder.push(
        " ON CONFLICT (account_vault_asset_id) DO UPDATE SET 
          amount = account_vault_asset.amount + EXCLUDED.amount",
    );

    let query = query_builder.build();
    query.execute(&mut **db_tx).await?;

    Ok(())
}
