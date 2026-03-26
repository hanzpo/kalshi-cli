use anyhow::Result;

use crate::cli::ExchangeCmd;
use crate::client::KalshiClient;
use crate::models::exchange::{
    ExchangeStatus, ExchangeStatusResponse, ScheduleResponse, UserDataTimestampResponse,
};
use crate::output::{OutputConfig, output_one, print_json};

pub async fn execute(client: &KalshiClient, cmd: ExchangeCmd, out: &OutputConfig) -> Result<()> {
    match cmd {
        ExchangeCmd::Status => {
            let resp: ExchangeStatusResponse = client.get("/exchange/status", &[]).await?;
            let status = ExchangeStatus {
                exchange_active: resp.exchange_active,
                trading_active: resp.trading_active,
            };
            output_one(&status, out)?;
        }
        ExchangeCmd::Announcement => {
            let resp: serde_json::Value = client.get("/exchange/announcements", &[]).await?;
            print_json(&resp, out.no_pager)?;
        }
        ExchangeCmd::Schedule => {
            let resp: ScheduleResponse = client.get("/exchange/schedule", &[]).await?;
            print_json(&resp, out.no_pager)?;
        }
        ExchangeCmd::UserDataTimestamp => {
            client.require_auth()?;
            let resp: UserDataTimestampResponse =
                client.get("/exchange/user_data_timestamp", &[]).await?;
            println!(
                "User data as of: {}",
                resp.as_of_time.unwrap_or_else(|| "-".to_string())
            );
        }
    }
    Ok(())
}
