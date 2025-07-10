use derive_macro::FromFile;
use merge::{option::overwrite_none, Merge};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Merge)]
pub struct LogConfigFile {
    #[merge(strategy = overwrite_none)]
    pub traxor: Option<String>,
    #[merge(strategy = overwrite_none)]
    pub ratatui: Option<String>,
    #[merge(strategy = overwrite_none)]
    pub transmission_rpc: Option<String>,
}

#[derive(Debug, Clone, FromFile)]
pub struct LogConfig {
    pub traxor: String,
    pub ratatui: String,
    pub transmission_rpc: String,
}

impl Default for LogConfigFile {
    fn default() -> Self {
        Self {
            traxor: Some("warn".to_string()),
            ratatui: Some("warn".to_string()),
            transmission_rpc: Some("warn".to_string()),
        }
    }
}
