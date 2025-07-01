use std::fmt;

pub struct NetSpeed(i64);

impl From<i64> for NetSpeed {
    fn from(bytes_per_second: i64) -> Self {
        NetSpeed(bytes_per_second)
    }
}

impl fmt::Display for NetSpeed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bytes_per_second = self.0;
        if bytes_per_second < 1024 {
            write!(f, "{} B/s", bytes_per_second)
        } else if bytes_per_second < 1024 * 1024 {
            write!(f, "{:.2} KB/s", bytes_per_second as f64 / 1024.0)
        } else if bytes_per_second < 1024 * 1024 * 1024 {
            write!(f, "{:.2} MB/s", bytes_per_second as f64 / (1024.0 * 1024.0))
        } else {
            write!(f, "{:.2} GB/s", bytes_per_second as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }
}