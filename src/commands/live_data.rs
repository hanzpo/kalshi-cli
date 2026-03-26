use anyhow::Result;

use crate::cli::LiveDataCmd;
use crate::client::KalshiClient;
use crate::output::{OutputConfig, print_json};

pub async fn execute(client: &KalshiClient, cmd: LiveDataCmd, out: &OutputConfig) -> Result<()> {
    match cmd {
        LiveDataCmd::Get {
            milestone_id,
            data_type,
        } => {
            let path = format!("/live_data/{}/milestone/{}", data_type, milestone_id);
            let resp: serde_json::Value = client.get(&path, &[]).await?;
            print_json(&resp, out.no_pager)?;
        }
        LiveDataCmd::Batch { milestone_ids } => {
            let query = [("milestone_ids", milestone_ids.as_str())];
            let resp: serde_json::Value = client.get("/live_data/batch", &query).await?;
            print_json(&resp, out.no_pager)?;
        }
    }
    Ok(())
}
