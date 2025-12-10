pub mod action;
mod command;
pub mod constants;
mod input;
mod tab;
mod torrent;
pub mod types;
pub mod utils;

use crate::error::Result;
use crate::{app::input::InputHandler, config::Config};
use ratatui::widgets::TableState;
use std::path::PathBuf;
use types::Selected;
pub use {tab::Tab, torrent::Torrents};

/// Main Application.
#[derive(Debug)]
pub struct App {
    pub running: bool,
    index: usize,
    tabs: Vec<Tab>,
    pub state: TableState,
    pub torrents: Torrents,
    pub show_help: bool,
    pub config: Config,
    pub input_handler: InputHandler,
    pub input_mode: bool,
}

impl App {
    /// Constructs a new instance of [`App`].
    /// Returns instance of `Self`.
    ///
    /// # Errors
    ///
    /// TODO: add error types
    pub fn new(config: Config) -> Result<Self> {
        Ok(Self {
            running: true,
            tabs: vec![Tab::All, Tab::Active, Tab::Downloading],
            index: 0,
            state: TableState::default(),
            torrents: Torrents::new()?, // Handle the Result here
            show_help: false,
            config,
            input_handler: InputHandler::new(),
            input_mode: false,
        })
    }

    /// # Errors
    ///
    /// TODO: add error types
    pub async fn complete_input(&mut self) -> Result<()> {
        self.input_handler.complete().await
    }

    /// Handles the tick event of the terminal.
    ///
    /// # Errors
    ///
    /// TODO: add error types
    pub async fn tick(&mut self) -> Result<()> {
        self.torrents.update().await?;
        Ok(())
    }

    /// Set running to false to quit the application.
    #[inline]
    pub const fn quit(&mut self) {
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
        self.close_help();
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
        self.close_help();
        self.state.select(Some(i));
    }

    /// Switches to the next tab.
    #[inline]
    pub const fn next_tab(&mut self) {
        self.close_help();
        self.index = (self.index + 1) % self.tabs.len();
    }

    /// Switches to the previous tab.
    #[inline]
    pub const fn prev_tab(&mut self) {
        self.close_help();
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.tabs.len() - 1;
        }
    }

    /// Switches to the tab whose index is `idx`.
    #[inline]
    pub const fn switch_tab(&mut self, idx: usize) {
        self.close_help();
        self.index = idx;
    }

    /// Returns current active [`Tab`] number
    #[inline]
    #[must_use]
    pub const fn index(&self) -> usize {
        self.index
    }

    /// Returns [`Tab`] slice
    #[inline]
    #[must_use]
    pub fn tabs(&self) -> &[Tab] {
        &self.tabs
    }

    #[inline]
    pub const fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    #[inline]
    pub const fn close_help(&mut self) {
        self.show_help = false;
    }

    #[inline]
    pub const fn open_help(&mut self) {
        self.show_help = true;
    }

    /// # Errors
    ///
    /// TODO: add error types
    pub async fn toggle_torrents(&mut self) -> Result<()> {
        let ids = self.selected(false);
        self.torrents.toggle(ids).await?;
        self.close_help();
        Ok(())
    }

    /// # Errors
    ///
    /// TODO: add error types
    pub async fn delete(&mut self, delete_local_data: bool) -> Result<()> {
        let ids = self.selected(false);
        self.torrents.delete(ids, delete_local_data).await?;
        self.close_help();
        Ok(())
    }

    /// # Errors
    ///
    /// TODO: add error types
    pub async fn move_torrent(&mut self) -> Result<()> {
        self.torrents
            .move_selection(&self.input_handler.text)
            .await?;
        self.input_handler.clear();
        self.input_mode = false;
        Ok(())
    }

    pub fn prepare_move_action(&mut self) {
        if let Some(download_dir) = self.get_current_downlaod_dir() {
            self.input_handler
                .set_text(download_dir.to_string_lossy().to_string());
        }
        self.input_mode = true;
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
            let selected_id = self
                .state
                .selected()
                .and_then(|idx| torrents.get(idx).and_then(|t| t.id));
            if let Some(id) = selected_id {
                return Selected::Current(id);
            }
        }
        let selected_torrents = torrents
            .iter()
            .filter_map(|t| t.id.filter(|id| self.torrents.selected.contains(id)))
            .collect();
        Selected::List(selected_torrents)
    }

    fn get_current_downlaod_dir(&self) -> Option<PathBuf> {
        match self.selected(true) {
            Selected::Current(current_id) => self
                .torrents
                .torrents
                .iter()
                .find(|&t| t.id == Some(current_id))
                .and_then(|t| t.download_dir.as_ref())
                .map(PathBuf::from),
            Selected::List(_) => None,
        }
    }
}
