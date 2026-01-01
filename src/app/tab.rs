use crate::config::tabs::TabConfig;
use std::fmt::Display;
use transmission_rpc::types::TorrentGetField;

/// A tab with name and column configuration.
#[derive(Debug, Clone)]
pub struct Tab {
    config: TabConfig,
    fields: Vec<TorrentGetField>,
}

impl Tab {
    /// Create a new tab from config.
    #[must_use]
    pub fn new(config: TabConfig) -> Self {
        let fields = config.fields();
        Self { config, fields }
    }

    /// Returns the column fields for this tab.
    #[must_use]
    pub fn fields(&self) -> &[TorrentGetField] {
        &self.fields
    }

    /// Returns the tab name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.config.name
    }
}

impl Display for Tab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.config.name)
    }
}
