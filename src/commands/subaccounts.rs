use anyhow::Result;

use crate::cli::SubaccountCmd;
use crate::client::KalshiClient;
use crate::models::subaccount::{
    CreateSubaccountRequest, CreateSubaccountResponse, NettingResponse, SubaccountBalancesResponse,
    TransferRequest, TransferResponse, TransfersResponse, UpdateNettingRequest,
};
use crate::output::{OutputConfig, output, print_json};

pub async fn execute(client: &KalshiClient, cmd: SubaccountCmd, out: &OutputConfig) -> Result<()> {
    client.require_auth()?;

    match cmd {
        SubaccountCmd::Create { name } => {
            let req = CreateSubaccountRequest { name };
            let resp: CreateSubaccountResponse =
                client.post("/portfolio/subaccounts", &req).await?;
            println!(
                "Subaccount created with ID: {}",
                resp.subaccount_id.unwrap_or(0.0) as i64
            );
        }
        SubaccountCmd::Transfer { from, to, amount } => {
            let req = TransferRequest { from, to, amount };
            let resp: TransferResponse =
                client.post("/portfolio/subaccounts/transfer", &req).await?;
            println!(
                "Transfer complete. ID: {}",
                resp.transfer_id.unwrap_or_else(|| "-".to_string())
            );
        }
        SubaccountCmd::Balance => {
            let resp: SubaccountBalancesResponse =
                client.get("/portfolio/subaccounts/balances", &[]).await?;
            output(&resp.subaccount_balances.unwrap_or_default(), out)?;
        }
        SubaccountCmd::TransferList {
            limit,
            cursor,
            all: _,
        } => {
            let mut query = Vec::new();
            let limit_str = limit.map(|l| l.to_string());
            if let Some(ref l) = limit_str {
                query.push(("limit", l.as_str()));
            }
            if let Some(ref c) = cursor {
                query.push(("cursor", c.as_str()));
            }
            let resp: TransfersResponse = client
                .get("/portfolio/subaccounts/transfers", &query)
                .await?;
            output(&resp.transfers.unwrap_or_default(), out)?;
        }
        SubaccountCmd::Netting => {
            let resp: NettingResponse = client.get("/portfolio/subaccounts/netting", &[]).await?;
            print_json(&resp.data, out.no_pager)?;
        }
        SubaccountCmd::NettingUpdate {
            subaccount_number,
            enabled,
        } => {
            let req = UpdateNettingRequest {
                subaccount_number,
                enabled,
            };
            let resp: serde_json::Value =
                client.put("/portfolio/subaccounts/netting", &req).await?;
            print_json(&resp, out.no_pager)?;
        }
    }
    Ok(())
}
