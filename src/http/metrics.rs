use crate::metrics;
use axum::http::{HeaderValue, StatusCode};
use axum::response::Response;

pub async fn metrics_handler() -> Response {
    match metrics::handler::metrics_text() {
        Ok(bytes) => Response::builder()
            .status(StatusCode::OK)
            .header(
                axum::http::header::CONTENT_TYPE,
                HeaderValue::from_static("text/plain; version=0.0.4; charset=utf-8"),
            )
            .body(axum::body::Body::from(bytes))
            .unwrap(),
        Err(err) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(axum::body::Body::from(format!(
                "metrics encode error: {}",
                err
            )))
            .unwrap(),
    }
}
