use std::fmt::Display;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum UnitError {
    #[error("Value cannot be negative: {value}")]
    NegativeValue { value: i64 },
    #[error("Value is too large: {value}")]
    ValueTooLarge { value: f64 },
    #[error("Value is invalid: {reason}")]
    InvalidValue { reason: String },
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Unit(u64);

impl Unit {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(&self) -> u64 {
        self.0
    }
}

pub struct UnitDisplay<'a> {
    unit: &'a Unit,
    units: &'a [&'a str],
}

impl<'a> UnitDisplay<'a> {
    pub fn new(unit: &'a Unit, units: &'a [&'a str]) -> Self {
        Self { unit, units }
    }
}

impl<'a> Display for UnitDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const THREASHOLD: f64 = 1024.0;

        let value = self.unit.0 as f64;

        if value < THREASHOLD {
            return write!(f, "{} {}", self.unit.0, self.units[0]);
        }

        let mut size = value;
        let mut unit_index = 0;

        while size >= THREASHOLD && unit_index < self.units.len() - 1 {
            size /= THREASHOLD;
            unit_index += 1;
        }
        write!(f, "{:.2} {}", size, self.units[unit_index])
    }
}

macro_rules! impl_from_unsigned {
    ($type:ty, $($t:ty), *) => {
        $(
            impl From<$t> for $type {
                fn from(value: $t) -> Self {
                    Self(value as u64)
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
                    Ok(Self(value as u64))
                }
            }
        )*
    };
}

impl_from_unsigned!(Unit, u8, u16, u32, u64, usize);
impl_try_from_signed!(Unit, UnitError, i8, i16, i32, i64, isize);
