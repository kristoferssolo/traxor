use crate::merge::Merge;
use derive_macro::Merge;
use ratatui::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Merge)]
pub struct ColorsConfig {
    pub highlight_background: Option<String>,
    pub highlight_foreground: Option<String>,
    pub warning_foreground: Option<String>,
    pub info_foreground: Option<String>,
    pub error_foreground: Option<String>,
}

impl ColorsConfig {
    pub fn get_color(&self, color_name: &Option<String>) -> Color {
        match color_name {
            Some(name) => match name.to_lowercase().as_str() {
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
            },
            None => Color::Reset,
        }
    }
}

impl Default for ColorsConfig {
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
