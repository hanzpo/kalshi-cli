use serde::{Deserialize, Serialize};

use crate::models::common::format_opt;
use crate::output::TableDisplay;

#[derive(Debug, Serialize)]
pub struct CreateOrderGroupRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_loss: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tickers: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct UpdateOrderGroupLimitRequest {
    pub max_loss: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderGroup {
    pub id: Option<String>,
    pub status: Option<String>,
    pub max_loss: Option<i64>,
    pub created_time: Option<String>,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderGroupResponse {
    pub order_group: Option<OrderGroup>,
}

#[derive(Debug, Deserialize)]
pub struct OrderGroupsResponse {
    pub order_groups: Option<Vec<OrderGroup>>,
}

impl TableDisplay for OrderGroup {
    fn headers() -> Vec<&'static str> {
        vec!["ID", "Status", "Max Loss", "Created"]
    }

    fn row(&self) -> Vec<String> {
        vec![
            format_opt(&self.id),
            format_opt(&self.status),
            format_opt(&self.max_loss),
            format_opt(&self.created_time),
        ]
    }
}
