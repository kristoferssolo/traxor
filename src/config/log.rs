use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LogConfig {
    pub traxor: String,
    pub ratatui: String,
    pub transmission_rpc: String,
}
