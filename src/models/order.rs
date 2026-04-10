use serde::{Deserialize, Serialize};

use crate::color;
use crate::models::common::{flexible_f64, format_opt};
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
    #[serde(default, deserialize_with = "flexible_f64::deserialize")]
    pub yes_price: Option<f64>,
    #[serde(default, deserialize_with = "flexible_f64::deserialize")]
    pub no_price: Option<f64>,
    #[serde(default, deserialize_with = "flexible_f64::deserialize")]
    pub count: Option<f64>,
    #[serde(default, deserialize_with = "flexible_f64::deserialize")]
    pub remaining_count: Option<f64>,
    pub created_time: Option<String>,
    pub updated_time: Option<String>,
    pub expiration_time: Option<String>,
    pub client_order_id: Option<String>,
    #[serde(rename = "type")]
    pub order_type: Option<String>,
    #[serde(default, deserialize_with = "flexible_f64::deserialize")]
    pub queue_position: Option<f64>,
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
pub struct BatchCreateOrderWrapper {
    pub order: Option<Order>,
    #[serde(flatten)]
    _extra: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct BatchCreateResponse {
    pub orders: Option<Vec<BatchCreateOrderWrapper>>,
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
            "Order ID",
            "Ticker",
            "Side",
            "Action",
            "Status",
            "Yes Price",
            "No Price",
            "Count",
            "Remaining",
            "Created",
        ]
    }

    fn row(&self) -> Vec<String> {
        let yes_price = self.yes_price.map(|v| v.to_string()).or_else(|| {
            self.extra
                .get("yes_price_dollars")
                .and_then(|v| v.as_str().map(|s| s.to_string()))
        });
        let no_price = self.no_price.map(|v| v.to_string()).or_else(|| {
            self.extra
                .get("no_price_dollars")
                .and_then(|v| v.as_str().map(|s| s.to_string()))
        });
        let count = self.count.map(|v| v.to_string()).or_else(|| {
            self.extra
                .get("initial_count_fp")
                .and_then(|v| v.as_str().map(|s| s.to_string()))
        });
        let remaining = self.remaining_count.map(|v| v.to_string()).or_else(|| {
            self.extra
                .get("remaining_count_fp")
                .and_then(|v| v.as_str().map(|s| s.to_string()))
        });
        vec![
            self.order_id
                .as_ref()
                .map(|id| {
                    if id.chars().count() > 12 {
                        format!("{}...", id.chars().take(12).collect::<String>())
                    } else {
                        id.clone()
                    }
                })
                .unwrap_or_else(|| "-".to_string()),
            format_opt(&self.ticker),
            format_opt(&self.side),
            format_opt(&self.action),
            format_opt(&self.status),
            format_opt(&yes_price),
            format_opt(&no_price),
            format_opt(&count),
            format_opt(&remaining),
            format_opt(&self.created_time),
        ]
    }

    fn colored_row(&self, c: bool) -> Vec<String> {
        let mut row = self.row();
        if let Some(ref status) = self.status {
            row[4] = color::color_order_status(status, c);
        }
        row
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_order() -> Order {
        Order {
            order_id: Some("abcdef123456789".to_string()),
            ticker: Some("MKT-1".to_string()),
            status: Some("resting".to_string()),
            side: Some("yes".to_string()),
            action: Some("buy".to_string()),
            yes_price: Some(65.0),
            no_price: Some(35.0),
            count: Some(10.0),
            remaining_count: Some(5.0),
            created_time: Some("2026-01-01T00:00:00Z".to_string()),
            updated_time: None,
            expiration_time: None,
            client_order_id: None,
            order_type: None,
            queue_position: None,
            extra: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn test_order_headers_count() {
        let headers = Order::headers();
        assert_eq!(headers.len(), 10);
    }

    #[test]
    fn test_order_headers_values() {
        let headers = Order::headers();
        assert_eq!(headers[0], "Order ID");
        assert_eq!(headers[9], "Created");
    }

    #[test]
    fn test_order_row_truncates_long_id() {
        let order = make_order();
        let row = order.row();
        assert_eq!(row[0], "abcdef123456...");
        assert_eq!(row[0].len(), 15); // 12 + "..."
    }

    #[test]
    fn test_order_row_short_id_no_truncation() {
        let mut order = make_order();
        order.order_id = Some("short-id".to_string());
        let row = order.row();
        assert_eq!(row[0], "short-id");
    }

    #[test]
    fn test_order_row_exactly_12_char_id() {
        let mut order = make_order();
        order.order_id = Some("123456789012".to_string());
        let row = order.row();
        assert_eq!(row[0], "123456789012");
    }

    #[test]
    fn test_order_row_none_id() {
        let mut order = make_order();
        order.order_id = None;
        let row = order.row();
        assert_eq!(row[0], "-");
    }

    #[test]
    fn test_order_row_all_none() {
        let order = Order {
            order_id: None,
            ticker: None,
            status: None,
            side: None,
            action: None,
            yes_price: None,
            no_price: None,
            count: None,
            remaining_count: None,
            created_time: None,
            updated_time: None,
            expiration_time: None,
            client_order_id: None,
            order_type: None,
            queue_position: None,
            extra: std::collections::HashMap::new(),
        };
        let row = order.row();
        for cell in &row {
            assert_eq!(cell, "-");
        }
    }

    #[test]
    fn test_create_order_request_skip_serializing_none() {
        let req = CreateOrderRequest {
            ticker: "MKT-1".to_string(),
            side: "yes".to_string(),
            action: "buy".to_string(),
            count: Some(5),
            yes_price: Some(65),
            no_price: None,
            time_in_force: None,
            expiration_ts: None,
            client_order_id: None,
            post_only: None,
            reduce_only: None,
            buy_max_cost: None,
            order_group_id: None,
        };
        let json = serde_json::to_value(&req).unwrap();
        assert!(json.get("no_price").is_none());
        assert!(json.get("type").is_none());
        assert!(json.get("expiration_ts").is_none());
        assert!(json.get("client_order_id").is_none());
        assert!(json.get("post_only").is_none());
        assert!(json.get("reduce_only").is_none());
        assert!(json.get("buy_max_cost").is_none());
        assert!(json.get("order_group_id").is_none());
        // Present fields
        assert_eq!(json["ticker"], "MKT-1");
        assert_eq!(json["count"], 5);
        assert_eq!(json["yes_price"], 65);
    }

    #[test]
    fn test_create_order_request_with_all_fields() {
        let req = CreateOrderRequest {
            ticker: "MKT-1".to_string(),
            side: "yes".to_string(),
            action: "buy".to_string(),
            count: Some(5),
            yes_price: Some(65),
            no_price: Some(35),
            time_in_force: Some("gtc".to_string()),
            expiration_ts: Some(1700000000),
            client_order_id: Some("client-1".to_string()),
            post_only: Some(true),
            reduce_only: Some(false),
            buy_max_cost: Some(1000),
            order_group_id: Some("group-1".to_string()),
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["no_price"], 35);
        assert_eq!(json["type"], "gtc");
        assert_eq!(json["post_only"], true);
    }

    #[test]
    fn test_batch_cancel_request_serialization() {
        let req = BatchCancelRequest {
            ticker: Some("MKT-1".to_string()),
            order_ids: None,
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["ticker"], "MKT-1");
        assert!(json.get("order_ids").is_none());
    }

    #[test]
    fn test_batch_cancel_request_with_order_ids() {
        let req = BatchCancelRequest {
            ticker: None,
            order_ids: Some(vec!["o1".to_string(), "o2".to_string()]),
        };
        let json = serde_json::to_value(&req).unwrap();
        assert!(json.get("ticker").is_none());
        let ids = json["order_ids"].as_array().unwrap();
        assert_eq!(ids.len(), 2);
    }
}
