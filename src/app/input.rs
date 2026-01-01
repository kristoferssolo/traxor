use crate::error::Result;
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};
use tokio::fs;

#[derive(Debug, Default)]
pub struct InputHandler {
    pub text: String,
    pub cursor_position: usize,
    completions: Vec<PathBuf>,
    completion_idx: usize,
}

impl InputHandler {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_char(&mut self, ch: char) {
        self.text.insert(self.cursor_position, ch);
        self.cursor_position += ch.len_utf8();
    }

    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            let ch = self.text[..self.cursor_position]
                .chars()
                .next_back()
                .expect("cursor position is valid");
            self.cursor_position -= ch.len_utf8();
            self.text.remove(self.cursor_position);
        }
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor_position = 0;
        self.completions.clear();
        self.completion_idx = 0;
    }

    pub fn set_text(&mut self, text: String) {
        self.cursor_position = text.len();
        self.text = text;
    }

    pub async fn complete(&mut self) -> Result<()> {
        let path = Path::new(&self.text);
        let (base_path, partial_name) = split_path_components(path);
        let matches = find_matching_entries(base_path, partial_name).await?;

        self.update_completions(matches);
        self.apply_completion();
        Ok(())
    }

    fn update_completions(&mut self, matches: Vec<PathBuf>) {
        if matches.is_empty() {
            self.completions.clear();
            self.completion_idx = 0;
        } else if matches != self.completions {
            self.completions = matches;
            self.completion_idx = 0;
        } else {
            self.completion_idx = (self.completion_idx + 1) % self.completions.len();
        }
    }

    fn apply_completion(&mut self) {
        if let Some(path) = self.completions.get(self.completion_idx) {
            self.set_text(path.to_string_lossy().into_owned());
        }
    }
}

fn split_path_components(path: &Path) -> (&Path, &OsStr) {
    if path.is_dir() {
        return (path, OsStr::new(""));
    }

    let partial = path.file_name().unwrap_or_default();
    let base = path.parent().unwrap_or_else(|| Path::new("/"));
    (base, partial)
}

async fn find_matching_entries(base_path: &Path, partial_name: &OsStr) -> Result<Vec<PathBuf>> {
    let partial_lower = partial_name.to_string_lossy().to_lowercase();
    let mut entries = fs::read_dir(base_path).await?;
    let mut matches = Vec::new();

    while let Some(entry) = entries.next_entry().await? {
        let file_name = entry.file_name();
        if file_name
            .to_string_lossy()
            .to_lowercase()
            .starts_with(&partial_lower)
        {
            matches.push(base_path.join(file_name));
        }
    }
    Ok(matches)
}
