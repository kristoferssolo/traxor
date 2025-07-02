use std::fmt::Display;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum NetSpeedError {
    #[error("Network speed cannot be negative: {value}")]
    NegativeSpeed { value: i64 },
    #[error("Network speed value is too large: {value}")]
    ValueTooLarge { value: f64 },
    #[error("Network speed value is invalid: {reason}")]
    InvalidValue { reason: String },
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct NetSpeed(u64);

impl NetSpeed {
    pub const fn new(bytes_per_second: u64) -> Self {
        Self(bytes_per_second)
    }

    pub const fn bytes_per_second(&self) -> u64 {
        self.0
    }

    pub const fn kilobytes_per_second(kb: u64) -> Self {
        Self(kb * 1024)
    }

    pub const fn megabytes_per_second(mb: u64) -> Self {
        Self(mb * 1024 * 1024)
    }

    pub const fn gigabytes_per_second(gb: u64) -> Self {
        Self(gb * 1024 * 1024 * 1024)
    }

    pub const fn from_bits_per_second(bps: u64) -> Self {
        NetSpeed(bps / 8)
    }

    pub const fn to_bits_per_second(&self) -> u64 {
        self.0 * 8
    }

    pub const fn from_kilobits_per_second(kbps: u64) -> Self {
        NetSpeed(kbps * 1000 / 8)
    }

    pub const fn from_megabits_per_second(mbps: u64) -> Self {
        NetSpeed(mbps * 1_000_000 / 8)
    }

    pub const fn from_gigabits_per_second(gbps: u64) -> Self {
        NetSpeed(gbps * 1_000_000_000 / 8)
    }
}

macro_rules! impl_from_unsigned {
    ($($t:ty),*) => {
        $(
            impl From<$t> for NetSpeed {
                fn from(value: $t) -> Self {
                    Self(value as u64)
                }
            }
        )*
    };
}

macro_rules! impl_try_from_signed {
    ($($t:ty),*) => {
        $(
            impl TryFrom<$t> for NetSpeed {
                type Error = NetSpeedError;

                fn try_from(value: $t) -> Result<Self, Self::Error> {
                    if value < 0 {
                        Err(NetSpeedError::NegativeSpeed { value: value as i64 })
                    } else {
                        Ok(Self(value as u64))
                    }
                }
            }
        )*
    };
}

impl_from_unsigned!(u8, u16, u32, u64, usize);
impl_try_from_signed!(i8, i16, i32, i64, isize);

impl Display for NetSpeed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const UNITS: &[&str] = &["B/s", "KB/s", "MB/s", "GB/s", "TB/s", "PB/s"];
        const THREASHOLD: f64 = 1024.0;

        let bytes_per_second = self.0 as f64;

        if bytes_per_second < THREASHOLD {
            return write!(f, "{} {}", self.0, UNITS[0]);
        }

        let mut size = bytes_per_second;
        let mut unit_index = 0;

        while size >= THREASHOLD && unit_index < UNITS.len() - 1 {
            size /= THREASHOLD;
            unit_index += 1;
        }
        if unit_index == 0 {
            return write!(f, "{} {}", size as u64, UNITS[unit_index]);
        }
        write!(f, "{:.2} {}", size, UNITS[unit_index])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_net_speed_display() {
        assert_eq!(NetSpeed::new(512).to_string(), "512 B/s");
        assert_eq!(NetSpeed::new(1536).to_string(), "1.50 KB/s");
        assert_eq!(NetSpeed::new(1048576).to_string(), "1.00 MB/s");
        assert_eq!(NetSpeed::new(1073741824).to_string(), "1.00 GB/s");
        assert_eq!(NetSpeed::new(1099511627776).to_string(), "1.00 TB/s");
    }

    #[test]
    fn test_bits_conversion() {
        let speed = NetSpeed::from_bits_per_second(8000);
        assert_eq!(speed.bytes_per_second(), 1000);
        assert_eq!(speed.to_bits_per_second(), 8000);
    }

    #[test]
    fn test_network_units() {
        // 1 Mbps = 125,000 bytes per second
        let speed = NetSpeed::from_megabits_per_second(1);
        assert_eq!(speed.bytes_per_second(), 125_000);

        // 1 Gbps = 125,000,000 bytes per second
        let speed = NetSpeed::from_gigabits_per_second(1);
        assert_eq!(speed.bytes_per_second(), 125_000_000);
    }

    #[test]
    fn test_try_from_i64() {
        assert_eq!(
            NetSpeed::try_from(1000i64).unwrap().bytes_per_second(),
            1000
        );
        assert_eq!(
            NetSpeed::try_from(-100i64),
            Err(NetSpeedError::NegativeSpeed { value: -100 })
        );
    }
}
