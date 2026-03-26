use anyhow::Result;

use crate::cli::StructuredTargetCmd;
use crate::client::KalshiClient;
use crate::output::{OutputConfig, print_json};

pub async fn execute(
    client: &KalshiClient,
    cmd: StructuredTargetCmd,
    out: &OutputConfig,
) -> Result<()> {
    match cmd {
        StructuredTargetCmd::List {
            target_type,
            competition,
            page_size,
            cursor,
        } => {
            let mut query = Vec::new();
            if let Some(ref t) = target_type {
                query.push(("type", t.as_str()));
            }
            if let Some(ref c) = competition {
                query.push(("competition", c.as_str()));
            }
            let page_size_str = page_size.map(|p| p.to_string());
            if let Some(ref p) = page_size_str {
                query.push(("page_size", p.as_str()));
            }
            if let Some(ref c) = cursor {
                query.push(("cursor", c.as_str()));
            }
            let resp: serde_json::Value =
                client.get("/structured_targets", &query).await?;
            print_json(&resp, out.no_pager)?;
        }
        StructuredTargetCmd::Get { id } => {
            let path = format!("/structured_targets/{}", id);
            let resp: serde_json::Value = client.get(&path, &[]).await?;
            print_json(&resp, out.no_pager)?;
        }
    }
    Ok(())
}
