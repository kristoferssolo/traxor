pub mod color;
pub mod keybinds;
pub mod log;
pub mod rpc;
pub mod tabs;
pub mod time;

use crate::config::{
    color::ColorConfig,
    keybinds::{KeybindError, KeybindsConfig, Keymap},
    log::LogConfig,
    rpc::{RpcConfig, RpcConfigError},
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
    keybinds: KeybindsConfig,
    pub colors: ColorConfig,
    pub log: LogConfig,
    pub rpc: RpcConfig,
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
    rpc: RpcConfig,
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
    #[error(transparent)]
    Rpc(#[from] RpcConfigError),
}

impl TryFrom<RawConfig> for Config {
    type Error = ConfigError;

    fn try_from(raw: RawConfig) -> Result<Self, Self::Error> {
        raw.rpc.validate()?;
        raw.time.validate().map_err(ConfigError::Invalid)?;
        for (idx, tab) in raw.tabs.iter().enumerate() {
            tab.validate(idx).map_err(ConfigError::Invalid)?;
        }
        let keymap = raw.keybinds.keymap()?;

        Ok(Self {
            keybinds: raw.keybinds,
            colors: raw.colors,
            log: raw.log,
            rpc: raw.rpc,
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
            rpc: config.rpc,
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

    /// Keybind strings as configured, matching [`Config::keymap`].
    ///
    /// Read-only so the strings cannot drift from the keymap built from them.
    #[must_use]
    pub const fn keybinds(&self) -> &KeybindsConfig {
        &self.keybinds
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
    use claims::{assert_err, assert_ok, assert_some, assert_some_eq};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    /// Parses the embedded defaults and merges `overrides` on top, as `Config::load` does
    /// with user config. Arrays such as `tabs` replace the defaults entirely.
    fn defaults_with(overrides: &str) -> Result<Value> {
        let mut value: Value = toml::from_str(DEFAULT_CONFIG)?;
        deep_merge(&mut value, toml::from_str(overrides)?);
        Ok(value)
    }

    #[test]
    fn rejects_invalid_tab_column() {
        let value = assert_ok!(defaults_with(
            r#"
            [[tabs]]
            name = "Broken"
            columns = ["name", "bogus"]
            "#,
        ));

        let err = format!("{:#}", assert_err!(Config::from_value(value)));
        assert!(err.contains("invalid columns"));
        assert!(err.contains("bogus"));
    }

    #[test]
    fn rejects_invalid_tab_status() {
        let value = assert_ok!(defaults_with(
            r#"
            [[tabs]]
            name = "Broken"
            columns = ["name"]
            statuses = ["Flying"]
            "#,
        ));

        let err = format!("{:#}", assert_err!(Config::from_value(value)));
        assert!(err.contains("invalid statuses"));
        assert!(err.contains("Flying"));
    }

    #[test]
    fn rejects_invalid_eta_format() {
        let value = assert_ok!(defaults_with(
            r#"
            [time]
            eta_format = "verbose"
            "#,
        ));

        let err = format!("{:#}", assert_err!(Config::from_value(value)));
        assert!(err.contains("time.eta_format"));
    }

    fn default_with_keybinds(overrides: &str) -> Result<Value> {
        defaults_with(&format!("[keybinds]\n{overrides}"))
    }

    #[test]
    fn rejects_invalid_keybind() {
        let value = assert_ok!(default_with_keybinds(r#"move_torrent = "ctrl+bogus""#));

        let err = format!("{:#}", assert_err!(Config::from_value(value)));
        assert!(err.contains("keybinds.move_torrent"));
        assert!(err.contains("ctrl+bogus"));
    }

    #[test]
    fn rejects_duplicate_keybind() {
        let value = assert_ok!(default_with_keybinds(r#"rename_torrent = "m""#));

        let err = format!("{:#}", assert_err!(Config::from_value(value)));
        assert!(err.contains("keybinds.move_torrent"));
        assert!(err.contains("keybinds.rename_torrent"));
    }

    #[test]
    fn accepts_default_config() {
        assert_ok!(Config::from_value(assert_ok!(defaults_with(""))));
    }

    #[test]
    fn direct_deserialization_builds_keymap() {
        let quit = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);

        let config: Config = assert_ok!(toml::from_str(DEFAULT_CONFIG));
        assert_some_eq!(config.keymap().action(&quit), Action::Quit);

        let round_tripped: Config =
            assert_ok!(toml::from_str(&assert_ok!(toml::to_string(&config))));
        assert_some_eq!(round_tripped.keymap().action(&quit), Action::Quit);
    }

    #[test]
    fn direct_deserialization_rejects_duplicate_keybind() {
        let input = assert_ok!(toml::to_string(&assert_ok!(default_with_keybinds(
            r#"rename_torrent = "m""#
        ))));

        let err = assert_err!(toml::from_str::<Config>(&input)).to_string();
        assert!(err.contains("keybinds.rename_torrent"));
    }

    #[test]
    fn rejects_invalid_rpc_url() {
        let value = assert_ok!(defaults_with(
            r#"
            [rpc]
            url = "not a url"
            "#,
        ));

        let err = format!("{:#}", assert_err!(Config::from_value(value)));
        assert!(err.contains("rpc.url"));
        assert!(err.contains("not a url"));
    }

    #[test]
    fn rejects_rpc_username_without_password() {
        let value = assert_ok!(defaults_with(
            r#"
            [rpc]
            username = "user"
            "#,
        ));

        let err = format!("{:#}", assert_err!(Config::from_value(value)));
        assert!(err.contains("rpc.password is missing"));
    }

    #[test]
    fn merges_rpc_credentials_over_default_url() {
        let value = assert_ok!(defaults_with(
            r#"
            [rpc]
            username = "user"
            password = "secret"
            "#,
        ));

        let config = assert_ok!(Config::from_value(value));
        assert_eq!(config.rpc.url, "http://localhost:9091/transmission/rpc");
        assert_some!(assert_ok!(config.rpc.basic_auth()));
    }
}
