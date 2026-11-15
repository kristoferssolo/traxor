pub mod action;
mod command;
pub mod constants;
mod input;
mod tab;
mod torrent;
pub mod types;
pub mod utils;

use crate::{
    app::{
        input::InputHandler,
        types::{DeleteTarget, Selected},
    },
    config::Config,
    error::Result,
};
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use ratatui::{layout::Rect, widgets::TableState};
use std::{cmp::Reverse, collections::HashSet, path::PathBuf};
use transmission_rpc::types::Torrent;
pub use {tab::Tab, torrent::Torrents};

/// Input mode type for the application.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum InputMode {
    #[default]
    None,
    Move,
    Rename,
    Filter,
    /// Confirm delete dialog for the torrents captured when it opened.
    ConfirmDelete(DeleteTarget),
}

/// Screen areas from the last render, used for mouse hit-testing.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RenderedAreas {
    pub tabs: Rect,
    pub table: Rect,
    /// Table scroll offset after the last render.
    pub table_offset: usize,
    /// Torrent ids of the rendered table rows in display order.
    ///
    /// The torrent list can be refreshed between a render and a click, so clicks resolve rows
    /// through these ids instead of indexing the current list.
    pub table_rows: Vec<Option<i64>>,
}

/// Transient error notice shown in the status bar.
///
/// Set with [`App::show_error`]. [`crate::handler::handle_key_event`] clears it
/// at the start of every key press, so it stays visible until the next one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusMessage {
    /// Single-line message text.
    pub text: String,
}

/// Main Application.
#[derive(Debug)]
pub struct App {
    pub running: bool,
    index: usize,
    tabs: Vec<Tab>,
    /// Id of the highlighted torrent. Actions resolve the highlight through it.
    highlighted: Option<i64>,
    /// Row of the highlighted torrent, kept in sync by [`App::sync_highlight`] for rendering.
    pub state: TableState,
    /// Torrents the open Move or Rename prompt acts on, captured when it opened.
    ///
    /// A refresh can move the highlight while the prompt is open; submitting acts on these ids
    /// instead, so it never retargets a different torrent.
    prompt_targets: Vec<i64>,
    pub torrents: Torrents,
    pub show_help: bool,
    pub config: Config,
    pub input_handler: InputHandler,
    pub input_mode: InputMode,
    pub filter_text: String,
    pub areas: RenderedAreas,
    pub status_message: Option<StatusMessage>,
}

impl App {
    /// Constructs a new instance of [`App`].
    ///
    /// # Errors
    ///
    /// Returns an error if the default RPC URL is invalid.
    pub fn new(config: Config) -> Result<Self> {
        let tabs: Vec<_> = config.tabs.iter().cloned().map(Tab::new).collect();
        let mut torrents = Torrents::new()?;
        torrents.set_fields(Some(tab::required_fields(&tabs)));
        Ok(Self {
            running: true,
            tabs,
            index: 0,
            highlighted: None,
            state: TableState::default(),
            prompt_targets: Vec::new(),
            torrents,
            show_help: false,
            config,
            input_handler: InputHandler::new(),
            input_mode: InputMode::None,
            filter_text: String::new(),
            areas: RenderedAreas::default(),
            status_message: None,
        })
    }

    /// Completes the input as a directory path, cycling matches on repeated calls.
    ///
    /// # Errors
    ///
    /// Returns an error if reading the parent directory or an entry's metadata fails.
    pub async fn complete_input(&mut self) -> Result<()> {
        self.input_handler.complete().await
    }

    /// Shows `text` as an error in the status bar until it is cleared.
    ///
    /// Whitespace runs, including newlines, are collapsed so the message
    /// fits on the single status bar line.
    pub fn show_error(&mut self, text: impl AsRef<str>) {
        let text = text
            .as_ref()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        self.status_message = Some(StatusMessage { text });
    }

    /// Removes the current status bar message, if any.
    pub fn clear_status_message(&mut self) {
        self.status_message = None;
    }

    /// Set running to false to quit the application.
    #[inline]
    pub const fn quit(&mut self) {
        self.running = false;
    }

    pub fn next(&mut self) {
        self.sync_highlight();
        let len = self.filtered_torrents().len();
        if len == 0 {
            return;
        }
        let i = self
            .state
            .selected()
            .map_or(0, |i| if i >= len - 1 { 0 } else { i + 1 });
        self.close_help();
        self.highlight_row(i);
    }

    pub fn previous(&mut self) {
        self.sync_highlight();
        let len = self.filtered_torrents().len();
        if len == 0 {
            return;
        }
        let i = self
            .state
            .selected()
            .map_or(0, |i| if i == 0 { len - 1 } else { i - 1 });
        self.close_help();
        self.highlight_row(i);
    }

    /// Moves [`App::state`] to the highlighted torrent's current row.
    ///
    /// Call after the visible rows may have changed (refresh, filter, tab switch). If the
    /// highlighted torrent is no longer visible, the previous row is clamped to the visible
    /// rows and the torrent there becomes the highlight. With no visible rows the highlight is
    /// cleared, so a torrent that reappears later is not acted on before it is drawn highlighted.
    pub fn sync_highlight(&mut self) {
        let rows = self.filtered_torrents();
        let row = self
            .highlighted
            .and_then(|id| rows.iter().position(|t| t.id == Some(id)));
        if let Some(row) = row {
            self.state.select(Some(row));
            return;
        }
        let Some(last) = rows.len().checked_sub(1) else {
            self.highlighted = None;
            self.state.select(None);
            return;
        };
        let Some(row) = self.state.selected() else {
            return;
        };
        self.highlight_row(row.min(last));
    }

    /// Highlights the visible torrent with `id`. Does nothing if it is not visible.
    pub fn highlight_torrent(&mut self, id: i64) {
        if let Some(row) = self
            .filtered_torrents()
            .iter()
            .position(|t| t.id == Some(id))
        {
            self.highlight_row(row);
        }
    }

    fn highlight_row(&mut self, row: usize) {
        self.highlighted = self.filtered_torrents().get(row).and_then(|t| t.id);
        self.state.select(Some(row));
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

    /// Toggle selected or highlighted torrent(s) between started and stopped.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
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
        let ids = Selected::List(self.prompt_targets.iter().copied().collect());
        self.torrents
            .move_torrents(ids, &self.input_handler.text)
            .await?;
        self.clear_input();
        Ok(())
    }

    /// Prepare move action by pre-filling current download directory.
    pub fn prepare_move_action(&mut self) {
        self.prompt_targets = self.selected(false).into();
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
        let target = self.prompt_targets.first().and_then(|id| {
            self.torrents
                .torrents
                .iter()
                .find(|t| t.id == Some(*id))
                .cloned()
        });
        let Some(torrent) = target else {
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
        let torrent = self.get_current_torrent();
        self.prompt_targets = torrent.as_ref().and_then(|t| t.id).into_iter().collect();
        if let Some(name) = torrent.and_then(|t| t.name) {
            self.input_handler.set_text(name);
        }
        self.input_mode = InputMode::Rename;
    }

    /// Clear input and reset input mode.
    fn clear_input(&mut self) {
        self.input_handler.clear();
        self.prompt_targets.clear();
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
        self.highlight_row(0);
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
        match self.input_mode {
            InputMode::Filter => &self.input_handler.text,
            _ => &self.filter_text,
        }
    }

    /// Get filtered torrents based on current filter text using fuzzy matching.
    #[must_use]
    pub fn filtered_torrents(&self) -> Vec<&Torrent> {
        let filter = self.active_filter();
        let tab = &self.tabs[self.index];
        if filter.is_empty() {
            return self
                .torrents
                .torrents
                .iter()
                .filter(|torrent| tab.matches(torrent))
                .collect();
        }

        let matcher = SkimMatcherV2::default();
        let mut scored = self
            .torrents
            .torrents
            .iter()
            .filter(|torrent| tab.matches(torrent))
            .filter_map(|t| {
                t.name
                    .as_ref()
                    .and_then(|name| matcher.fuzzy_match(name, filter).map(|score| (t, score)))
            })
            .collect::<Vec<_>>();
        // Sort by score descending (best matches first)
        scored.sort_by_key(|(_, score)| Reverse(*score));
        scored.into_iter().map(|(t, _)| t).collect()
    }

    /// Open the delete confirmation for the selected or highlighted torrent(s).
    ///
    /// Targets are captured now, so a refresh before confirming cannot change what gets
    /// deleted. The dialog does not open when no target is visible.
    pub fn prepare_delete(&mut self, delete_local_data: bool) {
        let ids: HashSet<i64> = self.selected(false).into();
        let torrents = self
            .filtered_torrents()
            .into_iter()
            .filter_map(|t| {
                let id = t.id.filter(|id| ids.contains(id))?;
                let name = t.name.clone().unwrap_or_else(|| format!("#{id}"));
                Some((id, name))
            })
            .collect();
        if let Some(target) = DeleteTarget::new(torrents, delete_local_data) {
            self.input_mode = InputMode::ConfirmDelete(target);
        }
    }

    /// Execute the confirmed delete action.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn confirm_delete(&mut self) -> Result<()> {
        let InputMode::ConfirmDelete(target) = &self.input_mode else {
            return Ok(());
        };
        self.torrents
            .delete(target.ids(), target.delete_local_data())
            .await?;
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
        let torrents = self.filtered_torrents();
        if (self.torrents.selected.is_empty() || highlighted)
            && let Some(id) = self
                .highlighted
                .filter(|id| torrents.iter().any(|t| t.id == Some(*id)))
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

    fn get_current_torrent(&self) -> Option<Torrent> {
        let id = self.highlighted?;
        self.filtered_torrents()
            .into_iter()
            .find(|t| t.id == Some(id))
            .cloned()
    }
}
