mod tab;
mod torrent;
pub mod utils;

use ratatui::widgets::TableState;
use transmission_rpc::types::Torrent;
mod command;

pub use self::{tab::Tab, torrent::Torrents};

/// Main Application.
/// TODO: write description
#[derive(Debug)]
pub struct App<'a> {
    pub running: bool,
    index: usize,
    tabs: &'a [Tab],
    pub state: TableState,
    pub torrents: Torrents,
    pub show_popup: bool,
}

impl<'a> App<'a> {
    /// Constructs a new instance of [`App`].
    /// Returns instance of `Self`.
    pub fn new() -> Self {
        Self {
            running: true,
            tabs: &[Tab::All, Tab::Active, Tab::Downloading],
            index: 0,
            state: TableState::default(),
            torrents: Torrents::new(),
            show_popup: false,
        }
    }

    /// Handles the tick event of the terminal.
    pub async fn tick(&mut self) {
        self.torrents.update().await;
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.torrents.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.close_popup();
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.torrents.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.close_popup();
        self.state.select(Some(i));
    }

    /// Switches to the next tab.
    pub fn next_tab(&mut self) {
        self.close_popup();
        self.index = (self.index + 1) % self.tabs.len();
    }

    /// Switches to the previous tab.
    pub fn prev_tab(&mut self) {
        self.close_popup();
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.tabs.len() - 1;
        }
    }

    /// Switches to the tab whose index is `idx`.
    pub fn switch_tab(&mut self, idx: usize) {
        self.close_popup();
        self.index = idx
    }

    /// Returns current active [`Tab`] number
    pub fn index(&self) -> usize {
        self.index
    }

    /// Returns [`Tab`] slice
    pub fn tabs(&self) -> &[Tab] {
        self.tabs
    }

    pub fn toggle_popup(&mut self) {
        self.show_popup = !self.show_popup;
    }

    pub fn close_popup(&mut self) {
        self.show_popup = false;
    }

    pub fn open_popup(&mut self) {
        self.show_popup = true;
    }

    pub async fn toggle_torrent(&mut self) {
        let torrent = self.selected().expect("Torrent not found");
        self.torrents.toggle(&torrent.clone()).await;
        self.close_popup();
    }

    fn selected(&self) -> Option<&Torrent> {
        let idx = self.state.selected()?;
        let torrent = self.torrents.torrents().get(idx)?;
        Some(torrent)
    }
}
