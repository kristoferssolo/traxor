use filecaster::FromFile;

#[derive(Debug, Clone, FromFile)]
pub struct LogConfig {
    #[from_file(default = "warn")]
    pub traxor: String,
    #[from_file(default = "warn")]
    pub ratatui: String,
    #[from_file(default = "warn")]
    pub transmission_rpc: String,
}
