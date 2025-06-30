use std::fmt;

const KB: f64 = 1e3;
const MB: f64 = 1e6;
const GB: f64 = 1e9;
const TB: f64 = 1e12;

pub struct FileSize(pub i64);

impl fmt::Display for FileSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 == 0 {
            return write!(f, "0");
        }
        let size = self.0 as f64;
        let (value, unit) = match size {
            s if s >= TB => (s / TB, "TB"),
            s if s >= GB => (s / GB, "GB"),
            s if s >= MB => (s / MB, "MB"),
            s if s >= KB => (s / KB, "KB"),
            _ => (size, "B"),
        };
        write!(f, "{:.2} {}", value, unit)
    }
}

impl From<i64> for FileSize {
    fn from(size: i64) -> Self {
        FileSize(size)
    }
}
