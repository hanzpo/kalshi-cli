use anyhow::Result;

use crate::cli::MilestoneCmd;
use crate::client::KalshiClient;
use crate::models::milestone::MilestonesResponse;
use crate::output::{OutputConfig, output_paginated, print_json};

pub async fn execute(client: &KalshiClient, cmd: MilestoneCmd, out: &OutputConfig) -> Result<()> {
    match cmd {
        MilestoneCmd::List {
            limit,
            cursor,
            minimum_start_date,
            category,
            competition,
            source_id,
            milestone_type,
            related_event_ticker,
            min_updated_ts,
        } => {
            let limit_str = limit.to_string();
            let mut query = vec![("limit", limit_str.as_str())];
            if let Some(ref c) = cursor {
                query.push(("cursor", c.as_str()));
            }
            if let Some(ref d) = minimum_start_date {
                query.push(("minimum_start_date", d.as_str()));
            }
            if let Some(ref c) = category {
                query.push(("category", c.as_str()));
            }
            if let Some(ref c) = competition {
                query.push(("competition", c.as_str()));
            }
            if let Some(ref s) = source_id {
                query.push(("source_id", s.as_str()));
            }
            if let Some(ref t) = milestone_type {
                query.push(("type", t.as_str()));
            }
            if let Some(ref r) = related_event_ticker {
                query.push(("related_event_ticker", r.as_str()));
            }
            let min_updated_str = min_updated_ts.map(|t| t.to_string());
            if let Some(ref t) = min_updated_str {
                query.push(("min_updated_ts", t.as_str()));
            }
            let resp: MilestonesResponse = client.get("/milestones", &query).await?;
            let milestones = resp.milestones.unwrap_or_default();
            let has_more = resp.cursor.is_some();
            output_paginated(&milestones, has_more, out)?;
        }
        MilestoneCmd::Get { milestone_id } => {
            let path = format!("/milestones/{}", milestone_id);
            let resp: serde_json::Value = client.get(&path, &[]).await?;
            print_json(&resp, out.no_pager)?;
        }
    }
    Ok(())
}
