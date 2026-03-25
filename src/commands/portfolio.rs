use anyhow::Result;

use crate::cli::PortfolioCmd;
use crate::client::KalshiClient;
use crate::models::portfolio::{
    Balance, BalanceResponse, FillsResponse, PositionsResponse, RestingValueResponse,
    SettlementsResponse,
};
use crate::output::{OutputConfig, output_one, output_paginated};
use crate::pagination::{PaginationOpts, auto_paginate};

pub async fn execute(
    client: &KalshiClient,
    cmd: PortfolioCmd,
    out: &OutputConfig,
) -> Result<()> {
    client.require_auth()?;

    match cmd {
        PortfolioCmd::Balance => {
            let resp: BalanceResponse = client.get("/portfolio/balance", &[]).await?;
            let balance = Balance {
                balance: resp.balance,
                portfolio_value: resp.portfolio_value,
                payout: resp.payout,
            };
            output_one(&balance, out)?;
        }
        PortfolioCmd::Positions {
            limit,
            cursor,
            all,
            ticker,
            event_ticker,
            count_filter,
            settlement_status,
        } => {
            let opts = PaginationOpts { limit, cursor, all };
            let result = auto_paginate(&opts, |page_limit, page_cursor| {
                let ticker = ticker.clone();
                let event_ticker = event_ticker.clone();
                let count_filter = count_filter.clone();
                let settlement_status = settlement_status.clone();
                async move {
                    let mut query = vec![("limit", page_limit.to_string())];
                    if let Some(c) = page_cursor {
                        query.push(("cursor", c));
                    }
                    if let Some(ref t) = ticker {
                        query.push(("ticker", t.clone()));
                    }
                    if let Some(ref e) = event_ticker {
                        query.push(("event_ticker", e.clone()));
                    }
                    if let Some(ref cf) = count_filter {
                        query.push(("count_filter", cf.clone()));
                    }
                    if let Some(ref ss) = settlement_status {
                        query.push(("settlement_status", ss.clone()));
                    }
                    let query_refs: Vec<(&str, &str)> =
                        query.iter().map(|(k, v)| (*k, v.as_str())).collect();
                    let resp: PositionsResponse =
                        client.get("/portfolio/positions", &query_refs).await?;
                    Ok((resp.market_positions.unwrap_or_default(), resp.cursor))
                }
            })
            .await?;
            output_paginated(&result.items, result.has_more, out)?;
        }
        PortfolioCmd::Fills {
            limit,
            cursor,
            all,
            ticker,
            order_id,
            min_ts,
            max_ts,
        } => {
            let opts = PaginationOpts { limit, cursor, all };
            let result = auto_paginate(&opts, |page_limit, page_cursor| {
                let ticker = ticker.clone();
                let order_id = order_id.clone();
                async move {
                    let mut query = vec![("limit", page_limit.to_string())];
                    if let Some(c) = page_cursor {
                        query.push(("cursor", c));
                    }
                    if let Some(ref t) = ticker {
                        query.push(("ticker", t.clone()));
                    }
                    if let Some(ref o) = order_id {
                        query.push(("order_id", o.clone()));
                    }
                    if let Some(ts) = min_ts {
                        query.push(("min_ts", ts.to_string()));
                    }
                    if let Some(ts) = max_ts {
                        query.push(("max_ts", ts.to_string()));
                    }
                    let query_refs: Vec<(&str, &str)> =
                        query.iter().map(|(k, v)| (*k, v.as_str())).collect();
                    let resp: FillsResponse =
                        client.get("/portfolio/fills", &query_refs).await?;
                    Ok((resp.fills.unwrap_or_default(), resp.cursor))
                }
            })
            .await?;
            output_paginated(&result.items, result.has_more, out)?;
        }
        PortfolioCmd::Settlements {
            limit,
            cursor,
            all,
            ticker,
        } => {
            let opts = PaginationOpts { limit, cursor, all };
            let result = auto_paginate(&opts, |page_limit, page_cursor| {
                let ticker = ticker.clone();
                async move {
                    let mut query = vec![("limit", page_limit.to_string())];
                    if let Some(c) = page_cursor {
                        query.push(("cursor", c));
                    }
                    if let Some(ref t) = ticker {
                        query.push(("ticker", t.clone()));
                    }
                    let query_refs: Vec<(&str, &str)> =
                        query.iter().map(|(k, v)| (*k, v.as_str())).collect();
                    let resp: SettlementsResponse =
                        client.get("/portfolio/settlements", &query_refs).await?;
                    Ok((resp.settlements.unwrap_or_default(), resp.cursor))
                }
            })
            .await?;
            output_paginated(&result.items, result.has_more, out)?;
        }
        PortfolioCmd::RestingValue => {
            let resp: RestingValueResponse = client
                .get("/portfolio/summary/total_resting_order_value", &[])
                .await?;
            println!(
                "Total resting order value: {} cents",
                resp.total_resting_order_value.unwrap_or(0)
            );
        }
    }
    Ok(())
}
