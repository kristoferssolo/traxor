pub struct FileSize(pub i64);

impl FileSize {
    pub fn to_b(&self) -> String {
        format!("{} b", self.0)
    }

    pub fn to_kb(&self) -> String {
        format!("{:.2} KB", self.0 as f64 / 1e3)
    }

    pub fn to_mb(&self) -> String {
        format!("{:.2} MB", self.0 as f64 / 1e6)
    }

    pub fn to_gb(&self) -> String {
        format!("{:.2} GB", self.0 as f64 / 1e9)
    }

    pub fn to_tb(&self) -> String {
        format!("{:.2} TB", self.0 as f64 / 1e12)
    }
}

impl ToString for FileSize {
    fn to_string(&self) -> String {
        if self.0 == 0 {
            return "0".to_string();
        }
        match self.0 as f64 {
            b if b >= 1e12 => self.to_tb(),
            b if b >= 1e9 => self.to_gb(),
            b if b >= 1e6 => self.to_mb(),
            b if b >= 1e3 => self.to_kb(),
            _ => self.to_b(),
        }
    }
}
