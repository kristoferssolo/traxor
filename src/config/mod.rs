pub mod color;
pub mod keybinds;
pub mod log;
pub mod tabs;
pub mod time;

use color::ColorConfig;
use color_eyre::{
    Result,
    eyre::{Context, ContextCompat, eyre},
};
use keybinds::KeybindsConfig;
use log::LogConfig;
use serde::{Deserialize, Serialize};
use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
};
use tabs::TabConfig;
use time::TimeConfig;
use toml::Value;
use tracing::{debug, info};

/// Embedded default configuration - single source of truth for all defaults.
const DEFAULT_CONFIG: &str = include_str!("../../config/default.toml");

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub keybinds: KeybindsConfig,
    pub colors: ColorConfig,
    pub log: LogConfig,
    #[serde(default)]
    pub time: TimeConfig,
    #[serde(default)]
    pub tabs: Vec<TabConfig>,
}

impl Config {
    /// Load configuration with fallback to embedded defaults.
    ///
    /// Merge order:
    /// 1. Embedded defaults (config/default.toml - compiled in)
    /// 2. System-wide config (`/etc/xdg/traxor/config.toml`)
    /// 3. User config (`~/.config/traxor/config.toml`)
    ///
    /// # Errors
    ///
    /// Returns an error if the TOML in any config file is invalid.
    #[tracing::instrument(name = "Loading configuration")]
    pub fn load() -> Result<Self> {
        let mut config: Value =
            toml::from_str(DEFAULT_CONFIG).context("Failed to parse embedded default config")?;

        let candidates = [
            ("system-wide", PathBuf::from("/etc/xdg/traxor/config.toml")),
            ("user-specific", get_config_path()?),
        ];

        for (label, path) in &candidates {
            if let Some(user_config) = load_toml_file(label, path)? {
                deep_merge(&mut config, user_config);
            }
        }

        let config = Self::from_value(config)?;

        debug!("Configuration loaded successfully.");
        Ok(config)
    }

    fn from_value(value: Value) -> Result<Self> {
        let config: Self = value
            .try_into()
            .context("Failed to deserialize merged config")?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<()> {
        self.time.validate().map_err(|err| eyre!(err))?;
        for (idx, tab) in self.tabs.iter().enumerate() {
            tab.validate(idx).map_err(|err| eyre!(err))?;
        }
        Ok(())
    }
}

fn get_config_path() -> Result<PathBuf> {
    let config_dir =
        dirs::config_dir().context("Could not determine user configuration directory")?;
    Ok(config_dir.join("traxor").join("config.toml"))
}

fn load_toml_file(label: &str, path: &Path) -> Result<Option<Value>> {
    if !path.exists() {
        debug!("{} config not found at: {:?}", label, path);
        return Ok(None);
    }

    info!("Loading {} config from: {:?}", label, path);
    let content = read_to_string(path)
        .with_context(|| format!("Failed to read {label} config: {}", path.display()))?;

    let value: Value =
        toml::from_str(&content).with_context(|| format!("Failed to parse {label} config TOML"))?;

    info!("Successfully loaded {} config.", label);
    Ok(Some(value))
}

/// Deep merge two TOML values. User values override defaults.
/// For tables, merge recursively. For arrays (like tabs), replace entirely.
fn deep_merge(base: &mut Value, other: Value) {
    match (base, other) {
        (Value::Table(base_table), Value::Table(other_table)) => {
            for (key, other_value) in other_table {
                match base_table.get_mut(&key) {
                    Some(base_value) => deep_merge(base_value, other_value),
                    None => {
                        base_table.insert(key, other_value);
                    }
                }
            }
        }
        (base, other) => *base = other,
    }
}

#[cfg(test)]
mod tests {
    use super::Config;
    use color_eyre::Result;
    use toml::Value;

    fn config_value(input: &str) -> Result<Value> {
        toml::from_str(input).map_err(Into::into)
    }

    #[test]
    fn rejects_invalid_tab_column() -> Result<()> {
        let value = config_value(
            r##"
            [keybinds]
            prev_torrent = "k"
            next_torrent = "j"
            prev_tab = "h"
            next_tab = "l"
            switch_tab_1 = "1"
            switch_tab_2 = "2"
            switch_tab_3 = "3"
            switch_tab_4 = "4"
            switch_tab_5 = "5"
            switch_tab_6 = "6"
            switch_tab_7 = "7"
            switch_tab_8 = "8"
            switch_tab_9 = "9"
            switch_tab_10 = "0"
            toggle_torrent = "enter"
            toggle_all = "a"
            select = " "
            move_torrent = "m"
            rename_torrent = "r"
            delete = "d"
            delete_force = "D"
            filter = "/"
            clear_filter = "escape"
            toggle_help = "?"
            quit = "q"

            [colors]
            highlight_background = "#3a3a5a"
            highlight_foreground = "white"
            selected_background = "#245d5a"
            selected_foreground = "white"
            header_foreground = "yellow"
            info_foreground = "blue"
            status_downloading = "cyan"
            status_seeding = "white"
            status_stopped = "dark_gray"
            status_verifying = "yellow"
            status_queued = "light_blue"

            [log]
            traxor = "info"
            ratatui = "warn"
            transmission_rpc = "warn"

            [time]
            date_format = "%Y-%m-%d %H:%M"
            eta_format = "compact"

            [[tabs]]
            name = "Broken"
            columns = ["name", "bogus"]
            "##,
        )?;

        let err = match Config::from_value(value) {
            Ok(_) => panic!("invalid tab column should fail validation"),
            Err(err) => err.to_string(),
        };
        assert!(err.contains("invalid columns"));
        assert!(err.contains("bogus"));
        Ok(())
    }

    #[test]
    fn rejects_invalid_tab_status() -> Result<()> {
        let value = config_value(
            r##"
            [keybinds]
            prev_torrent = "k"
            next_torrent = "j"
            prev_tab = "h"
            next_tab = "l"
            switch_tab_1 = "1"
            switch_tab_2 = "2"
            switch_tab_3 = "3"
            switch_tab_4 = "4"
            switch_tab_5 = "5"
            switch_tab_6 = "6"
            switch_tab_7 = "7"
            switch_tab_8 = "8"
            switch_tab_9 = "9"
            switch_tab_10 = "0"
            toggle_torrent = "enter"
            toggle_all = "a"
            select = " "
            move_torrent = "m"
            rename_torrent = "r"
            delete = "d"
            delete_force = "D"
            filter = "/"
            clear_filter = "escape"
            toggle_help = "?"
            quit = "q"

            [colors]
            highlight_background = "#3a3a5a"
            highlight_foreground = "white"
            selected_background = "#245d5a"
            selected_foreground = "white"
            header_foreground = "yellow"
            info_foreground = "blue"
            status_downloading = "cyan"
            status_seeding = "white"
            status_stopped = "dark_gray"
            status_verifying = "yellow"
            status_queued = "light_blue"

            [log]
            traxor = "info"
            ratatui = "warn"
            transmission_rpc = "warn"

            [time]
            date_format = "%Y-%m-%d %H:%M"
            eta_format = "compact"

            [[tabs]]
            name = "Broken"
            columns = ["name"]
            statuses = ["Flying"]
            "##,
        )?;

        let err = match Config::from_value(value) {
            Ok(_) => panic!("invalid tab status should fail validation"),
            Err(err) => err.to_string(),
        };
        assert!(err.contains("invalid statuses"));
        assert!(err.contains("Flying"));
        Ok(())
    }

    #[test]
    fn rejects_invalid_eta_format() -> Result<()> {
        let value = config_value(
            r##"
            [keybinds]
            prev_torrent = "k"
            next_torrent = "j"
            prev_tab = "h"
            next_tab = "l"
            switch_tab_1 = "1"
            switch_tab_2 = "2"
            switch_tab_3 = "3"
            switch_tab_4 = "4"
            switch_tab_5 = "5"
            switch_tab_6 = "6"
            switch_tab_7 = "7"
            switch_tab_8 = "8"
            switch_tab_9 = "9"
            switch_tab_10 = "0"
            toggle_torrent = "enter"
            toggle_all = "a"
            select = " "
            move_torrent = "m"
            rename_torrent = "r"
            delete = "d"
            delete_force = "D"
            filter = "/"
            clear_filter = "escape"
            toggle_help = "?"
            quit = "q"

            [colors]
            highlight_background = "#3a3a5a"
            highlight_foreground = "white"
            selected_background = "#245d5a"
            selected_foreground = "white"
            header_foreground = "yellow"
            info_foreground = "blue"
            status_downloading = "cyan"
            status_seeding = "white"
            status_stopped = "dark_gray"
            status_verifying = "yellow"
            status_queued = "light_blue"

            [log]
            traxor = "info"
            ratatui = "warn"
            transmission_rpc = "warn"

            [time]
            date_format = "%Y-%m-%d %H:%M"
            eta_format = "verbose"
            "##,
        )?;

        let err = match Config::from_value(value) {
            Ok(_) => panic!("invalid eta format should fail validation"),
            Err(err) => err.to_string(),
        };
        assert!(err.contains("time.eta_format"));
        Ok(())
    }

    #[test]
    fn accepts_valid_config() -> Result<()> {
        let value: Value = toml::from_str(include_str!("../../config/default.toml"))?;
        Config::from_value(value)?;
        Ok(())
    }
}
