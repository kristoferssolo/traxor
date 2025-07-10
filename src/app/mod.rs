pub mod action;
mod command;
mod tab;
mod torrent;
pub mod types;
pub mod utils;

use crate::config::Config;
use color_eyre::Result;
use ratatui::widgets::TableState;
use std::path::{Path, PathBuf};
use tokio::fs;
use types::Selected;
pub use {tab::Tab, torrent::Torrents};

/// Main Application.
#[derive(Debug)]
pub struct App<'a> {
    pub running: bool,
    index: usize,
    tabs: &'a [Tab],
    pub state: TableState,
    pub torrents: Torrents,
    pub show_help: bool,
    pub config: Config,
    pub input: String,
    pub cursor_position: usize,
    pub input_mode: bool,
    pub completions: Vec<String>,
    pub completion_idx: usize,
}

impl App<'_> {
    /// Constructs a new instance of [`App`].
    /// Returns instance of `Self`.
    ///
    /// # Errors
    ///
    /// TODO: add error types
    pub fn new(config: Config) -> Result<Self> {
        Ok(Self {
            running: true,
            tabs: &[Tab::All, Tab::Active, Tab::Downloading],
            index: 0,
            state: TableState::default(),
            torrents: Torrents::new()?, // Handle the Result here
            show_help: false,
            config,
            input: String::new(),
            cursor_position: 0,
            input_mode: false,
            completions: Vec::new(),
            completion_idx: 0,
        })
    }

    /// # Errors
    ///
    /// TODO: add error types
    pub async fn complete_input(&mut self) -> Result<()> {
        let path = PathBuf::from(&self.input);
        let (base_path, partial_name) = split_path_components(path);
        let matches = find_matching_entries(&base_path, &partial_name).await?;

        self.update_completions(matches);
        self.update_input_with_matches();

        Ok(())
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
    pub const fn tabs(&self) -> &[Tab] {
        self.tabs
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
        self.torrents.move_selection(&self.input).await?;
        self.input.clear();
        self.cursor_position = 0;
        self.input_mode = false;
        Ok(())
    }

    pub fn prepare_move_action(&mut self) {
        if let Some(download_dir) = self.get_current_downlaod_dir() {
            self.update_cursor(&download_dir);
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

    fn update_completions(&mut self, matches: Vec<String>) {
        if matches.is_empty() {
            self.completions.clear();
            self.completion_idx = 0;
            return;
        }

        if matches != self.completions {
            self.completions = matches;
            self.completion_idx = 0;
            return;
        }

        self.completion_idx = (self.completion_idx + 1) % self.completions.len();
    }

    fn update_input_with_matches(&mut self) {
        if let Some(completion) = self.completions.get(self.completion_idx) {
            self.input = completion.clone();
            self.cursor_position = self.input.len();
        }
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

    fn update_cursor(&mut self, path: &Path) {
        self.input = path.to_string_lossy().to_string();
        self.cursor_position = self.input.len();
    }
}

fn split_path_components(path: PathBuf) -> (PathBuf, String) {
    if path.is_dir() {
        return (path, String::new());
    }

    let partial = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let base = path
        .parent()
        .unwrap_or_else(|| Path::new("/"))
        .to_path_buf();

    (base, partial)
}

async fn find_matching_entries(base_path: &Path, partial_name: &str) -> Result<Vec<String>> {
    let mut entries = fs::read_dir(&base_path).await?;
    let mut matches = Vec::new();

    while let Some(entry) = entries.next_entry().await? {
        let file_name = entry.file_name().to_string_lossy().to_string();

        if file_name
            .to_lowercase()
            .starts_with(&partial_name.to_lowercase())
        {
            matches.push(format!("{}/{}", base_path.to_string_lossy(), file_name));
        }
    }

    Ok(matches)
}
