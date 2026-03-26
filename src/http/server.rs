use axum::{routing::get, Router};
use log::error;

use crate::config;
use crate::http::health::health_handler;
use crate::http::metrics::metrics_handler;

pub async fn run() {
    let port = config::CONFIG.http_port;
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/metrics", get(metrics_handler));

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(err) => {
            error!("failed to bind http server to {}: {}", addr, err);
            return;
        }
    };

    if let Err(err) = axum::serve(listener, app).await {
        error!("http server error: {}", err);
    }
}
