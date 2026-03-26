use super::models;
use anyhow::Result;

pub async fn get_ref(
    db_tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<models::DatabaseRef, sqlx::Error> {
    let response = sqlx::query_as::<_, models::DatabaseRef>("SELECT * FROM ref where id = 0")
        .fetch_one(&mut **db_tx)
        .await
        .unwrap();
    Ok(response)
}

pub async fn update_ref(
    db_tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    database_ref: models::DatabaseRef,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE ref SET block_commitment = $1, block_number = $2 WHERE id = 0")
        .bind(database_ref.block_commitment)
        .bind(database_ref.block_number)
        .execute(&mut **db_tx)
        .await?;

    Ok(())
}
