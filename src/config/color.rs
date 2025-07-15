use filecaster::FromFile;

#[derive(Debug, Clone, FromFile)]
pub struct ColorConfig {
    #[from_file(default = "magenta")]
    pub highlight_background: String,
    #[from_file(default = "black")]
    pub highlight_foreground: String,
    #[from_file(default = "yellow")]
    pub header_foreground: String,
    #[from_file(default = "blue")]
    pub info_foreground: String,
}
