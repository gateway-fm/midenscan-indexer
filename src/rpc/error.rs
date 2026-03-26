use std::fmt;

#[derive(Debug, Clone)]
pub enum RpcError {
    Timeout(&'static str),
    NotFound(u32),
    Unknown(String),
}

impl fmt::Display for RpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RpcError::Timeout(ctx) => write!(f, "timeout: {}", ctx),
            RpcError::NotFound(block) => write!(f, "block not found: {}", block),
            RpcError::Unknown(msg) => write!(f, "unknown error: {}", msg),
        }
    }
}

impl std::error::Error for RpcError {}
