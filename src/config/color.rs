use derive_macro::FromFile;
use merge::{Merge, option::overwrite_none};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Merge)]
pub struct ColorConfigFile {
    #[merge(strategy = overwrite_none)]
    pub highlight_background: Option<String>,
    #[merge(strategy = overwrite_none)]
    pub highlight_foreground: Option<String>,
    #[merge(strategy = overwrite_none)]
    pub header_foreground: Option<String>,
    #[merge(strategy = overwrite_none)]
    pub info_foreground: Option<String>,
}

#[derive(Debug, Clone, FromFile)]
pub struct ColorConfig {
    pub highlight_background: String,
    pub highlight_foreground: String,
    pub header_foreground: String,
    pub info_foreground: String,
}

impl Default for ColorConfigFile {
    fn default() -> Self {
        Self {
            highlight_background: Some("magenta".to_string()),
            highlight_foreground: Some("black".to_string()),
            header_foreground: Some("yellow".to_string()),
            info_foreground: Some("blue".to_string()),
        }
    }
}
