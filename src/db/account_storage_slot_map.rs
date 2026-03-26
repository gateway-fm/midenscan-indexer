use super::models;
use anyhow::Result;
use sqlx::{types::BigDecimal, QueryBuilder};

pub async fn insert_or_merge_account_storage_slot_maps(
    db_tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    account_storage_slot_maps: Vec<models::DatabaseAccountStorageSlotMap>,
) -> Result<(), sqlx::Error> {
    if account_storage_slot_maps.is_empty() {
        return Ok(());
    }

    let mut query_builder: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new(
        "INSERT INTO account_storage_slot_map (
            account_storage_slot_map_id,
            account_bech,
            slot_index,
            key,
            value,
            last_updated_at_block_number,
            last_updated_at_account_update_id
        ) ",
    );

    query_builder.push_values(account_storage_slot_maps, |mut b, account_storage_slot| {
        b.push_bind(account_storage_slot.account_storage_slot_map_id)
            .push_bind(account_storage_slot.account_bech)
            .push_bind(account_storage_slot.slot_index)
            .push_bind(account_storage_slot.key)
            .push_bind(account_storage_slot.value)
            .push_bind(BigDecimal::from(
                account_storage_slot.last_updated_at_block_number,
            ))
            .push_bind(account_storage_slot.last_updated_at_account_update_id);
    });

    query_builder.push(
        " ON CONFLICT (account_storage_slot_map_id) DO UPDATE SET 
            value = EXCLUDED.value,
            last_updated_at_block_number = EXCLUDED.last_updated_at_block_number,
            last_updated_at_account_update_id = EXCLUDED.last_updated_at_account_update_id",
    );

    let query = query_builder.build();
    query.execute(&mut **db_tx).await?;

    Ok(())
}
