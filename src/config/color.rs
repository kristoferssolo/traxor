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

    // Status colors
    #[from_file(default = "cyan")]
    pub status_downloading: String,
    #[from_file(default = "white")]
    pub status_seeding: String,
    #[from_file(default = "dark_gray")]
    pub status_stopped: String,
    #[from_file(default = "yellow")]
    pub status_verifying: String,
    #[from_file(default = "light_blue")]
    pub status_queued: String,
}
