use serde::{Deserialize, Serialize};

use crate::output::TableDisplay;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeStatus {
    pub exchange_active: Option<bool>,
    pub trading_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ExchangeStatusResponse {
    pub exchange_active: Option<bool>,
    pub trading_active: Option<bool>,
}

impl TableDisplay for ExchangeStatus {
    fn headers() -> Vec<&'static str> {
        vec!["Exchange Active", "Trading Active"]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.exchange_active
                .map_or("-".to_string(), |v| v.to_string()),
            self.trading_active
                .map_or("-".to_string(), |v| v.to_string()),
        ]
    }
}

#[derive(Debug, Deserialize)]
pub struct UserDataTimestampResponse {
    pub as_of_time: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScheduleResponse {
    pub schedule: Option<Schedule>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Schedule {
    pub standard_hours: Option<serde_json::Value>,
}
