use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Profile {
    pub api_key_id: Option<String>,
    pub private_key_path: Option<String>,
    pub private_key: Option<String>,
    pub demo: Option<bool>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Config {
    pub api_key_id: Option<String>,
    pub private_key_path: Option<String>,
    pub private_key: Option<String>,
    pub default_output: Option<String>,
    pub demo: Option<bool>,
    #[serde(default)]
    pub profiles: HashMap<String, Profile>,
}

impl Config {
    pub fn load(override_path: Option<&Path>) -> Result<Self> {
        let path = override_path
            .map(PathBuf::from)
            .unwrap_or_else(|| Self::config_dir().join("config.toml"));

        if !path.exists() {
            return Ok(Config::default());
        }

        let contents = std::fs::read_to_string(&path).context("Failed to read config file")?;
        let mut config: Config =
            toml::from_str(&contents).context("Failed to parse config file")?;

        // Environment variable overrides
        if let Ok(val) = std::env::var("KALSHI_API_KEY_ID") {
            config.api_key_id = Some(val);
        }
        if let Ok(val) = std::env::var("KALSHI_PRIVATE_KEY_PATH") {
            config.private_key_path = Some(val);
        }
        if let Ok(val) = std::env::var("KALSHI_PRIVATE_KEY") {
            config.private_key = Some(val);
        }

        Ok(config)
    }

    /// Overlay a named profile onto the base config.
    pub fn resolve(mut self, profile_name: Option<&str>) -> Result<Self> {
        let name = match profile_name {
            Some(n) => Some(n.to_string()),
            None => std::env::var("KALSHI_PROFILE").ok(),
        };
        if let Some(name) = name {
            let profile = self
                .profiles
                .get(&name)
                .ok_or_else(|| anyhow::anyhow!("Profile '{}' not found in config", name))?;
            if let Some(ref v) = profile.api_key_id {
                self.api_key_id = Some(v.clone());
            }
            if let Some(ref v) = profile.private_key_path {
                self.private_key_path = Some(v.clone());
                self.private_key = None; // clear inline key when path is set
            }
            if let Some(ref v) = profile.private_key {
                self.private_key = Some(v.clone());
                self.private_key_path = None; // clear path when inline key is set
            }
            if let Some(v) = profile.demo {
                self.demo = Some(v);
            }
        }
        Ok(self)
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_default_has_none_fields() {
        let config = Config::default();
        assert!(config.api_key_id.is_none());
        assert!(config.private_key_path.is_none());
        assert!(config.default_output.is_none());
        assert!(config.demo.is_none());
        assert!(config.profiles.is_empty());
    }

    #[test]
    fn test_config_resolve_no_profile() {
        // Unset KALSHI_PROFILE to avoid interference
        unsafe {
            std::env::remove_var("KALSHI_PROFILE");
        }
        let config = Config {
            api_key_id: Some("key123".to_string()),
            ..Config::default()
        };
        let resolved = config.resolve(None).unwrap();
        assert_eq!(resolved.api_key_id, Some("key123".to_string()));
    }

    #[test]
    fn test_config_resolve_valid_profile() {
        let mut profiles = HashMap::new();
        profiles.insert(
            "test".to_string(),
            Profile {
                api_key_id: Some("profile_key".to_string()),
                private_key_path: Some("/path/to/key".to_string()),
                private_key: None,
                demo: Some(true),
            },
        );
        let config = Config {
            api_key_id: Some("base_key".to_string()),
            private_key_path: Some("/base/path".to_string()),
            demo: Some(false),
            profiles,
            ..Config::default()
        };
        let resolved = config.resolve(Some("test")).unwrap();
        assert_eq!(resolved.api_key_id, Some("profile_key".to_string()));
        assert_eq!(resolved.private_key_path, Some("/path/to/key".to_string()));
        assert_eq!(resolved.demo, Some(true));
    }

    #[test]
    fn test_config_resolve_unknown_profile() {
        let config = Config::default();
        let result = config.resolve(Some("nonexistent"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_profile_overlay_partial() {
        let mut profiles = HashMap::new();
        profiles.insert(
            "partial".to_string(),
            Profile {
                api_key_id: Some("override_key".to_string()),
                private_key_path: None,
                private_key: None,
                demo: None,
            },
        );
        let config = Config {
            api_key_id: Some("base_key".to_string()),
            private_key_path: Some("/base/path".to_string()),
            demo: Some(false),
            profiles,
            ..Config::default()
        };
        let resolved = config.resolve(Some("partial")).unwrap();
        assert_eq!(resolved.api_key_id, Some("override_key".to_string()));
        assert_eq!(resolved.private_key_path, Some("/base/path".to_string()));
        assert_eq!(resolved.demo, Some(false));
    }

    #[test]
    fn test_config_save_load_roundtrip() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_path_buf();

        let mut profiles = HashMap::new();
        profiles.insert(
            "demo".to_string(),
            Profile {
                api_key_id: Some("demo_key".to_string()),
                private_key_path: None,
                private_key: None,
                demo: Some(true),
            },
        );
        let config = Config {
            api_key_id: Some("my_key".to_string()),
            private_key_path: Some("/keys/private.pem".to_string()),
            private_key: None,
            default_output: Some("json".to_string()),
            demo: Some(false),
            profiles,
        };
        config.save(Some(&path)).unwrap();

        // Unset env vars that would override loaded config
        unsafe {
            std::env::remove_var("KALSHI_API_KEY_ID");
            std::env::remove_var("KALSHI_PRIVATE_KEY_PATH");
        }

        let loaded = Config::load(Some(&path)).unwrap();
        assert_eq!(loaded.api_key_id, Some("my_key".to_string()));
        assert_eq!(
            loaded.private_key_path,
            Some("/keys/private.pem".to_string())
        );
        assert_eq!(loaded.default_output, Some("json".to_string()));
        assert_eq!(loaded.demo, Some(false));
        assert!(loaded.profiles.contains_key("demo"));
        let demo_profile = loaded.profiles.get("demo").unwrap();
        assert_eq!(demo_profile.api_key_id, Some("demo_key".to_string()));
        assert!(demo_profile.private_key_path.is_none());
        assert_eq!(demo_profile.demo, Some(true));
    }

    #[test]
    fn test_config_load_nonexistent_returns_default() {
        let path = std::path::PathBuf::from("/tmp/kalshi_test_nonexistent_config.toml");
        // make sure it doesn't exist
        let _ = std::fs::remove_file(&path);
        let config = Config::load(Some(&path)).unwrap();
        assert!(config.api_key_id.is_none());
        assert!(config.profiles.is_empty());
    }

    #[test]
    fn test_toml_parsing_with_profile_section() {
        let toml_str = r#"
api_key_id = "root_key"
private_key_path = "/root/key.pem"

[profiles.test]
api_key_id = "test_key"
demo = true
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.api_key_id, Some("root_key".to_string()));
        assert_eq!(config.private_key_path, Some("/root/key.pem".to_string()));
        let profile = config.profiles.get("test").unwrap();
        assert_eq!(profile.api_key_id, Some("test_key".to_string()));
        assert_eq!(profile.demo, Some(true));
        assert!(profile.private_key_path.is_none());
    }

    #[test]
    fn test_toml_parsing_empty_config() {
        let config: Config = toml::from_str("").unwrap();
        assert!(config.api_key_id.is_none());
        assert!(config.profiles.is_empty());
    }

    #[test]
    fn test_config_save_creates_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("subdir").join("config.toml");
        let config = Config::default();
        config.save(Some(&path)).unwrap();
        assert!(path.exists());
    }
}
