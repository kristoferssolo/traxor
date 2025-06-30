use std::fmt;

const KBPS: f64 = 1_000.0;
const MBPS: f64 = 1_000_000.0;
const GBPS: f64 = 1_000_000_000.0;

pub struct NetSpeed(pub i64);

impl fmt::Display for NetSpeed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let speed = self.0 as f64;
        if speed == 0.0 {
            return write!(f, "0 bps");
        }

        let (value, unit) = match speed {
            s if s >= GBPS => (s / GBPS, "Gbps"),
            s if s >= MBPS => (s / MBPS, "Mbps"),
            s if s >= KBPS => (s / KBPS, "kbps"),
            _ => (speed, "bps"),
        };

        if unit == "bps" {
            write!(f, "{:.0} {}", value, unit)
        } else {
            write!(f, "{:.2} {}", value, unit)
        }
    }
}

impl From<i64> for NetSpeed {
    fn from(speed: i64) -> Self {
        NetSpeed(speed)
    }
}
