#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Quit,
    NextTab,
    PrevTab,
    NextTorrent,
    PrevTorrent,
    SwitchTab(u8),
    ToggleHelp, // Add this line
    ToggleTorrent,
    ToggleAll,
    PauseAll,
    StartAll,
    Move,
    Delete(bool),
    Rename,
    Select,
}
