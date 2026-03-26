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

    #[cfg(test)]
    pub fn with_path(path: PathBuf) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_alert(id: &str, ticker: &str) -> Alert {
        Alert {
            id: id.to_string(),
            ticker: ticker.to_string(),
            above: Some(65.0),
            below: Some(35.0),
            created_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    fn temp_store() -> (AlertStore, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("alerts.json");
        (AlertStore::with_path(path), dir)
    }

    #[test]
    fn test_load_nonexistent_file_returns_empty() {
        let (store, _dir) = temp_store();
        let alerts = store.load().unwrap();
        assert!(alerts.is_empty());
    }

    #[test]
    fn test_add_and_load_single_alert() {
        let (store, _dir) = temp_store();
        let alert = make_alert("abc12345-6789", "TICKER-A");
        store.add(alert).unwrap();
        let alerts = store.load().unwrap();
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].id, "abc12345-6789");
        assert_eq!(alerts[0].ticker, "TICKER-A");
    }

    #[test]
    fn test_add_multiple_alerts() {
        let (store, _dir) = temp_store();
        store.add(make_alert("id-001-aaa", "T1")).unwrap();
        store.add(make_alert("id-002-bbb", "T2")).unwrap();
        store.add(make_alert("id-003-ccc", "T3")).unwrap();
        let alerts = store.load().unwrap();
        assert_eq!(alerts.len(), 3);
    }

    #[test]
    fn test_remove_by_full_id() {
        let (store, _dir) = temp_store();
        store.add(make_alert("id-001-aaa", "T1")).unwrap();
        store.add(make_alert("id-002-bbb", "T2")).unwrap();
        let removed = store.remove("id-001-aaa").unwrap();
        assert!(removed);
        let alerts = store.load().unwrap();
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].id, "id-002-bbb");
    }

    #[test]
    fn test_remove_by_prefix() {
        let (store, _dir) = temp_store();
        store.add(make_alert("abcdef12-3456-7890", "T1")).unwrap();
        store.add(make_alert("xxxxxxxx-yyyy-zzzz", "T2")).unwrap();
        let removed = store.remove("abcdef12").unwrap();
        assert!(removed);
        let alerts = store.load().unwrap();
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].id, "xxxxxxxx-yyyy-zzzz");
    }

    #[test]
    fn test_remove_nonexistent_id() {
        let (store, _dir) = temp_store();
        store.add(make_alert("id-001-aaa", "T1")).unwrap();
        let removed = store.remove("nonexistent").unwrap();
        assert!(!removed);
        let alerts = store.load().unwrap();
        assert_eq!(alerts.len(), 1);
    }

    #[test]
    fn test_alert_serialization_roundtrip() {
        let alert = Alert {
            id: "test-id-123".to_string(),
            ticker: "MARKET-1".to_string(),
            above: Some(70.5),
            below: None,
            created_at: "2026-03-01T12:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&alert).unwrap();
        let deserialized: Alert = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "test-id-123");
        assert_eq!(deserialized.ticker, "MARKET-1");
        assert_eq!(deserialized.above, Some(70.5));
        assert!(deserialized.below.is_none());
    }

    #[test]
    fn test_alert_with_no_thresholds() {
        let alert = Alert {
            id: "id-none".to_string(),
            ticker: "MKT".to_string(),
            above: None,
            below: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&alert).unwrap();
        assert!(json.contains("\"above\":null"));
        assert!(json.contains("\"below\":null"));
    }
}
