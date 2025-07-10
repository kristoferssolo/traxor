use merge::Merge;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Merge)]
pub struct KeybindsConfig {
    #[merge(strategy = merge::option::overwrite_none)]
    pub quit: Option<String>,
    #[merge(strategy = merge::option::overwrite_none)]
    pub next_tab: Option<String>,
    #[merge(strategy = merge::option::overwrite_none)]
    pub prev_tab: Option<String>,
    #[merge(strategy = merge::option::overwrite_none)]
    pub next_torrent: Option<String>,
    #[merge(strategy = merge::option::overwrite_none)]
    pub prev_torrent: Option<String>,
    #[merge(strategy = merge::option::overwrite_none)]
    pub switch_tab_1: Option<String>,
    #[merge(strategy = merge::option::overwrite_none)]
    pub switch_tab_2: Option<String>,
    #[merge(strategy = merge::option::overwrite_none)]
    pub switch_tab_3: Option<String>,
    #[merge(strategy = merge::option::overwrite_none)]
    pub toggle_torrent: Option<String>,
    #[merge(strategy = merge::option::overwrite_none)]
    pub toggle_all: Option<String>,
    #[merge(strategy = merge::option::overwrite_none)]
    pub delete: Option<String>,
    #[merge(strategy = merge::option::overwrite_none)]
    pub delete_force: Option<String>,
    #[merge(strategy = merge::option::overwrite_none)]
    pub select: Option<String>,
    #[merge(strategy = merge::option::overwrite_none)]
    pub toggle_help: Option<String>,
    #[merge(strategy = merge::option::overwrite_none)]
    pub move_torrent: Option<String>,
}

impl Default for KeybindsConfig {
    fn default() -> Self {
        Self {
            quit: Some("q".to_string()),
            next_tab: Some("l".to_string()),
            prev_tab: Some("h".to_string()),
            next_torrent: Some("j".to_string()),
            prev_torrent: Some("k".to_string()),
            switch_tab_1: Some("1".to_string()),
            switch_tab_2: Some("2".to_string()),
            switch_tab_3: Some("3".to_string()),
            toggle_torrent: Some("enter".to_string()),
            toggle_all: Some("a".to_string()),
            delete: Some("d".to_string()),
            delete_force: Some("D".to_string()),
            select: Some(" ".to_string()),
            toggle_help: Some("?".to_string()),
            move_torrent: Some("m".to_string()),
        }
    }
}
