use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TimeConfig {
    #[serde(default = "default_date_format")]
    pub date_format: String,
    #[serde(default = "default_eta_format")]
    pub eta_format: String,
}

impl Default for TimeConfig {
    fn default() -> Self {
        Self {
            date_format: default_date_format(),
            eta_format: default_eta_format(),
        }
    }
}

impl TimeConfig {
    #[must_use]
    pub fn use_compact_eta(&self) -> bool {
        !matches!(self.eta_format.to_lowercase().as_str(), "seconds")
    }
}

fn default_date_format() -> String {
    "%Y-%m-%d %H:%M".into()
}

fn default_eta_format() -> String {
    "compact".into()
}
