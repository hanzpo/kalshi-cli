use anyhow::Result;

use crate::cli::ExchangeCmd;
use crate::client::KalshiClient;
use crate::models::exchange::{ExchangeStatus, ExchangeStatusResponse, ScheduleResponse};
use crate::output::{OutputFormat, output_one, print_json};

pub async fn execute(client: &KalshiClient, cmd: ExchangeCmd, format: &OutputFormat) -> Result<()> {
    match cmd {
        ExchangeCmd::Status => {
            let resp: ExchangeStatusResponse = client.get("/exchange/status", &[]).await?;
            let status = ExchangeStatus {
                exchange_active: resp.exchange_active,
                trading_active: resp.trading_active,
            };
            output_one(&status, format)?;
        }
        ExchangeCmd::Announcements => {
            // The announcements endpoint returns varying structures
            let resp: serde_json::Value = client.get("/exchange/announcements", &[]).await?;
            print_json(&resp)?;
        }
        ExchangeCmd::Schedule => {
            let resp: ScheduleResponse = client.get("/exchange/schedule", &[]).await?;
            print_json(&resp)?;
        }
    }
    Ok(())
}
