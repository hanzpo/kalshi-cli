use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub ticker: String,
    pub above: Option<f64>,
    pub below: Option<f64>,
    pub created_at: String,
}

pub struct AlertStore {
    path: PathBuf,
}

impl AlertStore {
    pub fn new() -> Self {
        let path = crate::config::Config::config_dir().join("alerts.json");
        Self { path }
    }

    pub fn load(&self) -> Result<Vec<Alert>> {
        if !self.path.exists() {
            return Ok(vec![]);
        }
        let contents = std::fs::read_to_string(&self.path)?;
        let alerts: Vec<Alert> = serde_json::from_str(&contents)?;
        Ok(alerts)
    }

    pub fn save(&self, alerts: &[Alert]) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let contents = serde_json::to_string_pretty(alerts)?;
        std::fs::write(&self.path, contents)?;
        Ok(())
    }

    pub fn add(&self, alert: Alert) -> Result<()> {
        let mut alerts = self.load()?;
        alerts.push(alert);
        self.save(&alerts)
    }

    pub fn remove(&self, id: &str) -> Result<bool> {
        let mut alerts = self.load()?;
        let len_before = alerts.len();
        alerts.retain(|a| !a.id.starts_with(id));
        let removed = alerts.len() < len_before;
        self.save(&alerts)?;
        Ok(removed)
    }
}
