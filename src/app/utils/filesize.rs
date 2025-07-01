use std::fmt;

pub struct FileSize(i64);

impl From<i64> for FileSize {
    fn from(bytes: i64) -> Self {
        FileSize(bytes)
    }
}

impl fmt::Display for FileSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bytes = self.0;
        if bytes < 1024 {
            write!(f, "{} B", bytes)
        } else if bytes < 1024 * 1024 {
            write!(f, "{:.2} KB", bytes as f64 / 1024.0)
        } else if bytes < 1024 * 1024 * 1024 {
            write!(f, "{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
        } else {
            write!(f, "{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }
}