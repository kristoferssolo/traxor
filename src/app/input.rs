use crate::error::Result;
use std::path::{Path, PathBuf};
use tokio::fs;

#[derive(Debug, Default)]
pub struct InputHandler {
    pub text: String,
    pub cursor_position: usize,
    pub completions: Vec<String>,
    pub completion_idx: usize,
}

impl InputHandler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_char(&mut self, ch: char) {
        self.text.insert(self.cursor_position, ch);
        self.cursor_position += 1;
    }

    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
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
        let path = PathBuf::from(&self.text);
        let (base_path, partial_name) = split_path_components(path);
        let matches = find_matching_entries(&base_path, &partial_name).await?;

        self.update_completions(matches);
        self.update_from_completions();
        Ok(())
    }

    fn update_completions(&mut self, matches: Vec<String>) {
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

    fn update_from_completions(&mut self) {
        if let Some(completions) = self.completions.get(self.completion_idx) {
            self.set_text(completions.clone());
        }
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
