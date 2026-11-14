pub mod color;
pub mod keybinds;
pub mod log;
pub mod tabs;
pub mod time;

use crate::config::{
    color::ColorConfig,
    keybinds::{KeybindError, KeybindsConfig, Keymap},
    log::LogConfig,
    tabs::TabConfig,
    time::TimeConfig,
};
use color_eyre::{
    Result,
    eyre::{Context, ContextCompat},
};
use serde::{Deserialize, Serialize};
use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
};
use thiserror::Error;
use toml::Value;
use tracing::{debug, info};

/// Embedded default configuration - single source of truth for all defaults.
const DEFAULT_CONFIG: &str = include_str!("../../config/default.toml");

/// Validated configuration.
///
/// Deserialization validates the raw TOML first, so every `Config` has passed
/// validation and carries a [`Keymap`] built from its `keybinds`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(try_from = "RawConfig", into = "RawConfig")]
pub struct Config {
    pub keybinds: KeybindsConfig,
    pub colors: ColorConfig,
    pub log: LogConfig,
    pub time: TimeConfig,
    pub tabs: Vec<TabConfig>,
    keymap: Keymap,
}

/// Configuration as written in TOML, before validation
#[derive(Debug, Clone, Deserialize, Serialize)]
struct RawConfig {
    keybinds: KeybindsConfig,
    colors: ColorConfig,
    log: LogConfig,
    #[serde(default)]
    time: TimeConfig,
    #[serde(default)]
    tabs: Vec<TabConfig>,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("{0}")]
    Invalid(String),
    #[error(transparent)]
    Keybind(#[from] KeybindError),
}

impl TryFrom<RawConfig> for Config {
    type Error = ConfigError;

    fn try_from(raw: RawConfig) -> Result<Self, Self::Error> {
        raw.time.validate().map_err(ConfigError::Invalid)?;
        for (idx, tab) in raw.tabs.iter().enumerate() {
            tab.validate(idx).map_err(ConfigError::Invalid)?;
        }
        let keymap = raw.keybinds.keymap()?;

        Ok(Self {
            keybinds: raw.keybinds,
            colors: raw.colors,
            log: raw.log,
            time: raw.time,
            tabs: raw.tabs,
            keymap,
        })
    }
}

impl From<Config> for RawConfig {
    fn from(config: Config) -> Self {
        Self {
            keybinds: config.keybinds,
            colors: config.colors,
            log: config.log,
            time: config.time,
            tabs: config.tabs,
        }
    }
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

    /// Keymap built from [`Config::keybinds`]
    #[must_use]
    pub const fn keymap(&self) -> &Keymap {
        &self.keymap
    }

    fn from_value(value: Value) -> Result<Self> {
        value
            .try_into()
            .context("Failed to deserialize merged config")
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
    use super::*;
    use crate::app::action::Action;
    use claims::{assert_err, assert_ok, assert_some_eq};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    /// Parses the embedded defaults and merges `overrides` on top, as `Config::load` does
    /// with user config. Arrays such as `tabs` replace the defaults entirely.
    fn defaults_with(overrides: &str) -> Result<Value> {
        let mut value: Value = toml::from_str(DEFAULT_CONFIG)?;
        deep_merge(&mut value, toml::from_str(overrides)?);
        Ok(value)
    }

    #[test]
    fn rejects_invalid_tab_column() -> Result<()> {
        let value = defaults_with(
            r#"
            [[tabs]]
            name = "Broken"
            columns = ["name", "bogus"]
            "#,
        )?;

        let err = format!("{:#}", assert_err!(Config::from_value(value)));
        assert!(err.contains("invalid columns"));
        assert!(err.contains("bogus"));
        Ok(())
    }

    #[test]
    fn rejects_invalid_tab_status() -> Result<()> {
        let value = defaults_with(
            r#"
            [[tabs]]
            name = "Broken"
            columns = ["name"]
            statuses = ["Flying"]
            "#,
        )?;

        let err = format!("{:#}", assert_err!(Config::from_value(value)));
        assert!(err.contains("invalid statuses"));
        assert!(err.contains("Flying"));
        Ok(())
    }

    #[test]
    fn rejects_invalid_eta_format() -> Result<()> {
        let value = defaults_with(
            r#"
            [time]
            eta_format = "verbose"
            "#,
        )?;

        let err = format!("{:#}", assert_err!(Config::from_value(value)));
        assert!(err.contains("time.eta_format"));
        Ok(())
    }

    fn default_with_keybinds(overrides: &str) -> Result<Value> {
        defaults_with(&format!("[keybinds]\n{overrides}"))
    }

    #[test]
    fn rejects_invalid_keybind() -> Result<()> {
        let value = default_with_keybinds(r#"move_torrent = "ctrl+bogus""#)?;

        let err = format!("{:#}", assert_err!(Config::from_value(value)));
        assert!(err.contains("keybinds.move_torrent"));
        assert!(err.contains("ctrl+bogus"));
        Ok(())
    }

    #[test]
    fn rejects_duplicate_keybind() -> Result<()> {
        let value = default_with_keybinds(r#"rename_torrent = "m""#)?;

        let err = format!("{:#}", assert_err!(Config::from_value(value)));
        assert!(err.contains("keybinds.move_torrent"));
        assert!(err.contains("keybinds.rename_torrent"));
        Ok(())
    }

    #[test]
    fn accepts_default_config() -> Result<()> {
        assert_ok!(Config::from_value(defaults_with("")?));
        Ok(())
    }

    #[test]
    fn direct_deserialization_builds_keymap() -> Result<()> {
        let quit = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);

        let config: Config = toml::from_str(DEFAULT_CONFIG)?;
        assert_some_eq!(config.keymap().action(&quit), Action::Quit);

        let round_tripped: Config = toml::from_str(&toml::to_string(&config)?)?;
        assert_some_eq!(round_tripped.keymap().action(&quit), Action::Quit);
        Ok(())
    }

    #[test]
    fn direct_deserialization_rejects_duplicate_keybind() -> Result<()> {
        let input = toml::to_string(&default_with_keybinds(r#"rename_torrent = "m""#)?)?;

        let err = assert_err!(toml::from_str::<Config>(&input)).to_string();
        assert!(err.contains("keybinds.rename_torrent"));
        Ok(())
    }
}
