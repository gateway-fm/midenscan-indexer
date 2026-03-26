use super::models;
use anyhow::Result;
use num_bigint::BigInt;
use sqlx::{types::BigDecimal, QueryBuilder};

pub async fn insert_nullifiers(
    db_tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    nullifiers: Vec<models::DatabaseNullifier>,
) -> Result<(), sqlx::Error> {
    if nullifiers.is_empty() {
        return Ok(());
    }

    let mut query_builder: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new(
        "INSERT INTO nullifier (
            nullifier,
            nullifier_index,
            block_number,
            timestamp,
            internal_time
        ) ",
    );
    query_builder.push_values(nullifiers, |mut b, nullifier| {
        b.push_bind(nullifier.nullifier)
            .push_bind(BigDecimal::from(nullifier.nullifier_index))
            .push_bind(BigDecimal::from(nullifier.block_number))
            .push_bind(BigDecimal::from(nullifier.timestamp))
            .push_bind(BigDecimal::from(BigInt::from(nullifier.internal_time)));
    });

    let query = query_builder.build();
    query.execute(&mut **db_tx).await?;

    Ok(())
}
