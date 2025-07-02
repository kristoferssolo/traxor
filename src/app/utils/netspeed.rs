use super::unit::{Unit, UnitError};
use crate::app::utils::unit::UnitDisplay;
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

macro_rules! impl_from_unsigned {
    ($type:ty, $($t:ty), *) => {
        $(
            impl From<$t> for $type {
                fn from(value: $t) -> Self {
                    Self(Unit::from(value))
                }
            }
        )*
    };
}

macro_rules! impl_try_from_signed {
    ($type:ty, $error:ty, $($t:ty), *) => {
        $(
            impl TryFrom<$t> for $type {
                type Error = $error;

                fn try_from(value: $t) -> Result<Self, Self::Error> {
                    if value < 0 {
                        return Err(UnitError::NegativeValue { value: value as i64 });
                    }
                    Ok(Self(Unit::try_from(value)?))
                }
            }
        )*
    };
}

impl_from_unsigned!(NetSpeed, u8, u16, u32, u64, usize);
impl_try_from_signed!(NetSpeed, UnitError, i8, i16, i32, i64, isize);
