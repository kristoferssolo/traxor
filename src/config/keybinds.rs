use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KeybindsConfig {
    pub quit: String,
    pub next_tab: String,
    pub prev_tab: String,
    pub next_torrent: String,
    pub prev_torrent: String,
    pub switch_tab_1: String,
    pub switch_tab_2: String,
    pub switch_tab_3: String,
    pub switch_tab_4: String,
    pub switch_tab_5: String,
    pub toggle_torrent: String,
    pub toggle_all: String,
    pub delete: String,
    pub delete_force: String,
    pub select: String,
    pub toggle_help: String,
    pub move_torrent: String,
    pub rename_torrent: String,
}
