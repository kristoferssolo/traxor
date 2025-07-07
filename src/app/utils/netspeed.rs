use super::unit::{Unit, UnitDisplay};
use derive_macro::UnitConversions;
use std::fmt::Display;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Default, UnitConversions)]
#[error(UnitError)]
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
