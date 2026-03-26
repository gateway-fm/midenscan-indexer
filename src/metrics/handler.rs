use prometheus::{Encoder, TextEncoder};

pub fn metrics_text() -> Result<Vec<u8>, anyhow::Error> {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer)?;
    Ok(buffer)
}
