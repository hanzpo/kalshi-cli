use serde::{Deserialize, Serialize};

use crate::color;
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

    fn colored_row(&self, c: bool) -> Vec<String> {
        let mut row = self.row();
        if let Some(ref status) = self.status {
            row[1] = color::color_status(status, c);
        }
        row
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headers_returns_four_columns() {
        let h = OrderGroup::headers();
        assert_eq!(h, vec!["ID", "Status", "Max Loss", "Created"]);
    }

    #[test]
    fn row_all_some() {
        let og = OrderGroup {
            id: Some("og-1".to_string()),
            status: Some("active".to_string()),
            max_loss: Some(500),
            created_time: Some("2025-01-15T10:30:00Z".to_string()),
            extra: Default::default(),
        };
        let row = og.row();
        assert_eq!(row, vec!["og-1", "active", "500", "2025-01-15T10:30:00Z"]);
    }

    #[test]
    fn row_all_none() {
        let og = OrderGroup {
            id: None,
            status: None,
            max_loss: None,
            created_time: None,
            extra: Default::default(),
        };
        let row = og.row();
        assert_eq!(row, vec!["-", "-", "-", "-"]);
    }

    #[test]
    fn row_partial_fields() {
        let og = OrderGroup {
            id: Some("og-2".to_string()),
            status: None,
            max_loss: Some(1000),
            created_time: None,
            extra: Default::default(),
        };
        let row = og.row();
        assert_eq!(row[0], "og-2");
        assert_eq!(row[1], "-");
        assert_eq!(row[2], "1000");
        assert_eq!(row[3], "-");
    }
}
