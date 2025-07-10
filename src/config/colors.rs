use derive_macro::FromFile;
use merge::{Merge, option::overwrite_none};
use ratatui::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Merge)]
pub struct ColorsConfigFile {
    #[merge(strategy = overwrite_none)]
    pub highlight_background: Option<String>,
    #[merge(strategy = overwrite_none)]
    pub highlight_foreground: Option<String>,
    #[merge(strategy = overwrite_none)]
    pub warning_foreground: Option<String>,
    #[merge(strategy = overwrite_none)]
    pub info_foreground: Option<String>,
    #[merge(strategy = overwrite_none)]
    pub error_foreground: Option<String>,
}

#[derive(Debug, Clone, FromFile)]
pub struct ColorsConfig {
    pub highlight_background: String,
    pub highlight_foreground: String,
    pub warning_foreground: String,
    pub info_foreground: String,
    pub error_foreground: String,
}

impl ColorsConfig {
    pub fn get_color(&self, color_name: &str) -> Color {
        match color_name.to_lowercase().as_str() {
            "black" => Color::Black,
            "blue" => Color::Blue,
            "cyan" => Color::Cyan,
            "darkgray" => Color::DarkGray,
            "gray" => Color::Gray,
            "green" => Color::Green,
            "lightgreen" => Color::LightGreen,
            "lightred" => Color::LightRed,
            "magenta" => Color::Magenta,
            "red" => Color::Red,
            "white" => Color::White,
            "yellow" => Color::Yellow,
            _ => Color::Reset, // Default to reset, if color name is not recognized
        }
    }
}

impl Default for ColorsConfigFile {
    fn default() -> Self {
        Self {
            highlight_background: Some("magenta".to_string()),
            highlight_foreground: Some("black".to_string()),
            warning_foreground: Some("yellow".to_string()),
            info_foreground: Some("blue".to_string()),
            error_foreground: Some("red".to_string()),
        }
    }
}
