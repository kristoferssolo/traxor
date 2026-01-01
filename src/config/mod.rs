pub mod color;
pub mod keybinds;
pub mod log;
pub mod tabs;

use color::ColorConfig;
use color_eyre::{
    Result,
    eyre::{Context, ContextCompat, Ok},
};
use filecaster::FromFile;
use keybinds::KeybindsConfig;
use log::LogConfig;
use merge::Merge;
use serde::Deserialize;
use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
};
use tabs::TabConfig;
use tracing::{debug, info, warn};

const DEFAULT_CONFIG_STR: &str = include_str!("../../config/default.toml");

#[derive(Debug, Clone, FromFile)]
pub struct Config {
    pub keybinds: KeybindsConfig,
    pub colors: ColorConfig,
    pub log: LogConfig,
    #[from_file(skip)]
    pub tabs: Vec<TabConfig>,
}

/// Helper struct for parsing tabs from TOML.
#[derive(Debug, Deserialize, Default)]
struct TabsFile {
    #[serde(default)]
    tabs: Vec<TabConfig>,
}

impl Config {
    /// Load configuration with fallback to embedded defaults.
    ///
    /// Merge order:
    /// 1. Embedded defaults
    /// 2. System-wide config (`/etc/xdg/traxor/config.toml`)
    /// 3. User config (`~/.config/traxor/config.toml`)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The embedded default config cannot be parsed (should never happen unless corrupted at build time).
    /// - The system-wide or user config file cannot be read due to I/O errors.
    /// - The TOML in any config file is invalid and cannot be parsed.
    #[tracing::instrument(name = "Loading configuration")]
    pub fn load() -> Result<Self> {
        let mut cfg_file = toml::from_str::<ConfigFile>(DEFAULT_CONFIG_STR)
            .context("Failed to parse embedded default config")?;

        let candidates = [
            ("system-wide", PathBuf::from("/etc/xdg/traxor/config.toml")),
            ("user-specific", get_config_path()?),
        ];

        let mut tabs: Option<Vec<TabConfig>> = None;

        for (label, path) in &candidates {
            merge_config(&mut cfg_file, label, path)?;
            // Load tabs separately (last one wins)
            if let Some(t) = load_tabs(path) {
                tabs = Some(t);
            }
        }

        let mut config: Self = cfg_file.into();
        config.tabs = tabs.unwrap_or_else(tabs::default_tabs);

        debug!("Configuration loaded successfully.");
        Ok(config)
    }
}

fn load_tabs(path: &Path) -> Option<Vec<TabConfig>> {
    if !path.exists() {
        return None;
    }
    let content = read_to_string(path).ok()?;
    let tabs_file: TabsFile = toml::from_str(&content).ok()?;
    if tabs_file.tabs.is_empty() {
        None
    } else {
        Some(tabs_file.tabs)
    }
}

#[tracing::instrument(name = "Getting config path")]
fn get_config_path() -> Result<PathBuf> {
    let config_dir =
        dirs::config_dir().context("Could not determine user configuration directory")?;
    Ok(config_dir.join("traxor").join("config.toml"))
}

#[tracing::instrument(name = "Merging config", skip(cfg_file, path))]
fn merge_config(cfg_file: &mut ConfigFile, label: &str, path: &Path) -> Result<()> {
    if !exists_and_log(label, path) {
        return Ok(());
    }

    info!("Loading {} config from: {:?}", label, path);
    let s = read_config_str(label, path)?;
    let other = parse_config_toml(label, &s)?;

    cfg_file.merge(other);
    info!("Successfully loaded {} config.", label);
    Ok(())
}

fn exists_and_log(label: &str, path: &Path) -> bool {
    if !path.exists() {
        warn!("{} config not found at: {:?}", label, path);
        return false;
    }
    true
}

fn read_config_str(label: &str, path: &Path) -> Result<String> {
    read_to_string(path).with_context(|| {
        format!(
            "Failed to read {label} config file at {}",
            path.to_string_lossy()
        )
    })
}

fn parse_config_toml(label: &str, s: &str) -> Result<ConfigFile> {
    toml::from_str::<ConfigFile>(s)
        .with_context(|| format!("Failed to parse TOML in {label} config"))
}
