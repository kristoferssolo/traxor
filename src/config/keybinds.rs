use filecaster::FromFile;

#[derive(Debug, Clone, FromFile)]
pub struct KeybindsConfig {
    #[from_file(default = "q")]
    pub quit: String,
    #[from_file(default = "l")]
    pub next_tab: String,
    #[from_file(default = "h")]
    pub prev_tab: String,
    #[from_file(default = "j")]
    pub next_torrent: String,
    #[from_file(default = "k")]
    pub prev_torrent: String,
    #[from_file(default = "1")]
    pub switch_tab_1: String,
    #[from_file(default = "2")]
    pub switch_tab_2: String,
    #[from_file(default = "3")]
    pub switch_tab_3: String,
    #[from_file(default = "enter")]
    pub toggle_torrent: String,
    #[from_file(default = "a")]
    pub toggle_all: String,
    #[from_file(default = "d")]
    pub delete: String,
    #[from_file(default = "D")]
    pub delete_force: String,
    #[from_file(default = " ")]
    pub select: String,
    #[from_file(default = "?")]
    pub toggle_help: String,
    #[from_file(default = "m")]
    pub move_torrent: String,
    #[from_file(default = "r")]
    pub rename_torrent: String,
}
