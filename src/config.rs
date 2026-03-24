use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Config {
    pub api_key_id: Option<String>,
    pub private_key_path: Option<String>,
    pub default_output: Option<String>,
    pub demo: Option<bool>,
}

impl Config {
    pub fn load(override_path: Option<&Path>) -> Result<Self> {
        let path = override_path
            .map(PathBuf::from)
            .unwrap_or_else(|| Self::config_dir().join("config.toml"));

        if !path.exists() {
            return Ok(Config::default());
        }

        let contents =
            std::fs::read_to_string(&path).context("Failed to read config file")?;
        let mut config: Config =
            toml::from_str(&contents).context("Failed to parse config file")?;

        // Environment variable overrides
        if let Ok(val) = std::env::var("KALSHI_API_KEY_ID") {
            config.api_key_id = Some(val);
        }
        if let Ok(val) = std::env::var("KALSHI_PRIVATE_KEY_PATH") {
            config.private_key_path = Some(val);
        }

        Ok(config)
    }

    pub fn save(&self, override_path: Option<&Path>) -> Result<()> {
        let path = override_path
            .map(PathBuf::from)
            .unwrap_or_else(|| Self::config_dir().join("config.toml"));

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let contents = toml::to_string_pretty(self)?;
        std::fs::write(&path, contents)?;
        Ok(())
    }

    pub fn config_dir() -> PathBuf {
        dirs::home_dir()
            .expect("Could not determine home directory")
            .join(".kalshi")
    }
}
