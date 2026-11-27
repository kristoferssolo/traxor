use crate::config::rpc::RpcConfigError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TraxorError {
    #[error("Transmission RPC error: {0}")]
    TransmissionRpc(String),

    #[error("Configuration error: {0}")]
    RpcConfig(#[from] RpcConfigError),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<Box<dyn std::error::Error + Send + Sync>> for TraxorError {
    fn from(e: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self::TransmissionRpc(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, TraxorError>;
