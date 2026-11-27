use crate::error::Result;
use std::{
    ffi::OsStr,
    io::ErrorKind,
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
        self.cursor_position = self.cursor_position.saturating_add(ch.len_utf8());
        self.clear_completions();
    }

    pub fn delete_char(&mut self) {
        let Some(ch) = self.before_cursor().chars().next_back() else {
            return;
        };
        self.cursor_position = self.cursor_position.saturating_sub(ch.len_utf8());
        self.text.remove(self.cursor_position);
        self.clear_completions();
    }

    /// Deletes the character after the cursor.
    pub fn delete_char_forward(&mut self) {
        if self.cursor_position < self.text.len() {
            self.text.remove(self.cursor_position);
            self.clear_completions();
        }
    }

    /// Deletes the word before the cursor, shell-style: any trailing run of whitespace
    /// and `/` separators is skipped first, then the word up to the previous whitespace or `/`.
    ///
    /// On a path such as `/downloads/shows/`, this removes `shows/`.
    pub fn delete_word_before_cursor(&mut self) {
        let word_start = self
            .before_cursor()
            .trim_end_matches(is_word_boundary)
            .trim_end_matches(|ch| !is_word_boundary(ch))
            .len();
        self.delete_before_cursor(word_start);
    }

    /// Deletes everything before the cursor.
    pub fn delete_to_start(&mut self) {
        self.delete_before_cursor(0);
    }

    /// Moves the cursor one character to the left.
    pub fn move_cursor_left(&mut self) {
        if let Some(ch) = self.before_cursor().chars().next_back() {
            self.cursor_position = self.cursor_position.saturating_sub(ch.len_utf8());
        }
    }

    /// Moves the cursor one character to the right.
    pub fn move_cursor_right(&mut self) {
        if let Some(ch) = self.after_cursor().chars().next() {
            self.cursor_position = self.cursor_position.saturating_add(ch.len_utf8());
        }
    }

    /// The text before the cursor, or nothing if `cursor_position` is not a
    /// character boundary within the text
    fn before_cursor(&self) -> &str {
        self.text.get(..self.cursor_position).unwrap_or_default()
    }

    /// The text after the cursor, or nothing if `cursor_position` is not a
    /// character boundary within the text
    fn after_cursor(&self) -> &str {
        self.text.get(self.cursor_position..).unwrap_or_default()
    }

    pub const fn move_cursor_to_start(&mut self) {
        self.cursor_position = 0;
    }

    pub const fn move_cursor_to_end(&mut self) {
        self.cursor_position = self.text.len();
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
            self.next_completion();
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
        if let Some(next) = self
            .completion_idx
            .saturating_add(1)
            .checked_rem(self.completions.len())
        {
            self.completion_idx = next;
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

    /// Deletes from byte offset `start`, which must be a char boundary, up to the cursor.
    fn delete_before_cursor(&mut self, start: usize) {
        if start == self.cursor_position {
            return;
        }
        self.text.replace_range(start..self.cursor_position, "");
        self.cursor_position = start;
        self.clear_completions();
    }
}

const fn is_word_boundary(ch: char) -> bool {
    ch.is_whitespace() || ch == '/'
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
    let mut entries = match fs::read_dir(base_path).await {
        Ok(entries) => entries,
        // A base that is missing, unreadable, or not a directory has nothing to suggest.
        Err(error)
            if matches!(
                error.kind(),
                ErrorKind::NotFound | ErrorKind::PermissionDenied | ErrorKind::NotADirectory
            ) =>
        {
            return Ok(Vec::new());
        }
        Err(error) => return Err(error.into()),
    };
    let mut matches = Vec::new();

    while let Some(entry) = entries.next_entry().await? {
        let file_name = entry.file_name();
        if !is_suggestable_directory(&entry).await? {
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

async fn is_suggestable_directory(entry: &fs::DirEntry) -> Result<bool> {
    let file_type = entry.file_type().await?;
    if file_type.is_dir() {
        return Ok(true);
    }

    if !file_type.is_symlink() {
        return Ok(false);
    }

    match fs::metadata(entry.path()).await {
        Ok(metadata) => Ok(metadata.is_dir()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_none, assert_ok};
    #[cfg(unix)]
    use std::os::unix::fs::{PermissionsExt, symlink};
    use std::{fs, path::PathBuf};
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

        assert_eq!(input.completions(), Vec::<PathBuf>::new());
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
        assert_eq!(input.completions(), Vec::<PathBuf>::new());
        assert_eq!(input.completion_idx(), None);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn complete_suggests_symlinked_directories() {
        let dir = temp_dir();
        let target = dir.path().join("target");
        create_dir(&target);
        symlink(&target, dir.path().join("linked")).expect("symlink should be created");
        let mut input = InputHandler::new();
        input.set_text(format!("{}/li", dir.path().display()));

        input.complete().await.expect("completion should succeed");

        assert_eq!(input.completions().len(), 1);
        assert_eq!(input.text, format!("{}/linked/", dir.path().display()));
        assert_eq!(input.cursor_position, input.text.len());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn complete_ignores_symlinks_to_files() {
        let dir = temp_dir();
        let target = dir.path().join("target.txt");
        create_file(&target);
        symlink(&target, dir.path().join("linked")).expect("symlink should be created");
        let mut input = InputHandler::new();
        input.set_text(format!("{}/li", dir.path().display()));

        input.complete().await.expect("completion should succeed");

        assert_eq!(input.text, format!("{}/li", dir.path().display()));
        assert_eq!(input.completions(), Vec::<PathBuf>::new());
        assert_eq!(input.completion_idx(), None);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn complete_ignores_broken_symlinks() {
        let dir = temp_dir();
        symlink(dir.path().join("missing"), dir.path().join("linked"))
            .expect("symlink should be created");
        let mut input = InputHandler::new();
        input.set_text(format!("{}/li", dir.path().display()));

        input.complete().await.expect("completion should succeed");

        assert_eq!(input.text, format!("{}/li", dir.path().display()));
        assert_eq!(input.completions(), Vec::<PathBuf>::new());
        assert_eq!(input.completion_idx(), None);
    }

    #[tokio::test]
    async fn complete_with_missing_base_dir_keeps_text() {
        let dir = temp_dir();
        let text = format!("{}/missing/sub", dir.path().display());
        let mut input = InputHandler::new();
        input.set_text(text.clone());

        assert_ok!(input.complete().await);

        assert_eq!(input.text, text);
        assert_eq!(input.completions(), Vec::<PathBuf>::new());
        assert_none!(input.completion_idx());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn complete_with_unreadable_base_dir_keeps_text() {
        let dir = temp_dir();
        let locked = dir.path().join("locked");
        create_dir(&locked);
        assert_ok!(fs::set_permissions(
            &locked,
            fs::Permissions::from_mode(0o000)
        ));
        let readable_anyway = fs::read_dir(&locked).is_ok();
        let text = format!("{}/a", locked.display());
        let mut input = InputHandler::new();
        input.set_text(text.clone());

        let result = input.complete().await;
        assert_ok!(fs::set_permissions(
            &locked,
            fs::Permissions::from_mode(0o755)
        ));

        assert_ok!(result);
        if readable_anyway {
            // Privileges that bypass permission checks leave nothing to test.
            return;
        }
        assert_eq!(input.text, text);
        assert_eq!(input.completions(), Vec::<PathBuf>::new());
    }

    #[test]
    fn move_cursor_left_and_right_step_over_multibyte_chars() {
        let mut input = input_with("aé😀");

        for expected in [3, 1, 0, 0] {
            input.move_cursor_left();
            assert_eq!(input.cursor_position, expected);
        }
        for expected in [1, 3, 7, 7] {
            input.move_cursor_right();
            assert_eq!(input.cursor_position, expected);
        }
    }

    #[test]
    fn move_cursor_to_start_and_end() {
        let mut input = input_with("日本語");

        input.move_cursor_to_start();
        assert_eq!(input.cursor_position, 0);

        input.move_cursor_to_end();
        assert_eq!(input.cursor_position, "日本語".len());
    }

    #[test]
    fn insert_char_at_moved_cursor() {
        let mut input = input_with("aé");
        input.move_cursor_left();

        input.insert_char('😀');

        assert_eq!(input.text, "a😀é");
        assert_eq!(input.cursor_position, "a😀".len());
    }

    #[test]
    fn delete_char_forward_removes_char_after_cursor() {
        let mut input = input_with("éa");
        input.move_cursor_to_start();

        input.delete_char_forward();
        assert_eq!(input.text, "a");
        assert_eq!(input.cursor_position, 0);

        input.move_cursor_to_end();
        input.delete_char_forward();
        assert_eq!(input.text, "a");
        assert_eq!(input.cursor_position, 1);
    }

    #[test]
    fn delete_word_before_cursor_at_end() {
        for (text, expected) in [
            ("/downloads/shows/", "/downloads/"),
            ("/downloads/shows", "/downloads/"),
            ("/downloads//", "/"),
            ("foo bar  ", "foo "),
            ("日本/映画/", "日本/"),
            ("naïve café", "naïve "),
            ("/", ""),
            ("foo /", ""),
            ("a/ /", ""),
            ("/ ", ""),
            ("bar foo/ /", "bar "),
            ("word", ""),
            ("", ""),
        ] {
            let mut input = input_with(text);

            input.delete_word_before_cursor();

            assert_eq!(input.text, expected, "deleting word from {text:?}");
            assert_eq!(input.cursor_position, expected.len());
        }
    }

    #[test]
    fn delete_word_before_cursor_keeps_text_after_cursor() {
        let mut input = input_with("foo bär baz");
        for _ in 0..4 {
            input.move_cursor_left();
        }

        input.delete_word_before_cursor();

        assert_eq!(input.text, "foo  baz");
        assert_eq!(input.cursor_position, "foo ".len());
    }

    #[test]
    fn delete_word_before_cursor_at_start_is_noop() {
        let mut input = input_with("foo");
        input.move_cursor_to_start();

        input.delete_word_before_cursor();

        assert_eq!(input.text, "foo");
        assert_eq!(input.cursor_position, 0);
    }

    #[test]
    fn delete_to_start_keeps_text_after_cursor() {
        let mut input = input_with("日本語 text");
        for _ in 0..5 {
            input.move_cursor_left();
        }

        input.delete_to_start();
        assert_eq!(input.text, " text");
        assert_eq!(input.cursor_position, 0);

        input.delete_to_start();
        assert_eq!(input.text, " text");
    }

    #[tokio::test]
    async fn cursor_edits_clear_stale_completions() {
        let dir = temp_dir();
        create_dir(dir.path().join("alpha"));
        let edits: [fn(&mut InputHandler); 3] = [
            |input| {
                input.move_cursor_to_start();
                input.delete_char_forward();
            },
            InputHandler::delete_word_before_cursor,
            InputHandler::delete_to_start,
        ];

        for edit in edits {
            let mut input = input_with(&format!("{}/a", dir.path().display()));
            assert_ok!(input.complete().await);
            assert_eq!(input.completions().len(), 1);

            edit(&mut input);

            assert_eq!(input.completions(), Vec::<PathBuf>::new());
            assert_none!(input.completion_idx());
        }
    }

    fn input_with(text: &str) -> InputHandler {
        let mut input = InputHandler::new();
        input.set_text(text.to_owned());
        input
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
