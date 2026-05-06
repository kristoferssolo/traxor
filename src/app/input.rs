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
        self.clear_completions();
    }

    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            let ch = self.text[..self.cursor_position]
                .chars()
                .next_back()
                .expect("cursor position is valid");
            self.cursor_position -= ch.len_utf8();
            self.text.remove(self.cursor_position);
            self.clear_completions();
        }
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor_position = 0;
        self.clear_completions();
    }

    pub fn set_text(&mut self, text: String) {
        self.cursor_position = text.len();
        self.text = text;
        self.clear_completions();
    }

    #[inline]
    #[must_use]
    pub fn completions(&self) -> &[PathBuf] {
        &self.completions
    }

    #[inline]
    #[must_use]
    pub fn completion_idx(&self) -> Option<usize> {
        (!self.completions.is_empty()).then_some(self.completion_idx)
    }

    pub async fn complete(&mut self) -> Result<()> {
        if self.text_matches_active_completion() {
            self.next_completion();
            self.apply_completion();
            return Ok(());
        }

        let path = Path::new(&self.text);
        let (base_path, partial_name) = split_path_components(path);
        let matches = find_matching_directories(base_path, partial_name).await?;

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
            let mut text = path.to_string_lossy().into_owned();
            if !text.ends_with('/') {
                text.push('/');
            }
            self.set_text_without_clearing_completions(text);
        }
    }

    const fn next_completion(&mut self) {
        if !self.completions.is_empty() {
            self.completion_idx = (self.completion_idx + 1) % self.completions.len();
        }
    }

    fn text_matches_active_completion(&self) -> bool {
        let Some(path) = self.completions.get(self.completion_idx) else {
            return false;
        };

        let mut text = path.to_string_lossy().into_owned();
        if !text.ends_with('/') {
            text.push('/');
        }
        self.text == text
    }

    fn clear_completions(&mut self) {
        self.completions.clear();
        self.completion_idx = 0;
    }

    fn set_text_without_clearing_completions(&mut self, text: String) {
        self.cursor_position = text.len();
        self.text = text;
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

async fn find_matching_directories(base_path: &Path, partial_name: &OsStr) -> Result<Vec<PathBuf>> {
    let partial_lower = partial_name.to_string_lossy().to_lowercase();
    let mut entries = fs::read_dir(base_path).await?;
    let mut matches = Vec::new();

    while let Some(entry) = entries.next_entry().await? {
        let file_name = entry.file_name();
        if !entry.file_type().await?.is_dir() {
            continue;
        }

        if file_name
            .to_string_lossy()
            .to_lowercase()
            .starts_with(&partial_lower)
        {
            matches.push(base_path.join(file_name));
        }
    }
    matches.sort();
    Ok(matches)
}

#[cfg(test)]
mod tests {
    use super::InputHandler;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn complete_ignores_files_and_appends_slash() {
        let dir = temp_dir();
        create_dir(dir.path().join("shows"));
        create_file(dir.path().join("short.txt"));
        let mut input = InputHandler::new();
        input.set_text(format!("{}/sh", dir.path().display()));

        input.complete().await.expect("completion should succeed");

        assert_eq!(input.completions().len(), 1);
        assert_eq!(input.text, format!("{}/shows/", dir.path().display()));
        assert_eq!(input.cursor_position, input.text.len());
    }

    #[tokio::test]
    async fn complete_cycles_directory_matches_in_stable_order() {
        let dir = temp_dir();
        create_dir(dir.path().join("zeta"));
        create_dir(dir.path().join("alpha"));
        create_dir(dir.path().join("beta"));
        let mut input = InputHandler::new();
        input.set_text(format!("{}/", dir.path().display()));

        input
            .complete()
            .await
            .expect("first completion should succeed");
        assert_eq!(input.text, format!("{}/alpha/", dir.path().display()));
        assert_eq!(input.completion_idx(), Some(0));

        input
            .complete()
            .await
            .expect("second completion should succeed");
        assert_eq!(input.text, format!("{}/beta/", dir.path().display()));
        assert_eq!(input.completion_idx(), Some(1));

        input
            .complete()
            .await
            .expect("third completion should succeed");
        assert_eq!(input.text, format!("{}/zeta/", dir.path().display()));
        assert_eq!(input.completion_idx(), Some(2));
    }

    #[tokio::test]
    async fn manual_edits_clear_stale_completions() {
        let dir = temp_dir();
        create_dir(dir.path().join("alpha"));
        let mut input = InputHandler::new();
        input.set_text(format!("{}/a", dir.path().display()));
        input.complete().await.expect("completion should succeed");

        input.insert_char('x');

        assert!(input.completions().is_empty());
        assert_eq!(input.completion_idx(), None);
    }

    #[tokio::test]
    async fn no_matches_clear_completions_and_leave_text_unchanged() {
        let dir = temp_dir();
        create_dir(dir.path().join("alpha"));
        let mut input = InputHandler::new();
        input.set_text(format!("{}/a", dir.path().display()));
        input.complete().await.expect("completion should succeed");
        input.set_text(format!("{}/missing", dir.path().display()));

        input.complete().await.expect("completion should succeed");

        assert_eq!(input.text, format!("{}/missing", dir.path().display()));
        assert!(input.completions().is_empty());
        assert_eq!(input.completion_idx(), None);
    }

    fn temp_dir() -> TempDir {
        TempDir::new().expect("temporary directory should be created")
    }

    fn create_dir(path: impl AsRef<std::path::Path>) {
        fs::create_dir(path).expect("directory should be created");
    }

    fn create_file(path: impl AsRef<std::path::Path>) {
        fs::write(path, "").expect("file should be created");
    }
}
