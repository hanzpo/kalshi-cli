use anyhow::{Context, Result};

use crate::cli::OrderCmd;
use crate::client::KalshiClient;
use crate::models::order::{
    AmendOrderRequest, BatchCancelRequest, BatchCancelResponse, BatchCreateRequest,
    BatchCreateResponse, CreateOrderRequest, DecreaseOrderRequest, OrderResponse, OrdersResponse,
    QueuePositionsResponse,
};
use crate::output::{OutputConfig, output, output_one, output_paginated, print_json};
use crate::pagination::{PaginationOpts, auto_paginate};

pub async fn execute(client: &KalshiClient, cmd: OrderCmd, out: &OutputConfig) -> Result<()> {
    client.require_auth()?;

    match cmd {
        OrderCmd::Create {
            ticker,
            side,
            action,
            quantity,
            yes_price,
            no_price,
            tif,
            expiration_ts,
            post_only,
            reduce_only,
            client_order_id,
            order_group_id,
            buy_max_cost,
        } => {
            let req = CreateOrderRequest {
                ticker,
                side,
                action,
                count: Some(quantity),
                yes_price,
                no_price,
                time_in_force: tif,
                expiration_ts,
                client_order_id,
                post_only: if post_only { Some(true) } else { None },
                reduce_only: if reduce_only { Some(true) } else { None },
                buy_max_cost,
                order_group_id,
            };
            let resp: OrderResponse = client.post("/portfolio/orders", &req).await?;
            output_one(&resp.order, out)?;
        }
        OrderCmd::List {
            limit,
            cursor,
            all,
            ticker,
            status,
        } => {
            let opts = PaginationOpts { limit, cursor, all };
            let result = auto_paginate(&opts, |page_limit, page_cursor| {
                let ticker = ticker.clone();
                let status = status.clone();
                async move {
                    let mut query = vec![("limit", page_limit.to_string())];
                    if let Some(c) = page_cursor {
                        query.push(("cursor", c));
                    }
                    if let Some(ref t) = ticker {
                        query.push(("ticker", t.clone()));
                    }
                    if let Some(ref s) = status {
                        query.push(("status", s.clone()));
                    }
                    let query_refs: Vec<(&str, &str)> =
                        query.iter().map(|(k, v)| (*k, v.as_str())).collect();
                    let resp: OrdersResponse =
                        client.get("/portfolio/orders", &query_refs).await?;
                    Ok((resp.orders.unwrap_or_default(), resp.cursor))
                }
            })
            .await?;
            output_paginated(&result.items, result.has_more, out)?;
        }
        OrderCmd::Get { order_id } => {
            let path = format!("/portfolio/orders/{}", order_id);
            let resp: OrderResponse = client.get(&path, &[]).await?;
            output_one(&resp.order, out)?;
        }
        OrderCmd::Cancel { order_id } => {
            let path = format!("/portfolio/orders/{}", order_id);
            let resp: OrderResponse = client.delete(&path).await?;
            output_one(&resp.order, out)?;
        }
        OrderCmd::Amend {
            order_id,
            ticker,
            side,
            action,
            quantity,
            yes_price,
            no_price,
        } => {
            let req = AmendOrderRequest {
                ticker,
                side,
                action,
                count: quantity,
                yes_price,
                no_price,
            };
            let path = format!("/portfolio/orders/{}/amend", order_id);
            let resp: OrderResponse = client.post(&path, &req).await?;
            output_one(&resp.order, out)?;
        }
        OrderCmd::Decrease {
            order_id,
            reduce_by,
        } => {
            let req = DecreaseOrderRequest { reduce_by };
            let path = format!("/portfolio/orders/{}/decrease", order_id);
            let resp: OrderResponse = client.post(&path, &req).await?;
            output_one(&resp.order, out)?;
        }
        OrderCmd::BatchCreate { file } => {
            let contents = std::fs::read_to_string(&file)
                .with_context(|| format!("Failed to read {}", file.display()))?;
            let orders: Vec<CreateOrderRequest> = serde_json::from_str(&contents)
                .context("Failed to parse order JSON file")?;
            if orders.len() > 20 {
                anyhow::bail!("Batch create supports a maximum of 20 orders");
            }
            let req = BatchCreateRequest { orders };
            let resp: BatchCreateResponse =
                client.post("/portfolio/orders/batched", &req).await?;
            output(&resp.orders.unwrap_or_default(), out)?;
        }
        OrderCmd::BatchCancel { ticker, order_ids } => {
            let req = BatchCancelRequest { ticker, order_ids };
            let resp: BatchCancelResponse =
                client.delete_with_body("/portfolio/orders/batched", &req).await?;
            println!(
                "Orders canceled: {}",
                resp.orders_canceled.unwrap_or(0)
            );
        }
        OrderCmd::Queue { ticker } => {
            let query = [("ticker", ticker.as_str())];
            let resp: QueuePositionsResponse =
                client.get("/portfolio/orders/queue_positions", &query).await?;
            print_json(&resp.queue_positions, out.no_pager)?;
        }
    }
    Ok(())
}
