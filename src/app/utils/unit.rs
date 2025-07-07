use std::fmt::{Debug, Display};

mod sealed {
    pub trait Sealed {}

    macro_rules! impl_sealed {
        ($($t:ty),*) => {
            $(
                impl Sealed for $t {}
                impl Sealed for Option<$t> {}
            )*
        };
    }

    impl_sealed!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);
}

pub trait IntoU64: sealed::Sealed {
    fn into_u64(self) -> u64;
}

macro_rules! impl_into_u64 {
    ($($t:ty),*) => {$(
        impl IntoU64 for $t {
            fn into_u64(self) -> u64 {
                self.try_into().unwrap_or(0)
            }
        }

        impl IntoU64 for Option<$t> {
            fn into_u64(self) -> u64 {
                self.unwrap_or(0).try_into().unwrap_or(0)
            }
        }
    )*};
}

impl_into_u64!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Unit(u64);

impl<U> From<U> for Unit
where
    U: IntoU64,
{
    fn from(value: U) -> Self {
        Self(value.into_u64())
    }
}

impl Unit {
    pub const fn from_raw(value: u64) -> Self {
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
        const THRESHOLD: f64 = 1024.0;

        let value = self.unit.0 as f64;

        if value < THRESHOLD {
            return write!(f, "{} {}", self.unit.0, self.units[0]);
        }

        let mut size = value;
        let mut unit_index = 0;

        while size >= THRESHOLD && unit_index < self.units.len() - 1 {
            size /= THRESHOLD;
            unit_index += 1;
        }
        write!(f, "{:.2} {}", size, self.units[unit_index])
    }
}

#[macro_export]
macro_rules! impl_unit_newtype {
    ($wrapper:ident) => {
        impl From<Unit> for $wrapper {
            fn from(unit: $crate::app::utils::unit::Unit) -> Self {
                Self(unit)
            }
        }

        impl<T> From<T> for $wrapper
        where
            T: $crate::app::utils::unit::IntoU64,
        {
            fn from(value: T) -> Self {
                Self($crate::app::utils::unit::Unit::from(value))
            }
        }

        impl $wrapper {
            pub fn unit(&self) -> &Unit {
                &self.0
            }
        }
    };
}
