use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ColorConfig {
    pub highlight_background: String,
    pub highlight_foreground: String,
    pub selected_background: String,
    pub selected_foreground: String,
    pub header_foreground: String,
    pub info_foreground: String,
    pub status_downloading: String,
    pub status_seeding: String,
    pub status_stopped: String,
    pub status_verifying: String,
    pub status_queued: String,
    pub progress_downloading: String,
    pub progress_seeding: String,
    pub progress_stopped: String,
    pub progress_verifying: String,
    pub progress_queued: String,
}
