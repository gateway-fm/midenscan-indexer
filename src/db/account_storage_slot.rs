use super::models;
use anyhow::Result;
use sqlx::{types::BigDecimal, QueryBuilder};

pub async fn insert_or_merge_account_storage_slots(
    db_tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    account_storage_slots: Vec<models::DatabaseAccountStorageSlot>,
) -> Result<(), sqlx::Error> {
    if account_storage_slots.is_empty() {
        return Ok(());
    }

    let mut query_builder: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new(
        "INSERT INTO account_storage_slot (
            account_storage_slot_id,
            account_bech,
            slot_index,
            value,
            account_storage_slot_type,
            last_updated_at_block_number,
            last_updated_at_account_update_id
        ) ",
    );

    query_builder.push_values(account_storage_slots, |mut b, account_storage_slot| {
        b.push_bind(account_storage_slot.account_storage_slot_id)
            .push_bind(account_storage_slot.account_bech)
            .push_bind(account_storage_slot.slot_index)
            .push_bind(account_storage_slot.value)
            .push_bind(account_storage_slot.account_storage_slot_type)
            .push_bind(BigDecimal::from(
                account_storage_slot.last_updated_at_block_number,
            ))
            .push_bind(account_storage_slot.last_updated_at_account_update_id);
    });

    query_builder.push(
        " ON CONFLICT (account_storage_slot_id) DO UPDATE SET 
            value = EXCLUDED.value,
            account_storage_slot_type = EXCLUDED.account_storage_slot_type,
            last_updated_at_block_number = EXCLUDED.last_updated_at_block_number,
            last_updated_at_account_update_id = EXCLUDED.last_updated_at_account_update_id",
    );

    let query = query_builder.build();
    query.execute(&mut **db_tx).await?;

    Ok(())
}
