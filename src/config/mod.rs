mod colors;
mod keybinds;
mod log;

use color_eyre::Result;
use colors::{ColorsConfig, ColorsConfigFile};
use keybinds::{KeybindsConfig, KeybindsConfigFile};
use log::{LogConfig, LogConfigFile};
use merge::{Merge, option::overwrite_none};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Default, Deserialize, Serialize, Merge)]
pub struct ConfigFile {
    #[merge(strategy = overwrite_none)]
    pub keybinds: Option<KeybindsConfigFile>,
    #[merge(strategy = overwrite_none)]
    pub colors: Option<ColorsConfigFile>,
    #[merge(strategy = overwrite_none)]
    pub log: Option<LogConfigFile>,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub keybinds: KeybindsConfig,
    pub colors: ColorsConfig,
    pub log: LogConfig,
}

impl Config {
    #[tracing::instrument(name = "Loading configuration")]
    pub fn load() -> Result<Self> {
        let mut config = ConfigFile::default();

        // Load system-wide config
        let system_config_path = PathBuf::from("/etc/xdg/traxor/config.toml");
        if system_config_path.exists() {
            info!("Loading system-wide config from: {:?}", system_config_path);
            let config_str = std::fs::read_to_string(&system_config_path)?;
            let system_config = toml::from_str::<ConfigFile>(&config_str)?;
            config.merge(system_config);
            info!("Successfully loaded system-wide config.");
        } else {
            warn!("System-wide config not found at: {:?}", system_config_path);
        }

        // Load user-specific config
        let user_config_path = Self::get_config_path()?;
        if user_config_path.exists() {
            info!("Loading user-specific config from: {:?}", user_config_path);
            let config_str = std::fs::read_to_string(&user_config_path)?;
            let user_config = toml::from_str::<ConfigFile>(&config_str)?;
            config.merge(user_config);
            info!("Successfully loaded user-specific config.");
        } else {
            warn!("User-specific config not found at: {:?}", user_config_path);
        }

        debug!("Configuration loaded successfully.");
        Ok(config.into())
    }

    #[tracing::instrument(name = "Getting config path")]
    fn get_config_path() -> Result<PathBuf> {
        let config_dir = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                dirs::home_dir()
                    .unwrap_or_else(|| panic!("Could not find home directory"))
                    .join(".config")
            });
        Ok(config_dir.join("traxor").join("config.toml"))
    }
}

impl From<ConfigFile> for Config {
    fn from(value: ConfigFile) -> Self {
        Self {
            keybinds: value.keybinds.into(),
            colors: value.colors.into(),
            log: value.log.into(),
        }
    }
}
