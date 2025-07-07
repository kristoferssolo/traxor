mod colors;
mod keybinds;

use color_eyre::Result;
use colors::ColorsConfig;
use keybinds::KeybindsConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Config {
    pub keybinds: KeybindsConfig,
    pub colors: ColorsConfig,
}

impl Config {
    pub fn load() -> Result<Self> {
        let mut config = Self::default();

        // Load system-wide config
        let system_config_path = PathBuf::from("/etc/xdg/traxor/config.toml");
        if system_config_path.exists() {
            let config_str = std::fs::read_to_string(&system_config_path)?;
            let system_config: Config = toml::from_str(&config_str)?;
            config.merge(system_config);
        }

        // Load user-specific config
        let user_config_path = Self::get_config_path()?;
        if user_config_path.exists() {
            let config_str = std::fs::read_to_string(&user_config_path)?;
            let user_config: Config = toml::from_str(&config_str)?;
            config.merge(user_config);
        }

        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path()?;
        let config_str = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, config_str)?;
        Ok(())
    }

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

    pub fn merge(&mut self, other: Self) {
        self.keybinds.merge(other.keybinds);
        self.colors.merge(other.colors);
    }
}
