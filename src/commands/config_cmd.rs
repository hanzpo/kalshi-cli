use std::io::{self, Write};

use anyhow::Result;

use crate::cli::ConfigCmd;
use crate::config::Config;

pub async fn execute(cmd: ConfigCmd) -> Result<()> {
    match cmd {
        ConfigCmd::Init => {
            println!("Kalshi CLI Configuration");
            println!("========================\n");

            let api_key_id = prompt("API Key ID (leave empty to skip): ")?;
            let private_key_path = prompt("Path to private key PEM file (leave empty to skip): ")?;

            let config = Config {
                api_key_id: if api_key_id.is_empty() {
                    None
                } else {
                    Some(api_key_id)
                },
                private_key_path: if private_key_path.is_empty() {
                    None
                } else {
                    // Expand ~ to home dir
                    let expanded = if private_key_path.starts_with("~/") {
                        dirs::home_dir()
                            .map(|h| h.join(&private_key_path[2..]).to_string_lossy().to_string())
                            .unwrap_or(private_key_path)
                    } else {
                        private_key_path
                    };
                    Some(expanded)
                },
                default_output: Some("table".to_string()),
                demo: None,
                profiles: std::collections::HashMap::new(),
            };

            config.save(None)?;
            let path = Config::config_dir().join("config.toml");
            println!("\nConfig saved to {}", path.display());
        }
        ConfigCmd::Show => {
            let config = Config::load(None)?;
            println!(
                "Config file: {}",
                Config::config_dir().join("config.toml").display()
            );
            println!();
            println!(
                "api_key_id: {}",
                config.api_key_id.as_deref().unwrap_or("(not set)")
            );
            println!(
                "private_key_path: {}",
                config.private_key_path.as_deref().unwrap_or("(not set)")
            );
            println!(
                "default_output: {}",
                config.default_output.as_deref().unwrap_or("table")
            );
            println!("demo: {}", config.demo.unwrap_or(false));
        }
        ConfigCmd::ProfileList => {
            let config = Config::load(None)?;
            if config.profiles.is_empty() {
                println!("No profiles configured.");
            } else {
                for name in config.profiles.keys() {
                    println!("{}", name);
                }
            }
        }
        ConfigCmd::ProfileAdd { name } => {
            let mut config = Config::load(None)?;

            let api_key_id = prompt("API Key ID: ")?;
            let private_key_path = prompt("Path to private key PEM file: ")?;

            let profile = crate::config::Profile {
                api_key_id: if api_key_id.is_empty() {
                    None
                } else {
                    Some(api_key_id)
                },
                private_key_path: if private_key_path.is_empty() {
                    None
                } else {
                    let expanded = if private_key_path.starts_with("~/") {
                        dirs::home_dir()
                            .map(|h| h.join(&private_key_path[2..]).to_string_lossy().to_string())
                            .unwrap_or(private_key_path)
                    } else {
                        private_key_path
                    };
                    Some(expanded)
                },
                demo: None,
            };

            config.profiles.insert(name.clone(), profile);
            config.save(None)?;
            println!("Profile '{}' added.", name);
        }
        ConfigCmd::ProfileRemove { name } => {
            let mut config = Config::load(None)?;
            if config.profiles.remove(&name).is_some() {
                config.save(None)?;
                println!("Profile '{}' removed.", name);
            } else {
                println!("Profile '{}' not found.", name);
            }
        }
    }
    Ok(())
}

fn prompt(msg: &str) -> Result<String> {
    print!("{}", msg);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}
