use super::unit::{Unit, UnitDisplay, UnitError};
use std::fmt::Display;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct NetSpeed(Unit);

impl NetSpeed {
    pub fn new(bytes_per_second: u64) -> Self {
        Self(Unit::new(bytes_per_second))
    }
}

impl Display for NetSpeed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const UNITS: &[&str] = &["B/s", "KB/s", "MB/s", "GB/s", "TB/s", "PB/s"];
        write!(f, "{}", UnitDisplay::new(&self.0, UNITS))
    }
}

impl From<u64> for NetSpeed {
    fn from(value: u64) -> Self {
        Self(Unit::from(value))
    }
}

impl TryFrom<i64> for NetSpeed {
    type Error = UnitError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Ok(Self(Unit::try_from(value)?))
    }
}
