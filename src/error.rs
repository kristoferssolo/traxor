use thiserror::Error;

#[derive(Debug, Error)]
pub enum TraxorError {
    #[error("Transmission RPC error: {0}")]
    TransmissionRpc(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("No torrent selected")]
    NoSelection,

    #[error("Invalid torrent ID: {0}")]
    InvalidTorrentId(i64),
}

impl From<Box<dyn std::error::Error + Send + Sync>> for TraxorError {
    fn from(e: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self::TransmissionRpc(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, TraxorError>;
