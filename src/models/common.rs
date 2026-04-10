use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    Yes,
    No,
}

impl std::fmt::Display for Side {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Side::Yes => write!(f, "yes"),
            Side::No => write!(f, "no"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Buy,
    Sell,
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Buy => write!(f, "buy"),
            Action::Sell => write!(f, "sell"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, clap::ValueEnum)]
pub enum TimeInForce {
    #[serde(rename = "gtc")]
    #[value(alias = "gtc")]
    GoodTillCanceled,
    #[serde(rename = "fok")]
    #[value(alias = "fok")]
    FillOrKill,
    #[serde(rename = "ioc")]
    #[value(alias = "ioc")]
    ImmediateOrCancel,
}

impl std::fmt::Display for TimeInForce {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeInForce::GoodTillCanceled => write!(f, "gtc"),
            TimeInForce::FillOrKill => write!(f, "fok"),
            TimeInForce::ImmediateOrCancel => write!(f, "ioc"),
        }
    }
}

/// Serde helper: deserialize a JSON value that may be a number or a numeric string into an f64.
/// Handles the Kalshi API's inconsistent typing of numeric fields.
pub mod flexible_f64 {
    use serde::{self, Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val: Option<serde_json::Value> = Option::deserialize(deserializer)?;
        match val {
            None | Some(serde_json::Value::Null) => Ok(None),
            Some(serde_json::Value::Number(n)) => Ok(n.as_f64()),
            Some(serde_json::Value::String(s)) => {
                if s.is_empty() {
                    Ok(None)
                } else {
                    s.parse::<f64>().map(Some).map_err(serde::de::Error::custom)
                }
            }
            Some(other) => Err(serde::de::Error::custom(format!(
                "expected number or string, got {:?}",
                other
            ))),
        }
    }
}

/// Format an optional field
pub fn format_opt<T: std::fmt::Display>(val: &Option<T>) -> String {
    match val {
        Some(v) => v.to_string(),
        None => "-".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_opt_some_i32() {
        assert_eq!(format_opt(&Some(42)), "42");
    }

    #[test]
    fn test_format_opt_none_i32() {
        assert_eq!(format_opt(&None::<i32>), "-");
    }

    #[test]
    fn test_format_opt_some_string() {
        assert_eq!(format_opt(&Some("hello")), "hello");
    }

    #[test]
    fn test_format_opt_some_float() {
        assert_eq!(format_opt(&Some(2.5)), "2.5");
    }

    #[test]
    fn test_format_opt_none_string() {
        assert_eq!(format_opt(&None::<String>), "-");
    }

    #[test]
    fn test_format_opt_some_zero() {
        assert_eq!(format_opt(&Some(0)), "0");
    }

    #[test]
    fn test_format_opt_some_empty_string() {
        assert_eq!(format_opt(&Some("")), "");
    }

    #[test]
    fn test_side_display_yes() {
        assert_eq!(format!("{}", Side::Yes), "yes");
    }

    #[test]
    fn test_side_display_no() {
        assert_eq!(format!("{}", Side::No), "no");
    }

    #[test]
    fn test_action_display_buy() {
        assert_eq!(format!("{}", Action::Buy), "buy");
    }

    #[test]
    fn test_action_display_sell() {
        assert_eq!(format!("{}", Action::Sell), "sell");
    }

    #[test]
    fn test_time_in_force_display_gtc() {
        assert_eq!(format!("{}", TimeInForce::GoodTillCanceled), "gtc");
    }

    #[test]
    fn test_time_in_force_display_fok() {
        assert_eq!(format!("{}", TimeInForce::FillOrKill), "fok");
    }

    #[test]
    fn test_time_in_force_display_ioc() {
        assert_eq!(format!("{}", TimeInForce::ImmediateOrCancel), "ioc");
    }

    #[test]
    fn test_side_serde_roundtrip() {
        let json = serde_json::to_string(&Side::Yes).unwrap();
        assert_eq!(json, "\"yes\"");
        let deserialized: Side = serde_json::from_str(&json).unwrap();
        assert_eq!(format!("{}", deserialized), "yes");
    }

    #[test]
    fn test_action_serde_roundtrip() {
        let json = serde_json::to_string(&Action::Sell).unwrap();
        assert_eq!(json, "\"sell\"");
        let deserialized: Action = serde_json::from_str(&json).unwrap();
        assert_eq!(format!("{}", deserialized), "sell");
    }

    #[test]
    fn test_time_in_force_serde_roundtrip() {
        let json = serde_json::to_string(&TimeInForce::FillOrKill).unwrap();
        assert_eq!(json, "\"fok\"");
        let deserialized: TimeInForce = serde_json::from_str(&json).unwrap();
        assert_eq!(format!("{}", deserialized), "fok");
    }
}
