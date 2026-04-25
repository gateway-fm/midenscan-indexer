use super::models::DatabaseAccountVerifiedComponent;

use anyhow::Result;
use sqlx::QueryBuilder;

pub async fn insert_standard_components(
    db_tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    components: Vec<DatabaseAccountVerifiedComponent>,
) -> Result<(), sqlx::Error> {
    if components.is_empty() {
        return Ok(());
    }

    let mut query_builder: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new(
        "INSERT INTO account_verified_component (id, name, procedure_digests, rust, masm, timestamp, is_custom) ",
    );
    query_builder.push_values(components, |mut b, component| {
        b.push_bind(component.id)
            .push_bind(component.name)
            .push_bind(component.procedure_digests)
            .push_bind(component.rust)
            .push_bind(component.masm)
            .push_bind(component.timestamp)
            .push_bind(component.is_custom);
    });
    query_builder.push(" ON CONFLICT (id) DO NOTHING");

    let query = query_builder.build();
    query.execute(&mut **db_tx).await?;

    Ok(())
}

pub async fn get_existing_procedure_digests(
    db_tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<Vec<Vec<String>>, sqlx::Error> {
    let rows: Vec<(Vec<String>,)> = sqlx::query_as(
        "SELECT procedure_digests FROM account_verified_component WHERE is_custom = false",
    )
    .fetch_all(&mut **db_tx)
    .await?;
    Ok(rows.into_iter().map(|(digests,)| digests).collect())
}
