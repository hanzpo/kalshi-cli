use serde::{Deserialize, Serialize};

use crate::models::common::{flexible_f64, format_opt};
use crate::output::TableDisplay;

#[derive(Debug, Serialize)]
pub struct CreateSubaccountRequest {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct TransferRequest {
    pub from: i64,
    pub to: i64,
    pub amount: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubaccountBalance {
    #[serde(default, deserialize_with = "flexible_f64::deserialize")]
    pub subaccount_id: Option<f64>,
    #[serde(default, deserialize_with = "flexible_f64::deserialize")]
    pub balance: Option<f64>,
    #[serde(default, deserialize_with = "flexible_f64::deserialize")]
    pub portfolio_value: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct SubaccountBalancesResponse {
    pub subaccount_balances: Option<Vec<SubaccountBalance>>,
}

impl TableDisplay for SubaccountBalance {
    fn headers() -> Vec<&'static str> {
        vec!["Subaccount ID", "Balance", "Portfolio Value"]
    }

    fn row(&self) -> Vec<String> {
        vec![
            format_opt(&self.subaccount_id),
            format_opt(&self.balance),
            format_opt(&self.portfolio_value),
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubaccountTransfer {
    pub id: Option<String>,
    #[serde(default, deserialize_with = "flexible_f64::deserialize")]
    pub from: Option<f64>,
    #[serde(default, deserialize_with = "flexible_f64::deserialize")]
    pub to: Option<f64>,
    #[serde(default, deserialize_with = "flexible_f64::deserialize")]
    pub amount: Option<f64>,
    pub created_time: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TransfersResponse {
    pub transfers: Option<Vec<SubaccountTransfer>>,
}

impl TableDisplay for SubaccountTransfer {
    fn headers() -> Vec<&'static str> {
        vec!["ID", "From", "To", "Amount", "Created"]
    }

    fn row(&self) -> Vec<String> {
        vec![
            format_opt(&self.id),
            format_opt(&self.from),
            format_opt(&self.to),
            format_opt(&self.amount),
            format_opt(&self.created_time),
        ]
    }
}

#[derive(Debug, Deserialize)]
pub struct NettingResponse {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct UpdateNettingRequest {
    pub subaccount_number: i64,
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateSubaccountResponse {
    #[serde(default, deserialize_with = "flexible_f64::deserialize")]
    pub subaccount_id: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct TransferResponse {
    pub transfer_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subaccount_balance_headers() {
        let h = SubaccountBalance::headers();
        assert_eq!(h, vec!["Subaccount ID", "Balance", "Portfolio Value"]);
    }

    #[test]
    fn subaccount_balance_row_all_some() {
        let b = SubaccountBalance {
            subaccount_id: Some(42.0),
            balance: Some(1000.0),
            portfolio_value: Some(5000.0),
        };
        let row = b.row();
        assert_eq!(row, vec!["42", "1000", "5000"]);
    }

    #[test]
    fn subaccount_balance_row_all_none() {
        let b = SubaccountBalance {
            subaccount_id: None,
            balance: None,
            portfolio_value: None,
        };
        let row = b.row();
        assert_eq!(row, vec!["-", "-", "-"]);
    }

    #[test]
    fn subaccount_transfer_headers() {
        let h = SubaccountTransfer::headers();
        assert_eq!(h, vec!["ID", "From", "To", "Amount", "Created"]);
    }

    #[test]
    fn subaccount_transfer_row_all_some() {
        let t = SubaccountTransfer {
            id: Some("xfer-1".to_string()),
            from: Some(1.0),
            to: Some(2.0),
            amount: Some(500.0),
            created_time: Some("2025-06-01T00:00:00Z".to_string()),
        };
        let row = t.row();
        assert_eq!(row[0], "xfer-1");
        assert_eq!(row[1], "1");
        assert_eq!(row[2], "2");
        assert_eq!(row[3], "500");
        assert_eq!(row[4], "2025-06-01T00:00:00Z");
    }

    #[test]
    fn subaccount_transfer_row_all_none() {
        let t = SubaccountTransfer {
            id: None,
            from: None,
            to: None,
            amount: None,
            created_time: None,
        };
        let row = t.row();
        for field in &row {
            assert_eq!(field, "-");
        }
    }

    #[test]
    fn update_netting_request_serialization() {
        let req = UpdateNettingRequest {
            subaccount_number: 7,
            enabled: true,
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["subaccount_number"], 7);
        assert_eq!(json["enabled"], true);
    }

    #[test]
    fn update_netting_request_serialization_disabled() {
        let req = UpdateNettingRequest {
            subaccount_number: 3,
            enabled: false,
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["subaccount_number"], 3);
        assert_eq!(json["enabled"], false);
    }
}
