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

/// Format an optional field
pub fn format_opt<T: std::fmt::Display>(val: &Option<T>) -> String {
    match val {
        Some(v) => v.to_string(),
        None => "-".to_string(),
    }
}
