use super::unit::{Unit, UnitDisplay, UnitError};
use std::fmt::Display;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
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

impl From<u64> for FileSize {
    fn from(value: u64) -> Self {
        Self(Unit::from(value))
    }
}

impl TryFrom<i64> for FileSize {
    type Error = UnitError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Ok(Self(Unit::try_from(value)?))
    }
}
