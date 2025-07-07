use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
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
}

impl KeybindsConfig {
    pub fn merge(&mut self, other: Self) {
        if let Some(quit) = other.quit {
            self.quit = Some(quit);
        }
        if let Some(next_tab) = other.next_tab {
            self.next_tab = Some(next_tab);
        }
        if let Some(prev_tab) = other.prev_tab {
            self.prev_tab = Some(prev_tab);
        }
        if let Some(next_torrent) = other.next_torrent {
            self.next_torrent = Some(next_torrent);
        }
        if let Some(prev_torrent) = other.prev_torrent {
            self.prev_torrent = Some(prev_torrent);
        }
        if let Some(switch_tab_1) = other.switch_tab_1 {
            self.switch_tab_1 = Some(switch_tab_1);
        }
        if let Some(switch_tab_2) = other.switch_tab_2 {
            self.switch_tab_2 = Some(switch_tab_2);
        }
        if let Some(switch_tab_3) = other.switch_tab_3 {
            self.switch_tab_3 = Some(switch_tab_3);
        }
        if let Some(toggle_torrent) = other.toggle_torrent {
            self.toggle_torrent = Some(toggle_torrent);
        }
        if let Some(toggle_all) = other.toggle_all {
            self.toggle_all = Some(toggle_all);
        }
        if let Some(delete) = other.delete {
            self.delete = Some(delete);
        }
        if let Some(delete_force) = other.delete_force {
            self.delete_force = Some(delete_force);
        }
        if let Some(select) = other.select {
            self.select = Some(select);
        }
        if let Some(toggle_help) = other.toggle_help {
            self.toggle_help = Some(toggle_help);
        }
    }
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
        }
    }
}
