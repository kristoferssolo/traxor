use derive_more::Display;

#[derive(Debug, Clone, PartialEq, Eq, Display)]
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
    #[display("First Torrent")]
    FirstTorrent,
    #[display("Last Torrent")]
    LastTorrent,
    #[display("Page Down")]
    PageDown,
    #[display("Page Up")]
    PageUp,
    #[display("Switch to Tab {}", _0)]
    SwitchTab(u8),
    #[display("Toggle Help")]
    ToggleHelp,
    #[display("Toggle Torrent")]
    ToggleTorrent,
    #[display("Toggle All Torrents")]
    ToggleAll,
    #[display("Move Torrent(-s)")]
    Move,
    #[display("Delete Torrent(-s) (force: {})", _0)]
    Delete(bool),
    #[display("Rename Torrent")]
    Rename,
    #[display("Filter")]
    Filter,
    #[display("Clear Filter")]
    ClearFilter,
    #[display("Select")]
    Select,
    #[display("Submit")]
    Submit,
    #[display("Confirm Yes")]
    ConfirmYes,
    #[display("Cancel")]
    Cancel,
}
