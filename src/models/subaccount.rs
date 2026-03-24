use serde::{Deserialize, Serialize};

use crate::models::common::format_opt;
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
    pub subaccount_id: Option<i64>,
    pub balance: Option<i64>,
    pub portfolio_value: Option<i64>,
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
    pub from: Option<i64>,
    pub to: Option<i64>,
    pub amount: Option<i64>,
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

#[derive(Debug, Deserialize)]
pub struct CreateSubaccountResponse {
    pub subaccount_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct TransferResponse {
    pub transfer_id: Option<String>,
}
