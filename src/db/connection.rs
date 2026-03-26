use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Database {
    pub db_conn: sqlx::Pool<sqlx::Postgres>,
}

impl Database {
    pub async fn new(db_url: &str) -> Self {
        let connect_options: sqlx::postgres::PgConnectOptions =
            db_url.parse().expect("Failed to parse database URL");

        let db_conn = sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect_with(connect_options)
            .await
            .expect("Unable to connect to the database");

        Self { db_conn }
    }

    pub async fn execute_transaction<F, T>(&self, operation: F) -> Result<T>
    where
        for<'a> F: FnOnce(
            &'a mut sqlx::Transaction<'_, sqlx::postgres::Postgres>,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<T>> + Send + 'a>,
        >,
    {
        let mut db_tx = self.db_conn.begin().await?;
        let result: std::result::Result<T, anyhow::Error> = operation(&mut db_tx).await;

        match result {
            Ok(value) => {
                db_tx.commit().await?;
                Ok(value)
            }
            Err(e) => {
                db_tx.rollback().await?;
                Err(e)
            }
        }
    }
}
