use anyhow::Result;

use crate::cli::SearchCmd;
use crate::client::KalshiClient;
use crate::output::{OutputConfig, print_json};

pub async fn execute(client: &KalshiClient, cmd: SearchCmd, out: &OutputConfig) -> Result<()> {
    match cmd {
        SearchCmd::Tag => {
            let resp: serde_json::Value = client.get("/search/tags_by_categories", &[]).await?;
            print_json(&resp, out.no_pager)?;
        }
        SearchCmd::Filter => {
            let resp: serde_json::Value = client.get("/search/filters_by_sport", &[]).await?;
            print_json(&resp, out.no_pager)?;
        }
    }
    Ok(())
}
