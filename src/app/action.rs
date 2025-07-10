use derive_more::Display;

#[derive(Debug, Clone, PartialEq, Display)]
pub enum Action {
    #[display("Quit")]
    Quit,
    #[display("Next Tab")]
    NextTab,
    #[display("Previous Tab")]
    PrevTab,
    #[display("Next Torrent")]
    NextTorrent,
    #[display("Previous Torrent")]
    PrevTorrent,
    #[display("Switch to Tab {}", _0)]
    SwitchTab(u8),
    #[display("Toggle Help")]
    ToggleHelp,
    #[display("Toggle Torrent")]
    ToggleTorrent,
    #[display("Toggle All Torrents")]
    ToggleAll,
    #[display("Pause All Torrents")]
    PauseAll,
    #[display("Start All Torrents")]
    StartAll,
    #[display("Move Torrent(-s)")]
    Move,
    #[display("Delete Torrent(-s) (force: {})", _0)]
    Delete(bool),
    #[display("Rename Torrent")]
    Rename,
    #[display("Select")]
    Select,
    #[display("Submit")]
    Submit,
    #[display("Cancel")]
    Cancel,
}
