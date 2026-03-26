use anyhow::Result;

use crate::cli::AccountCmd;
use crate::client::KalshiClient;
use crate::models::account::AccountLimits;
use crate::output::{OutputConfig, output_one};

pub async fn execute(client: &KalshiClient, cmd: AccountCmd, out: &OutputConfig) -> Result<()> {
    client.require_auth()?;

    match cmd {
        AccountCmd::Limit => {
            let resp: AccountLimits = client.get("/account/limits", &[]).await?;
            output_one(&resp, out)?;
        }
    }
    Ok(())
}
