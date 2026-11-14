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
    use super::*;
    use claims::{assert_err, assert_ok};

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

        let err = assert_err!(Config::from_value(value)).to_string();
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

        let err = assert_err!(Config::from_value(value)).to_string();
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

        let err = assert_err!(Config::from_value(value)).to_string();
        assert!(err.contains("time.eta_format"));
        Ok(())
    }

    #[test]
    fn progress_colors_default_and_override_individually() -> Result<()> {
        let value = defaults_with(
            r##"
            [colors]
            progress_seeding = "#303030"
            "##,
        )?;

        let colors = Config::from_value(value)?.colors;
        assert_eq!(colors.progress_downloading, "#163a40");
        assert_eq!(colors.progress_seeding, "#303030");
        assert_eq!(colors.progress_stopped, "#1c1c1c");
        assert_eq!(colors.progress_verifying, "#3a3220");
        assert_eq!(colors.progress_queued, "#1c2a3a");
        Ok(())
    }

    #[test]
    fn accepts_default_config() -> Result<()> {
        assert_ok!(Config::from_value(defaults_with("")?));
        Ok(())
    }
}
