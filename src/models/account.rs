use serde::{Deserialize, Serialize};

use crate::models::common::format_opt;
use crate::output::TableDisplay;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountLimits {
    pub usage_tier: Option<String>,
    pub read_limit: Option<i64>,
    pub write_limit: Option<i64>,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

impl TableDisplay for AccountLimits {
    fn headers() -> Vec<&'static str> {
        vec!["Usage Tier", "Read Limit", "Write Limit"]
    }

    fn row(&self) -> Vec<String> {
        vec![
            format_opt(&self.usage_tier),
            format_opt(&self.read_limit),
            format_opt(&self.write_limit),
        ]
    }
}
