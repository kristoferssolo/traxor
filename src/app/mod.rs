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

/// Input mode type for the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputMode {
    #[default]
    None,
    Move,
    Rename,
}

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
    pub input_mode: InputMode,
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
            torrents: Torrents::new()?,
            show_help: false,
            config,
            input_handler: InputHandler::new(),
            input_mode: InputMode::None,
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
        let len = self.torrents.len();
        if len == 0 {
            return;
        }
        let i = self
            .state
            .selected()
            .map_or(0, |i| if i >= len - 1 { 0 } else { i + 1 });
        self.close_help();
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let len = self.torrents.len();
        if len == 0 {
            return;
        }
        let i = self
            .state
            .selected()
            .map_or(0, |i| if i == 0 { len - 1 } else { i - 1 });
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
    pub fn prev_tab(&mut self) {
        self.close_help();
        self.index = self.index.checked_sub(1).unwrap_or(self.tabs.len() - 1);
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

    /// Move selected or highlighted torrent(s) to a new location.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn move_torrent(&mut self) -> Result<()> {
        let ids = self.selected(false);
        self.torrents
            .move_torrents(ids, &self.input_handler.text)
            .await?;
        self.clear_input();
        Ok(())
    }

    /// Prepare move action by pre-filling current download directory.
    pub fn prepare_move_action(&mut self) {
        if let Some(download_dir) = self.get_current_download_dir() {
            self.input_handler
                .set_text(download_dir.to_string_lossy().into_owned());
        }
        self.input_mode = InputMode::Move;
    }

    /// Rename the highlighted torrent.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn rename_torrent(&mut self) -> Result<()> {
        let Some(torrent) = self.get_current_torrent() else {
            self.clear_input();
            return Ok(());
        };
        self.torrents
            .rename(&torrent, std::path::Path::new(&self.input_handler.text))
            .await?;
        self.clear_input();
        Ok(())
    }

    /// Prepare rename action by pre-filling current torrent name.
    pub fn prepare_rename_action(&mut self) {
        if let Some(name) = self.get_current_torrent_name() {
            self.input_handler.set_text(name);
        }
        self.input_mode = InputMode::Rename;
    }

    /// Clear input and reset input mode.
    fn clear_input(&mut self) {
        self.input_handler.clear();
        self.input_mode = InputMode::None;
        self.close_help();
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
        if (self.torrents.selected.is_empty() || highlighted)
            && let Some(id) = self
                .state
                .selected()
                .and_then(|idx| torrents.get(idx).and_then(|t| t.id))
        {
            return Selected::Current(id);
        }
        Selected::List(
            torrents
                .iter()
                .filter_map(|t| t.id.filter(|id| self.torrents.selected.contains(id)))
                .collect(),
        )
    }

    fn get_current_download_dir(&self) -> Option<PathBuf> {
        self.get_current_torrent()
            .and_then(|t| t.download_dir)
            .map(PathBuf::from)
    }

    fn get_current_torrent_name(&self) -> Option<String> {
        self.get_current_torrent().and_then(|t| t.name)
    }

    fn get_current_torrent(&self) -> Option<transmission_rpc::types::Torrent> {
        let Selected::Current(current_id) = self.selected(true) else {
            return None;
        };
        self.torrents
            .torrents
            .iter()
            .find(|t| t.id == Some(current_id))
            .cloned()
    }
}
