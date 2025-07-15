pub mod color;
pub mod keybinds;
pub mod log;

use color::ColorConfig;
use color_eyre::{
    Result,
    eyre::{Context, ContextCompat, Ok},
};
use filecaster::FromFile;
use keybinds::KeybindsConfig;
use log::LogConfig;
use merge::Merge;
use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
};
use tracing::{debug, info, warn};

#[derive(Debug, Clone, FromFile)]
pub struct Config {
    pub keybinds: KeybindsConfig,
    pub colors: ColorConfig,
    pub log: LogConfig,
}

impl Config {
    /// # Errors
    ///
    /// TODO: add error types
    #[tracing::instrument(name = "Loading configuration")]
    pub fn load() -> Result<Self> {
        let mut cfg_file = ConfigFile::default();

        let candidates = [
            ("system-wide", PathBuf::from("/etc/xdg/traxor/config.toml")),
            ("user-specific", get_config_path()?),
        ];

        for (label, path) in &candidates {
            merge_config(&mut cfg_file, label, path)?;
        }

        debug!("Configuration loaded successfully.");
        Ok(cfg_file.into())
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
