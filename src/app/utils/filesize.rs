use std::fmt::Display;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum FileSizeError {
    #[error("File size cannot be negative: {value}")]
    NegativeSize { value: i64 },
    #[error("File size value is too large: {value}")]
    ValueTooLarge { value: f64 },
    #[error("File size value is invalid: {reason}")]
    InvalidValue { reason: String },
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct FileSize(u64);

impl FileSize {
    pub const fn new(bytes: u64) -> Self {
        Self(bytes)
    }

    pub const fn bytes(&self) -> u64 {
        self.0
    }

    pub const fn kilobytes(kb: u64) -> Self {
        Self(kb * 1024)
    }

    pub const fn megabytes(mb: u64) -> Self {
        Self(mb * 1024 * 1024)
    }

    pub const fn gigabytes(gb: u64) -> Self {
        Self(gb * 1024 * 1024 * 1024)
    }
}

macro_rules! impl_from_unsigned {
    ($($t:ty),*) => {
        $(
            impl From<$t> for FileSize {
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
            impl TryFrom<$t> for FileSize {
                type Error = FileSizeError;

                fn try_from(value: $t) -> Result<Self, Self::Error> {
                    if value < 0 {
                        Err(FileSizeError::NegativeSize { value: value as i64 })
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

impl Display for FileSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
        const THREASHOLD: f64 = 1024.0;

        let bytes = self.0 as f64;

        if bytes < THREASHOLD {
            return write!(f, "{} {}", self.0, UNITS[0]);
        }

        let mut size = bytes;
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
    fn test_file_size_display() {
        assert_eq!(FileSize::new(512).to_string(), "512 B");
        assert_eq!(FileSize::new(1536).to_string(), "1.50 KB");
        assert_eq!(FileSize::new(1048576).to_string(), "1.00 MB");
        assert_eq!(FileSize::new(1073741824).to_string(), "1.00 GB");
        assert_eq!(FileSize::new(1099511627776).to_string(), "1.00 TB");
    }
}
