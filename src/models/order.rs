use serde::{Deserialize, Serialize};

use crate::models::common::format_opt;
use crate::output::TableDisplay;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrderRequest {
    pub ticker: String,
    pub side: String,
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yes_price: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_price: Option<i64>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduce_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buy_max_cost: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_group_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AmendOrderRequest {
    pub ticker: String,
    pub side: String,
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yes_price: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_price: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct DecreaseOrderRequest {
    pub reduce_by: i64,
}

#[derive(Debug, Serialize)]
pub struct BatchCreateRequest {
    pub orders: Vec<CreateOrderRequest>,
}

#[derive(Debug, Serialize)]
pub struct BatchCancelRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub order_id: Option<String>,
    pub ticker: Option<String>,
    pub status: Option<String>,
    pub side: Option<String>,
    pub action: Option<String>,
    pub yes_price: Option<i64>,
    pub no_price: Option<i64>,
    pub count: Option<i64>,
    pub remaining_count: Option<i64>,
    pub created_time: Option<String>,
    pub updated_time: Option<String>,
    pub expiration_time: Option<String>,
    pub client_order_id: Option<String>,
    #[serde(rename = "type")]
    pub order_type: Option<String>,
    pub queue_position: Option<i64>,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct OrderResponse {
    pub order: Order,
}

#[derive(Debug, Deserialize)]
pub struct OrdersResponse {
    pub orders: Option<Vec<Order>>,
    pub cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BatchCreateResponse {
    pub orders: Option<Vec<Order>>,
}

#[derive(Debug, Deserialize)]
pub struct BatchCancelResponse {
    pub orders_canceled: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuePosition {
    pub order_id: Option<String>,
    pub queue_position: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct QueuePositionsResponse {
    pub queue_positions: Option<Vec<QueuePosition>>,
}

impl TableDisplay for Order {
    fn headers() -> Vec<&'static str> {
        vec![
            "Order ID", "Ticker", "Side", "Action", "Status", "Yes Price", "No Price", "Count",
            "Remaining", "Created",
        ]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.order_id
                .as_ref()
                .map(|id| if id.len() > 12 { format!("{}...", &id[..12]) } else { id.clone() })
                .unwrap_or_else(|| "-".to_string()),
            format_opt(&self.ticker),
            format_opt(&self.side),
            format_opt(&self.action),
            format_opt(&self.status),
            self.yes_price.map_or("-".into(), |v| format!("{}", v)),
            self.no_price.map_or("-".into(), |v| format!("{}", v)),
            format_opt(&self.count),
            format_opt(&self.remaining_count),
            format_opt(&self.created_time),
        ]
    }
}
