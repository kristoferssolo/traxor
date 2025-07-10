use derive_macro::FromFile;
use merge::{Merge, option::overwrite_none};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Merge)]
pub struct KeybindsConfigFile {
    #[merge(strategy = overwrite_none)]
    pub quit: Option<String>,
    #[merge(strategy = overwrite_none)]
    pub next_tab: Option<String>,
    #[merge(strategy = overwrite_none)]
    pub prev_tab: Option<String>,
    #[merge(strategy = overwrite_none)]
    pub next_torrent: Option<String>,
    #[merge(strategy = overwrite_none)]
    pub prev_torrent: Option<String>,
    #[merge(strategy = overwrite_none)]
    pub switch_tab_1: Option<String>,
    #[merge(strategy = overwrite_none)]
    pub switch_tab_2: Option<String>,
    #[merge(strategy = overwrite_none)]
    pub switch_tab_3: Option<String>,
    #[merge(strategy = overwrite_none)]
    pub toggle_torrent: Option<String>,
    #[merge(strategy = overwrite_none)]
    pub toggle_all: Option<String>,
    #[merge(strategy = overwrite_none)]
    pub delete: Option<String>,
    #[merge(strategy = overwrite_none)]
    pub delete_force: Option<String>,
    #[merge(strategy = overwrite_none)]
    pub select: Option<String>,
    #[merge(strategy = overwrite_none)]
    pub toggle_help: Option<String>,
    #[merge(strategy = overwrite_none)]
    pub move_torrent: Option<String>,
}

#[derive(Debug, Clone, FromFile)]
pub struct KeybindsConfig {
    pub quit: String,
    pub next_tab: String,
    pub prev_tab: String,
    pub next_torrent: String,
    pub prev_torrent: String,
    pub switch_tab_1: String,
    pub switch_tab_2: String,
    pub switch_tab_3: String,
    pub toggle_torrent: String,
    pub toggle_all: String,
    pub delete: String,
    pub delete_force: String,
    pub select: String,
    pub toggle_help: String,
    pub move_torrent: String,
}

impl Default for KeybindsConfigFile {
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
