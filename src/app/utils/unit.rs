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

#[derive(Debug)]
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

impl From<u64> for Unit {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl TryFrom<i64> for Unit {
    type Error = UnitError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        if value < 0 {
            return Err(UnitError::NegativeValue { value });
        }
        Ok(Self(value as u64))
    }
}
