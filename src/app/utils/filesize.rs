use super::unit::{Unit, UnitDisplay};
use crate::impl_unit_newtype;
use std::fmt::Display;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct FileSize(Unit);
impl_unit_newtype!(FileSize);

impl FileSize {
    #[inline]
    #[must_use]
    pub const fn new(bytes: u64) -> Self {
        Self(Unit::from_raw(bytes))
    }
}

impl Display for FileSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
        write!(f, "{}", UnitDisplay::new(&self.0, UNITS))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let file_size = FileSize::new(1024);
        assert_eq!(file_size.0.as_raw(), 1024);
    }

    #[test]
    fn test_display() {
        let file_size = FileSize::new(1024);
        assert_eq!(file_size.to_string(), "1.00 KB");

        let file_size = FileSize::new(1024 * 1024);
        assert_eq!(file_size.to_string(), "1.00 MB");

        let file_size = FileSize::new(1024 * 1024 * 1024);
        assert_eq!(file_size.to_string(), "1.00 GB");
    }
}

