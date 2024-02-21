pub struct NetSpeed(pub i64);

impl NetSpeed {
    pub fn to_bps(&self) -> String {
        format!("{} bps", self.0)
    }

    pub fn to_kbps(&self) -> String {
        format!("{:.2} kbps", self.0 as f64 / 1e3)
    }

    pub fn to_mbps(&self) -> String {
        format!("{:.2} mbps", self.0 as f64 / 1e6)
    }

    pub fn to_gbps(&self) -> String {
        format!("{:.2} gbps", self.0 as f64 / 1e9)
    }
}

impl ToString for NetSpeed {
    fn to_string(&self) -> String {
        if self.0 == 0 {
            return "0".to_string();
        }
        match self.0 as f64 {
            b if b >= 1e9 => self.to_gbps(),
            b if b >= 1e6 => self.to_mbps(),
            b if b >= 1e3 => self.to_kbps(),
            _ => self.to_bps(),
        }
    }
}
