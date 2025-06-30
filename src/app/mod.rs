mod tab;
mod torrent;
pub mod utils;

use ratatui::widgets::TableState;
pub mod action;
mod command;
pub mod types;

use self::types::Selected;
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
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            running: true,
            tabs: &[Tab::All, Tab::Active, Tab::Downloading],
            index: 0,
            state: TableState::default(),
            torrents: Torrents::new()?, // Handle the Result here
            show_popup: false,
        })
    }

    /// Handles the tick event of the terminal.
    pub async fn tick(&mut self) -> anyhow::Result<()> {
        self.torrents.update().await?;
        Ok(())
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

    pub async fn toggle_torrents(&mut self) -> anyhow::Result<()> {
        let ids = self.selected(false);
        self.torrents.toggle(ids).await?;
        self.close_popup();
        Ok(())
    }

    pub async fn delete(&mut self, delete_local_data: bool) -> anyhow::Result<()> {
        let ids = self.selected(false);
        self.torrents.delete(ids, delete_local_data).await?;
        self.close_popup();
        Ok(())
    }

    pub fn select(&mut self) {
        if let Selected::Current(current_id) = self.selected(true) {
            if self.torrents.selected.contains(&current_id) {
                self.torrents.selected.remove(&current_id);
            } else {
                self.torrents.selected.insert(current_id);
            }
        }
        self.next();
    }

    fn selected(&self, highlighted: bool) -> Selected {
        let torrents = &self.torrents.torrents;
        if self.torrents.selected.is_empty() || highlighted {
            let selected_id = self.state.selected().and_then(|idx| torrents.get(idx).and_then(|torrent| torrent.id));
            if let Some(id) = selected_id {
                return Selected::Current(id);
            }
        }
        let selected_torrents = torrents
            .iter()
            .filter_map(|torrent| torrent.id.filter(|id| self.torrents.selected.contains(id)))
            .collect();
        Selected::List(selected_torrents)
    }
}
