use anyhow::Result;

use crate::cli::IncentiveProgramCmd;
use crate::client::KalshiClient;
use crate::output::{OutputConfig, print_json};

pub async fn execute(
    client: &KalshiClient,
    cmd: IncentiveProgramCmd,
    out: &OutputConfig,
) -> Result<()> {
    match cmd {
        IncentiveProgramCmd::List {
            status,
            program_type,
            limit,
            cursor,
            all: _,
        } => {
            let mut query = Vec::new();
            if let Some(ref s) = status {
                query.push(("status", s.as_str()));
            }
            if let Some(ref t) = program_type {
                query.push(("type", t.as_str()));
            }
            let limit_str = limit.map(|l| l.to_string());
            if let Some(ref l) = limit_str {
                query.push(("limit", l.as_str()));
            }
            if let Some(ref c) = cursor {
                query.push(("cursor", c.as_str()));
            }
            let resp: serde_json::Value = client.get("/incentive_programs", &query).await?;
            print_json(&resp, out.no_pager)?;
        }
    }
    Ok(())
}
