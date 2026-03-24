use anyhow::Result;

use crate::cli::SubaccountCmd;
use crate::client::KalshiClient;
use crate::models::subaccount::{
    CreateSubaccountRequest, CreateSubaccountResponse, NettingResponse, SubaccountBalancesResponse,
    TransferRequest, TransferResponse, TransfersResponse,
};
use crate::output::{OutputFormat, output, print_json};

pub async fn execute(
    client: &KalshiClient,
    cmd: SubaccountCmd,
    format: &OutputFormat,
) -> Result<()> {
    client.require_auth()?;

    match cmd {
        SubaccountCmd::Create { name } => {
            let req = CreateSubaccountRequest { name };
            let resp: CreateSubaccountResponse =
                client.post("/portfolio/subaccounts", &req).await?;
            println!(
                "Subaccount created with ID: {}",
                resp.subaccount_id.unwrap_or(0)
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
        SubaccountCmd::Balances => {
            let resp: SubaccountBalancesResponse =
                client.get("/portfolio/subaccounts/balances", &[]).await?;
            output(&resp.subaccount_balances.unwrap_or_default(), format)?;
        }
        SubaccountCmd::Transfers { limit, cursor } => {
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
            output(&resp.transfers.unwrap_or_default(), format)?;
        }
        SubaccountCmd::Netting => {
            let resp: NettingResponse = client
                .get("/portfolio/subaccounts/netting", &[])
                .await?;
            print_json(&resp.data)?;
        }
    }
    Ok(())
}
