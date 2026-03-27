use serde::{Deserialize, Serialize};

use crate::color;
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

    fn colored_row(&self, c: bool) -> Vec<String> {
        vec![
            self.exchange_active
                .map_or("-".to_string(), |v| color::color_bool(v, c)),
            self.trading_active
                .map_or("-".to_string(), |v| color::color_bool(v, c)),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headers_returns_two_columns() {
        let h = ExchangeStatus::headers();
        assert_eq!(h, vec!["Exchange Active", "Trading Active"]);
    }

    #[test]
    fn row_with_some_true() {
        let s = ExchangeStatus {
            exchange_active: Some(true),
            trading_active: Some(true),
        };
        let row = s.row();
        assert_eq!(row, vec!["true", "true"]);
    }

    #[test]
    fn row_with_some_false() {
        let s = ExchangeStatus {
            exchange_active: Some(false),
            trading_active: Some(false),
        };
        let row = s.row();
        assert_eq!(row, vec!["false", "false"]);
    }

    #[test]
    fn row_with_none_values() {
        let s = ExchangeStatus {
            exchange_active: None,
            trading_active: None,
        };
        let row = s.row();
        assert_eq!(row, vec!["-", "-"]);
    }

    #[test]
    fn row_mixed_values() {
        let s = ExchangeStatus {
            exchange_active: Some(true),
            trading_active: None,
        };
        let row = s.row();
        assert_eq!(row[0], "true");
        assert_eq!(row[1], "-");
    }
}
