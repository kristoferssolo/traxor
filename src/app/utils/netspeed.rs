use super::unit::{Unit, UnitDisplay};
use crate::impl_unit_newtype;
use std::fmt::Display;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct NetSpeed(Unit);
impl_unit_newtype!(NetSpeed);

impl NetSpeed {
    #[inline]
    #[must_use]
    pub const fn new(bytes_per_second: u64) -> Self {
        Self(Unit::from_raw(bytes_per_second))
    }
}

impl Display for NetSpeed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const UNITS: &[&str] = &["B/s", "KB/s", "MB/s", "GB/s", "TB/s", "PB/s"];
        write!(f, "{}", UnitDisplay::new(&self.0, UNITS))
    }
}
