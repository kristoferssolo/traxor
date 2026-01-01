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
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
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
    Filter,
    /// Confirm delete dialog. Bool indicates whether to delete local data.
    ConfirmDelete(bool),
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
    pub filter_text: String,
}

impl App {
    /// Constructs a new instance of [`App`].
    /// Returns instance of `Self`.
    ///
    /// # Errors
    ///
    /// TODO: add error types
    pub fn new(config: Config) -> Result<Self> {
        let tabs = config.tabs.iter().cloned().map(Tab::new).collect();
        Ok(Self {
            running: true,
            tabs,
            index: 0,
            state: TableState::default(),
            torrents: Torrents::new()?,
            show_help: false,
            config,
            input_handler: InputHandler::new(),
            input_mode: InputMode::None,
            filter_text: String::new(),
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
        self.index = self
            .index
            .checked_sub(1)
            .unwrap_or_else(|| self.tabs.len() - 1);
    }

    /// Switches to the tab whose index is `idx` if it exists.
    #[inline]
    pub const fn switch_tab(&mut self, idx: usize) {
        if idx < self.tabs.len() {
            self.close_help();
            self.index = idx;
        }
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

    /// Start filter mode.
    pub fn start_filter(&mut self) {
        self.input_handler.set_text(self.filter_text.clone());
        self.input_mode = InputMode::Filter;
    }

    /// Apply filter from input.
    pub fn apply_filter(&mut self) {
        self.filter_text = self.input_handler.text.clone();
        self.input_handler.clear();
        self.input_mode = InputMode::None;
        self.state.select(Some(0));
    }

    /// Clear the active filter.
    pub fn clear_filter(&mut self) {
        self.filter_text.clear();
        self.input_handler.clear();
        self.input_mode = InputMode::None;
    }

    /// Get the active filter text (live from input or saved).
    #[must_use]
    pub fn active_filter(&self) -> &str {
        if self.input_mode == InputMode::Filter {
            &self.input_handler.text
        } else {
            &self.filter_text
        }
    }

    /// Get filtered torrents based on current filter text using fuzzy matching.
    #[must_use]
    pub fn filtered_torrents(&self) -> Vec<&transmission_rpc::types::Torrent> {
        let filter = self.active_filter();
        if filter.is_empty() {
            self.torrents.torrents.iter().collect()
        } else {
            let matcher = SkimMatcherV2::default();
            let mut scored: Vec<_> = self
                .torrents
                .torrents
                .iter()
                .filter_map(|t| {
                    t.name
                        .as_ref()
                        .and_then(|name| matcher.fuzzy_match(name, filter).map(|score| (t, score)))
                })
                .collect();
            // Sort by score descending (best matches first)
            scored.sort_by(|a, b| b.1.cmp(&a.1));
            scored.into_iter().map(|(t, _)| t).collect()
        }
    }

    /// Prepare delete confirmation dialog.
    pub const fn prepare_delete(&mut self, delete_local_data: bool) {
        self.input_mode = InputMode::ConfirmDelete(delete_local_data);
    }

    /// Execute the confirmed delete action.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn confirm_delete(&mut self) -> Result<()> {
        let InputMode::ConfirmDelete(delete_local_data) = self.input_mode else {
            return Ok(());
        };
        let ids = self.selected(false);
        self.torrents.delete(ids, delete_local_data).await?;
        self.clear_input();
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
