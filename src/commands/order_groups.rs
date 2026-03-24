use anyhow::Result;

use crate::cli::OrderGroupCmd;
use crate::client::KalshiClient;
use crate::models::order_group::{
    CreateOrderGroupRequest, OrderGroupResponse, OrderGroupsResponse, UpdateOrderGroupLimitRequest,
};
use crate::output::{OutputFormat, output, output_one, print_json};

pub async fn execute(
    client: &KalshiClient,
    cmd: OrderGroupCmd,
    format: &OutputFormat,
) -> Result<()> {
    client.require_auth()?;

    match cmd {
        OrderGroupCmd::List { limit, cursor } => {
            let mut query = Vec::new();
            let limit_str = limit.map(|l| l.to_string());
            if let Some(ref l) = limit_str {
                query.push(("limit", l.as_str()));
            }
            if let Some(ref c) = cursor {
                query.push(("cursor", c.as_str()));
            }
            let resp: OrderGroupsResponse =
                client.get("/portfolio/order_groups", &query).await?;
            output(&resp.order_groups.unwrap_or_default(), format)?;
        }
        OrderGroupCmd::Create { max_loss } => {
            let req = CreateOrderGroupRequest {
                max_loss,
                tickers: None,
            };
            let resp: OrderGroupResponse =
                client.post("/portfolio/order_groups/create", &req).await?;
            if let Some(og) = resp.order_group {
                output_one(&og, format)?;
            } else {
                print_json(&resp)?;
            }
        }
        OrderGroupCmd::Get { group_id } => {
            let path = format!("/portfolio/order_groups/{}", group_id);
            let resp: OrderGroupResponse = client.get(&path, &[]).await?;
            if let Some(og) = resp.order_group {
                output_one(&og, format)?;
            } else {
                println!("Order group not found.");
            }
        }
        OrderGroupCmd::Delete { group_id } => {
            let path = format!("/portfolio/order_groups/{}", group_id);
            let resp: serde_json::Value = client.delete(&path).await?;
            print_json(&resp)?;
        }
        OrderGroupCmd::Reset { group_id } => {
            let path = format!("/portfolio/order_groups/{}/reset", group_id);
            let resp: serde_json::Value = client.put(&path, &serde_json::json!({})).await?;
            print_json(&resp)?;
        }
        OrderGroupCmd::Trigger { group_id } => {
            let path = format!("/portfolio/order_groups/{}/trigger", group_id);
            let resp: serde_json::Value = client.put(&path, &serde_json::json!({})).await?;
            print_json(&resp)?;
        }
        OrderGroupCmd::UpdateLimit { group_id, max_loss } => {
            let path = format!("/portfolio/order_groups/{}/limit", group_id);
            let req = UpdateOrderGroupLimitRequest { max_loss };
            let resp: serde_json::Value = client.put(&path, &req).await?;
            print_json(&resp)?;
        }
    }
    Ok(())
}
