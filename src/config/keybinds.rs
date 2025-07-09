use crate::merge::Merge;
use derive_macro::Merge;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Merge)]
pub struct KeybindsConfig {
    pub quit: Option<String>,
    pub next_tab: Option<String>,
    pub prev_tab: Option<String>,
    pub next_torrent: Option<String>,
    pub prev_torrent: Option<String>,
    pub switch_tab_1: Option<String>,
    pub switch_tab_2: Option<String>,
    pub switch_tab_3: Option<String>,
    pub toggle_torrent: Option<String>,
    pub toggle_all: Option<String>,
    pub delete: Option<String>,
    pub delete_force: Option<String>,
    pub select: Option<String>,
    pub toggle_help: Option<String>,
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
