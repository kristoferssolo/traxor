use crate::config::tabs::TabConfig;
use std::fmt::Display;
use transmission_rpc::types::{Torrent, TorrentGetField, TorrentStatus};

/// A tab with name and column configuration.
#[derive(Debug, Clone)]
pub struct Tab {
    config: TabConfig,
    fields: Vec<TorrentGetField>,
    statuses: Vec<TorrentStatus>,
}

impl Tab {
    /// Create a new tab from config.
    #[must_use]
    pub fn new(config: TabConfig) -> Self {
        let fields = config.fields();
        let statuses = config.statuses();
        Self {
            config,
            fields,
            statuses,
        }
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

    /// Returns `true` if the torrent belongs in this tab.
    #[must_use]
    pub fn matches(&self, torrent: &Torrent) -> bool {
        self.statuses.is_empty()
            || torrent
                .status
                .is_some_and(|status| self.statuses.contains(&status))
    }
}

impl Display for Tab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.config.name)
    }
}
