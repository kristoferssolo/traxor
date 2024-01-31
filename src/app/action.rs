#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Quit,
    NextTab,
    PrevTab,
    NextTorrent,
    PrevTorrent,
    SwitchTab(u8),
    TogglePopup,
    ToggleTorrent,
    ToggleAll,
    PauseAll,
    StartAll,
    Move,
    Delete(bool),
    Rename,
}
