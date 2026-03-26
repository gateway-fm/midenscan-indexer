use axum::Json;
use log::warn;

use serde::Serialize;
use std::time::Duration;

use crate::config::CONFIG;

#[derive(Serialize, Debug)]
struct ComponentStatus {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Serialize)]
pub struct HealthResponse {
    status: &'static str,
    db: ComponentStatus,
    rpc: ComponentStatus,
}

async fn check_db() -> ComponentStatus {
    // Try to acquire a connection and run a lightweight query with a short timeout
    let result = async {
        let connect_options: sqlx::postgres::PgConnectOptions = CONFIG
            .postgres_url
            .parse()
            .map_err(|e| anyhow::anyhow!(format!("parse dsn: {}", e)))?;

        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .acquire_timeout(Duration::from_secs(2))
            .connect_with(connect_options)
            .await?;

        // SELECT 1 to validate connectivity
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&pool)
            .await?;

        Ok::<(), anyhow::Error>(())
    }
    .await;

    match result {
        Ok(()) => ComponentStatus {
            ok: true,
            error: None,
        },
        Err(e) => ComponentStatus {
            ok: false,
            error: Some(e.to_string()),
        },
    }
}

async fn check_rpc() -> ComponentStatus {
    // Try to establish a gRPC connection within a short timeout
    let result = tokio::time::timeout(
        Duration::from_secs(2),
        miden_node_proto::generated::rpc::api_client::ApiClient::connect(CONFIG.rpc_url.clone()),
    )
    .await
    .map_err(|_| anyhow::anyhow!("rpc connect timeout"))
    .and_then(|r| {
        r.map(|_client| ())
            .map_err(|e| anyhow::anyhow!(e.to_string()))
    });

    match result {
        Ok(()) => ComponentStatus {
            ok: true,
            error: None,
        },
        Err(e) => ComponentStatus {
            ok: false,
            error: Some(e.to_string()),
        },
    }
}

pub async fn health_handler() -> (axum::http::StatusCode, Json<HealthResponse>) {
    let (db, rpc) = tokio::join!(check_db(), check_rpc());
    let all_ok = db.ok && rpc.ok;
    let status = if all_ok { "ok" } else { "degraded" };

    let body = HealthResponse { status, db, rpc };
    let code = if all_ok {
        axum::http::StatusCode::OK
    } else {
        warn!(
            "Health check failed: db: {:?}, rpc: {:?}",
            body.db, body.rpc
        );
        axum::http::StatusCode::SERVICE_UNAVAILABLE
    };
    (code, Json(body))
}
