use super::unit::{Unit, UnitDisplay};
use derive_macro::UnitConversions;
use std::fmt::Display;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Default, UnitConversions)]
#[error(UnitError)]
pub struct FileSize(Unit);

impl FileSize {
    pub fn new(bytes: u64) -> Self {
        Self(Unit::new(bytes))
    }
}

impl Display for FileSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
        write!(f, "{}", UnitDisplay::new(&self.0, UNITS))
    }
}
