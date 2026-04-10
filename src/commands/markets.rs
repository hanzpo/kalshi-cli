use anyhow::Result;

use crate::browse;
use crate::cli::MarketCmd;
use crate::client::KalshiClient;
use crate::color;
use crate::models::market::{
    CandlesticksResponse, MarketResponse, MarketsResponse, OrderbookResponse, TradesResponse,
};
use crate::output::{OutputConfig, output, output_one, output_paginated, paged_print, print_json};
use crate::pagination::{
    DEFAULT_DISPLAY_LIMIT, MARKETS_PAGE_SIZE, PaginationOpts, auto_paginate, paginated_list,
};

/// Maximum number of markets to scan for aggregate commands (hot, expiring, spread).
/// With 1000/page and 200ms throttle, 20000 = 20 requests ≈ 4 seconds.
const SCAN_LIMIT: u32 = 20000;

/// Allowed page sizes for the internal search API.
const SEARCH_PAGE_SIZES: &[u32] = &[3, 5, 8, 25, 30, 50, 70, 100];

/// Pick the nearest allowed page_size for the search API.
fn pick_search_page_size(desired: u32) -> u32 {
    *SEARCH_PAGE_SIZES
        .iter()
        .find(|&&s| s >= desired)
        .unwrap_or(&100)
}

/// Build the query params shared by the paginated fetcher.
fn build_market_query(
    page_limit: u32,
    page_cursor: Option<String>,
    status: &Option<String>,
    series_ticker: &Option<String>,
    event_ticker: &Option<String>,
    include_combos: bool,
) -> Vec<(String, String)> {
    let mut query = vec![("limit".to_string(), page_limit.to_string())];
    if let Some(c) = page_cursor {
        query.push(("cursor".to_string(), c));
    }
    if let Some(s) = status {
        query.push(("status".to_string(), s.clone()));
    }
    if let Some(s) = series_ticker {
        query.push(("series_ticker".to_string(), s.to_uppercase()));
    }
    if let Some(e) = event_ticker {
        query.push(("event_ticker".to_string(), e.to_uppercase()));
    }
    if !include_combos {
        query.push(("mve_filter".to_string(), "exclude".to_string()));
    }
    query
}

async fn fetch_markets_page(
    client: &KalshiClient,
    query: &[(String, String)],
) -> Result<(Vec<crate::models::market::Market>, Option<String>)> {
    let query_refs: Vec<(&str, &str)> = query
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    let resp: MarketsResponse = client.get("/markets", &query_refs).await?;
    Ok((resp.markets.unwrap_or_default(), resp.cursor))
}

pub async fn execute(client: &KalshiClient, cmd: MarketCmd, format: &OutputConfig) -> Result<()> {
    match cmd {
        MarketCmd::List {
            limit,
            cursor,
            all,
            status,
            series_ticker,
            event_ticker,
            include_combos,
            search,
        } => {
            let search_lower = search.map(|s| s.to_lowercase());
            paginated_list(
                all,
                limit,
                cursor,
                Some(MARKETS_PAGE_SIZE),
                format,
                |page_limit, page_cursor| {
                    let status = status.clone();
                    let series_ticker = series_ticker.clone();
                    let event_ticker = event_ticker.clone();
                    let search_lower = search_lower.clone();
                    async move {
                        let query = build_market_query(
                            page_limit,
                            page_cursor,
                            &status,
                            &series_ticker,
                            &event_ticker,
                            include_combos,
                        );
                        let (markets, cursor) = fetch_markets_page(client, &query).await?;
                        let filtered = if let Some(ref needle) = search_lower {
                            markets
                                .into_iter()
                                .filter(|m| {
                                    m.title
                                        .as_ref()
                                        .is_some_and(|t| t.to_lowercase().contains(needle))
                                        || m.ticker
                                            .as_ref()
                                            .is_some_and(|t| t.to_lowercase().contains(needle))
                                })
                                .collect()
                        } else {
                            markets
                        };
                        Ok((filtered, cursor))
                    }
                },
            )
            .await?;
        }
        MarketCmd::Get { ticker } => {
            let upper = ticker.to_uppercase();
            let path = format!("/markets/{}", upper);
            match client.get::<MarketResponse>(&path, &[]).await {
                Ok(resp) => output_one(&resp.market, format)?,
                Err(e) => {
                    let msg = e.to_string();
                    if msg.contains("404") {
                        eprintln!("Error: market '{}' not found.", upper);
                        eprintln!(
                            "Hint: try `kalshi market search \"{}\"` to find matching markets.",
                            upper.split('-').next().unwrap_or(&upper)
                        );
                        return Err(e);
                    }
                    return Err(e);
                }
            }
        }
        MarketCmd::Trade {
            ticker,
            limit,
            cursor,
            all,
            min_ts,
            max_ts,
        } => {
            paginated_list(
                all,
                limit,
                cursor,
                Some(MARKETS_PAGE_SIZE),
                format,
                |page_limit: u32, page_cursor: Option<String>| {
                    let ticker = ticker.clone();
                    async move {
                        let mut query: Vec<(String, String)> =
                            vec![("limit".to_string(), page_limit.to_string())];
                        if let Some(c) = page_cursor {
                            query.push(("cursor".to_string(), c));
                        }
                        if let Some(ref t) = ticker {
                            query.push(("ticker".to_string(), t.clone()));
                        }
                        if let Some(ts) = min_ts {
                            query.push(("min_ts".to_string(), ts.to_string()));
                        }
                        if let Some(ts) = max_ts {
                            query.push(("max_ts".to_string(), ts.to_string()));
                        }
                        let query_refs: Vec<(&str, &str)> = query
                            .iter()
                            .map(|(k, v)| (k.as_str(), v.as_str()))
                            .collect();
                        let resp: TradesResponse =
                            client.get("/markets/trades", &query_refs).await?;
                        Ok((resp.trades.unwrap_or_default(), resp.cursor))
                    }
                },
            )
            .await?;
        }
        MarketCmd::Candlestick {
            ticker,
            series_ticker,
            period,
            start_ts,
            end_ts,
        } => {
            let path = format!(
                "/series/{}/markets/{}/candlesticks",
                series_ticker.to_uppercase(),
                ticker.to_uppercase()
            );
            let mut query = Vec::new();
            let period_str = period.map(|p| p.to_string());
            // Default to last 7 days if not specified (API requires both start_ts and end_ts)
            let now = chrono::Utc::now().timestamp();
            let start_str = Some(start_ts.unwrap_or(now - 7 * 24 * 3600).to_string());
            let end_str = Some(end_ts.unwrap_or(now).to_string());

            if let Some(ref p) = period_str {
                query.push(("period_interval", p.as_str()));
            }
            if let Some(ref s) = start_str {
                query.push(("start_ts", s.as_str()));
            }
            if let Some(ref e) = end_str {
                query.push(("end_ts", e.as_str()));
            }
            let resp: CandlesticksResponse = client.get(&path, &query).await?;
            output_paginated(&resp.candlesticks.unwrap_or_default(), false, format)?;
        }
        MarketCmd::Search {
            query,
            limit,
            cursor,
            all,
            status: _,
            include_combos: _,
        } => {
            use crate::models::market::SearchResponse;

            // Use Kalshi's internal semantic search API.
            let search_url = format!("{}/v1/search/series", client.host());
            // Map our limit to nearest allowed page_size: 3, 5, 8, 25, 30, 50, 70, 100
            let page_size = pick_search_page_size(limit.unwrap_or(DEFAULT_DISPLAY_LIMIT));
            let page_size_str = page_size.to_string();

            if all && format.is_non_interactive() {
                // Non-interactive --all: fetch all pages and dump to stdout
                let ps = pick_search_page_size(100).to_string();
                let mut all_items = Vec::new();
                let mut next = cursor;
                loop {
                    let mut params: Vec<(&str, &str)> = vec![
                        ("query", query.as_str()),
                        ("order_by", "querymatch"),
                        ("page_size", ps.as_str()),
                        ("fuzzy_threshold", "4"),
                    ];
                    let cursor_val;
                    if let Some(ref c) = next {
                        cursor_val = c.clone();
                        params.push(("cursor", &cursor_val));
                    }
                    let resp: SearchResponse = client.get_absolute(&search_url, &params).await?;
                    let items = resp.current_page.unwrap_or_default();
                    let done =
                        items.is_empty() || resp.next_cursor.as_ref().is_none_or(|c| c.is_empty());
                    all_items.extend(items);
                    if done {
                        break;
                    }
                    next = resp.next_cursor;
                }
                output_paginated(&all_items, false, format)?;
            } else if all {
                // Interactive browser
                let display_size = limit.unwrap_or(25);
                let ps = pick_search_page_size(display_size).to_string();
                let initial_cursor = cursor;
                browse::browse(display_size, |_display_limit, page_cursor| {
                    let search_url = search_url.clone();
                    let query = query.clone();
                    let ps = ps.clone();
                    let initial_cursor = initial_cursor.clone();
                    async move {
                        let effective_cursor = page_cursor.or(initial_cursor);
                        let mut params: Vec<(&str, &str)> = vec![
                            ("query", query.as_str()),
                            ("order_by", "querymatch"),
                            ("page_size", ps.as_str()),
                            ("fuzzy_threshold", "4"),
                        ];
                        let cursor_val;
                        if let Some(ref c) = effective_cursor {
                            cursor_val = c.clone();
                            params.push(("cursor", &cursor_val));
                        }
                        let resp: SearchResponse =
                            client.get_absolute(&search_url, &params).await?;
                        let items = resp.current_page.unwrap_or_default();
                        Ok((items, resp.next_cursor))
                    }
                })
                .await?;
            } else {
                let mut params: Vec<(&str, &str)> = vec![
                    ("query", query.as_str()),
                    ("order_by", "querymatch"),
                    ("page_size", &page_size_str),
                    ("fuzzy_threshold", "4"),
                ];
                let cursor_val;
                if let Some(ref c) = cursor {
                    cursor_val = c.clone();
                    params.push(("cursor", &cursor_val));
                }
                let resp: SearchResponse = client.get_absolute(&search_url, &params).await?;
                let items = resp.current_page.unwrap_or_default();
                let has_more = resp.next_cursor.as_ref().is_some_and(|c| !c.is_empty());
                if let Some(total) = resp.total_results_count {
                    if total >= 250 {
                        eprintln!("250+ results found");
                    } else {
                        eprintln!("{} results found", total);
                    }
                }
                output_paginated(&items, has_more, format)?;
            }
        }
        MarketCmd::CandlestickBatch {
            tickers,
            start_ts,
            end_ts,
            period,
        } => {
            let mut query = vec![("market_tickers", tickers.as_str())];
            let period_str = period.map(|p| p.to_string());
            let start_str = start_ts.map(|t| t.to_string());
            let end_str = end_ts.map(|t| t.to_string());
            if let Some(ref p) = period_str {
                query.push(("period_interval", p.as_str()));
            }
            if let Some(ref s) = start_str {
                query.push(("start_ts", s.as_str()));
            }
            if let Some(ref e) = end_str {
                query.push(("end_ts", e.as_str()));
            }
            let resp: serde_json::Value = client.get("/markets/candlesticks", &query).await?;
            print_json(&resp, format.no_pager)?;
        }
        MarketCmd::Orderbook { ticker, depth } => {
            client.require_auth()?;
            let path = format!("/markets/{}/orderbook", ticker.to_uppercase());
            let depth_str = depth.map(|d| d.to_string());
            let mut query = Vec::new();
            if let Some(ref d) = depth_str {
                query.push(("depth", d.as_str()));
            }
            let resp: OrderbookResponse = client.get(&path, &query).await?;
            match resp.orderbook {
                Some(ob) => print_json(&ob, format.no_pager)?,
                None => eprintln!("No orderbook data for {}", ticker),
            }
        }
        MarketCmd::Hot {
            limit: top_n,
            include_combos,
        } => {
            let opts = PaginationOpts {
                limit: Some(SCAN_LIMIT),
                cursor: None,
                all: false,
                max_page_size: Some(MARKETS_PAGE_SIZE),
            };
            let status = Some("open".to_string());
            let result = auto_paginate(&opts, |page_limit, page_cursor| {
                let status = status.clone();
                async move {
                    let query = build_market_query(
                        page_limit,
                        page_cursor,
                        &status,
                        &None,
                        &None,
                        include_combos,
                    );
                    fetch_markets_page(client, &query).await
                }
            })
            .await?;

            let get_volume = |m: &crate::models::market::Market| -> f64 {
                m.volume_24h
                    .map(|v| v as f64)
                    .or_else(|| {
                        m.extra
                            .get("volume_24h_fp")
                            .and_then(|v| v.as_str())
                            .and_then(|s| s.parse().ok())
                    })
                    .or_else(|| m.volume.map(|v| v as f64))
                    .or_else(|| {
                        m.extra
                            .get("volume_fp")
                            .and_then(|v| v.as_str())
                            .and_then(|s| s.parse().ok())
                    })
                    .unwrap_or(0.0)
            };
            let mut markets: Vec<_> = result
                .items
                .into_iter()
                .filter(|m| get_volume(m) > 0.0)
                .collect();
            markets.sort_by(|a, b| {
                get_volume(b)
                    .partial_cmp(&get_volume(a))
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            markets.truncate(top_n as usize);
            if result.has_more {
                eprintln!("(scanned {} markets; results are approximate)", SCAN_LIMIT);
            }
            output(&markets, format)?;
        }
        MarketCmd::Expiring {
            within,
            limit: top_n,
            include_combos,
        } => {
            let opts = PaginationOpts {
                limit: Some(SCAN_LIMIT),
                cursor: None,
                all: false,
                max_page_size: Some(MARKETS_PAGE_SIZE),
            };
            let status = Some("open".to_string());
            let result = auto_paginate(&opts, |page_limit, page_cursor| {
                let status = status.clone();
                async move {
                    let query = build_market_query(
                        page_limit,
                        page_cursor,
                        &status,
                        &None,
                        &None,
                        include_combos,
                    );
                    fetch_markets_page(client, &query).await
                }
            })
            .await?;

            let now = chrono::Utc::now();
            let cutoff = now + chrono::Duration::hours(within as i64);

            let mut expiring: Vec<_> = result
                .items
                .into_iter()
                .filter(|m| {
                    if let Some(ref ct) = m.close_time
                        && let Ok(t) = chrono::DateTime::parse_from_rfc3339(ct)
                    {
                        let t_utc = t.with_timezone(&chrono::Utc);
                        return t_utc > now && t_utc <= cutoff;
                    }
                    false
                })
                .collect();
            expiring.sort_by(|a, b| a.close_time.cmp(&b.close_time));
            if let Some(n) = top_n {
                expiring.truncate(n as usize);
            }
            if result.has_more {
                eprintln!(
                    "(scanned {} markets; some expiring markets may be missing)",
                    SCAN_LIMIT
                );
            }
            output(&expiring, format)?;
        }
        MarketCmd::Spread {
            limit: top_n,
            include_combos,
        } => {
            let opts = PaginationOpts {
                limit: Some(SCAN_LIMIT),
                cursor: None,
                all: false,
                max_page_size: Some(MARKETS_PAGE_SIZE),
            };
            let status = Some("open".to_string());
            let result = auto_paginate(&opts, |page_limit, page_cursor| {
                let status = status.clone();
                async move {
                    let query = build_market_query(
                        page_limit,
                        page_cursor,
                        &status,
                        &None,
                        &None,
                        include_combos,
                    );
                    fetch_markets_page(client, &query).await
                }
            })
            .await?;

            let get_bid = |m: &crate::models::market::Market| -> Option<f64> {
                m.yes_bid.or_else(|| {
                    m.extra
                        .get("yes_bid_dollars")
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse().ok())
                })
            };
            let get_ask = |m: &crate::models::market::Market| -> Option<f64> {
                m.yes_ask.or_else(|| {
                    m.extra
                        .get("yes_ask_dollars")
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse().ok())
                })
            };
            let mut markets: Vec<_> = result
                .items
                .into_iter()
                .filter(|m| {
                    let bid = get_bid(m);
                    let ask = get_ask(m);
                    bid.is_some() && ask.is_some() && bid != ask
                })
                .collect();
            markets.sort_by(|a, b| {
                let spread_a = get_ask(a).unwrap_or(0.0) - get_bid(a).unwrap_or(0.0);
                let spread_b = get_ask(b).unwrap_or(0.0) - get_bid(b).unwrap_or(0.0);
                spread_b
                    .partial_cmp(&spread_a)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            markets.truncate(top_n as usize);
            if result.has_more {
                eprintln!("(scanned {} markets; results are approximate)", SCAN_LIMIT);
            }
            output(&markets, format)?;
        }
        MarketCmd::Dist {
            event_ticker,
            cdf,
            width,
            ask,
            bid,
        } => {
            // Fetch all markets for this event
            let event_ticker = event_ticker.to_uppercase();
            let query =
                build_market_query(1000, None, &None, &None, &Some(event_ticker.clone()), false);
            let (markets, _) = fetch_markets_page(client, &query).await?;
            if markets.is_empty() {
                eprintln!("No markets found for event {}", event_ticker);
                return Ok(());
            }

            let chart = render_dist_colored(&markets, cdf, width, ask, bid, format.color);
            paged_print(&chart, format.no_pager);
        }
        MarketCmd::Analyze { ticker, buy, sell } => {
            client.require_auth()?;
            let path = format!("/markets/{}/orderbook", ticker.to_uppercase());
            let resp: OrderbookResponse = client.get(&path, &[("depth", "100")]).await?;
            let ob = resp
                .orderbook
                .ok_or_else(|| anyhow::anyhow!("No orderbook data returned"))?;

            let yes_levels = ob.yes.unwrap_or_default();
            let no_levels = ob.no.unwrap_or_default();

            let parse_levels = |levels: &[Vec<serde_json::Value>]| -> Vec<(f64, i64)> {
                levels
                    .iter()
                    .filter_map(|row| {
                        if row.len() >= 2 {
                            let price = row[0].as_f64()?;
                            let qty = row[1].as_i64()?;
                            Some((price, qty))
                        } else {
                            None
                        }
                    })
                    .collect()
            };

            let yes_parsed = parse_levels(&yes_levels);
            let no_parsed = parse_levels(&no_levels);

            let best_bid = yes_parsed.last().map(|(p, _)| *p);
            let best_ask = yes_parsed.first().map(|(p, _)| *p);
            let spread = match (best_bid, best_ask) {
                (Some(bid), Some(ask)) => Some(ask - bid),
                _ => None,
            };

            let total_yes_depth: i64 = yes_parsed.iter().map(|(_, q)| *q).sum();
            let total_no_depth: i64 = no_parsed.iter().map(|(_, q)| *q).sum();

            println!("=== Orderbook Analysis: {} ===", ticker);
            println!();
            println!(
                "Best Bid:       {}",
                best_bid.map_or("-".to_string(), |v| format!("{:.2}", v))
            );
            println!(
                "Best Ask:       {}",
                best_ask.map_or("-".to_string(), |v| format!("{:.2}", v))
            );
            println!(
                "Spread:         {}",
                spread.map_or("-".to_string(), |v| format!("{:.2}", v))
            );
            println!("Yes-side depth: {} contracts", total_yes_depth);
            println!("No-side depth:  {} contracts", total_no_depth);

            // Simulate fill
            if let Some(qty) = buy {
                let cost = simulate_fill(&yes_parsed, qty, true);
                println!();
                println!(
                    "Simulated BUY {} contracts: avg price {:.2}c, total cost {:.2}c",
                    qty,
                    if qty > 0 { cost / qty as f64 } else { 0.0 },
                    cost
                );
            }
            if let Some(qty) = sell {
                let proceeds = simulate_fill(&yes_parsed, qty, false);
                println!();
                println!(
                    "Simulated SELL {} contracts: avg price {:.2}c, total proceeds {:.2}c",
                    qty,
                    if qty > 0 { proceeds / qty as f64 } else { 0.0 },
                    proceeds
                );
            }
        }
        MarketCmd::History {
            ticker,
            interval,
            period,
        } => {
            // Parse interval to minutes
            let period_minutes = parse_interval(&interval)?;
            // Parse lookback period to seconds
            let lookback_secs = parse_period(&period)?;

            // Fetch market to get series_ticker
            let ticker_upper = ticker.to_uppercase();
            let market_resp: MarketResponse =
                client.get(&format!("/markets/{ticker_upper}"), &[]).await?;
            let series_ticker = market_resp
                .market
                .extra
                .get("series_ticker")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Could not resolve series ticker for {ticker_upper}. Use `market candlestick` with --series-ticker instead."
                    )
                })?;

            let now = chrono::Utc::now().timestamp();
            let start = now - lookback_secs;
            let period_str = period_minutes.to_string();
            let start_str = start.to_string();
            let end_str = now.to_string();

            let path = format!(
                "/series/{}/markets/{}/candlesticks",
                series_ticker.to_uppercase(),
                ticker_upper
            );
            let query = vec![
                ("period_interval", period_str.as_str()),
                ("start_ts", start_str.as_str()),
                ("end_ts", end_str.as_str()),
            ];
            let resp: CandlesticksResponse = client.get(&path, &query).await?;
            output_paginated(&resp.candlesticks.unwrap_or_default(), false, format)?;
        }
        MarketCmd::Prices { tickers } => {
            let mut markets = Vec::new();
            for ticker in &tickers {
                let ticker_upper = ticker.to_uppercase();
                match client
                    .get::<MarketResponse>(&format!("/markets/{ticker_upper}"), &[])
                    .await
                {
                    Ok(resp) => {
                        markets.push(resp.market);
                    }
                    Err(e) => {
                        eprintln!("Failed to fetch {ticker_upper}: {e}");
                    }
                }
            }
            output_paginated(&markets, false, format)?;
        }
    }
    Ok(())
}

fn parse_interval(s: &str) -> anyhow::Result<i64> {
    match s {
        "1m" => Ok(1),
        "5m" => Ok(5),
        "1h" => Ok(60),
        "6h" => Ok(360),
        "1d" => Ok(1440),
        "1w" => Ok(10080),
        _ => anyhow::bail!("Invalid interval '{s}'. Valid values: 1m, 5m, 1h, 6h, 1d, 1w"),
    }
}

fn parse_period(s: &str) -> anyhow::Result<i64> {
    match s {
        "1d" => Ok(86_400),
        "1w" => Ok(7 * 86_400),
        "1m" => Ok(30 * 86_400),
        "3m" => Ok(90 * 86_400),
        "1y" => Ok(365 * 86_400),
        _ => anyhow::bail!("Invalid period '{s}'. Valid values: 1d, 1w, 1m, 3m, 1y"),
    }
}

/// Render an implied probability distribution chart for an event's markets.
///
/// Markets with ">=" strike prices form a survival function P(X >= n).
/// Default (PMF) mode differences consecutive values to get P(X in [n_i, n_{i+1})).
/// CDF mode (--cdf) shows the raw survival probabilities.
/// Zero-probability buckets are hidden in PMF mode.
/// Labels show range intervals (e.g. "50–60") with "+" on the last bucket.
fn render_dist_colored(
    markets: &[crate::models::market::Market],
    cdf: bool,
    bar_width: usize,
    use_ask: bool,
    use_bid: bool,
    color_enabled: bool,
) -> String {
    // Helper: parse a dollar-string from the extra map (e.g. "0.6500" -> 0.65)
    let extra_f64 = |m: &crate::models::market::Market, key: &str| -> Option<f64> {
        m.extra.get(key).and_then(|v| {
            v.as_str()
                .and_then(|s| s.parse::<f64>().ok())
                .or_else(|| v.as_f64())
        })
    };

    let get_prob = |m: &crate::models::market::Market| -> Option<f64> {
        let get_bid = || m.yes_bid.or_else(|| extra_f64(m, "yes_bid_dollars"));
        let get_ask = || m.yes_ask.or_else(|| extra_f64(m, "yes_ask_dollars"));

        if use_ask {
            get_ask()
        } else if use_bid {
            get_bid()
        } else {
            let bid = get_bid().unwrap_or(0.0);
            let ask = get_ask().unwrap_or(0.0);
            if bid == 0.0 && ask == 0.0 {
                return None;
            }
            Some(if bid == 0.0 {
                ask
            } else if ask == 0.0 {
                bid
            } else {
                (bid + ask) / 2.0
            })
        }
    };

    // Try numeric strike extraction first (for range/numeric markets)
    let mut strikes: Vec<(f64, f64)> = markets
        .iter()
        .filter_map(|m| {
            let prob = get_prob(m)?;
            let strike = m.floor_strike.or_else(|| {
                let text = m.yes_sub_title.as_ref().or(m.title.as_ref())?;
                text.split(|c: char| !c.is_ascii_digit() && c != '.' && c != '-')
                    .find_map(|s| s.parse::<f64>().ok())
            })?;
            Some((strike, prob))
        })
        .collect();

    // If no numeric strikes found, fall back to categorical mode (use labels directly)
    let is_categorical = strikes.is_empty();
    let mut categorical: Vec<(String, f64)> = if is_categorical {
        let mut items: Vec<(String, f64)> = markets
            .iter()
            .filter_map(|m| {
                let prob = get_prob(m)?;
                let label = m
                    .yes_sub_title
                    .as_ref()
                    .or(m.subtitle.as_ref())
                    .or(m.ticker.as_ref())
                    .cloned()
                    .unwrap_or_else(|| "-".to_string());
                Some((label, prob))
            })
            .collect();
        items.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        items
    } else {
        Vec::new()
    };

    if strikes.is_empty() && categorical.is_empty() {
        return "No markets with price data.\n".to_string();
    }

    strikes.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    // Build display rows: (label, probability)
    let display: Vec<(String, f64)> = if is_categorical {
        // Categorical markets: just show label + probability (no CDF/PMF distinction)
        std::mem::take(&mut categorical)
    } else if cdf {
        // CDF mode: show raw survival values with original labels
        strikes
            .iter()
            .map(|(s, p)| (format_strike(*s), *p))
            .collect()
    } else {
        // PMF mode: difference consecutive survival values, use range labels
        let mut pmf = Vec::new();
        for i in 0..strikes.len() {
            let next_prob = strikes.get(i + 1).map(|(_, p)| *p).unwrap_or(0.0);
            let p = (strikes[i].1 - next_prob).max(0.0);
            if p < 0.005 {
                continue; // skip ~0% buckets
            }
            let label = if let Some(next) = strikes.get(i + 1) {
                format!("{}–{}", format_strike(strikes[i].0), format_strike(next.0))
            } else {
                format!("{}+", format_strike(strikes[i].0))
            };
            pmf.push((label, p));
        }
        pmf
    };

    if display.is_empty() {
        return "No significant probability buckets.\n".to_string();
    }

    let max_prob = display.iter().map(|(_, p)| *p).fold(0.0_f64, f64::max);
    let label_width = display
        .iter()
        .map(|(l, _)| l.chars().count())
        .max()
        .unwrap_or(10);

    let mut out = String::new();

    if cdf {
        let price_label = if use_ask {
            "ask"
        } else if use_bid {
            "bid"
        } else {
            "mid"
        };
        out.push_str(&format!(
            "  {}  P(X >= strike), {} price\n\n",
            color::green("SURVIVAL CDF", color_enabled),
            price_label,
        ));
    } else {
        out.push_str(&format!(
            "  {}  P(ends in window)\n\n",
            color::green("PROBABILITY DENSITY", color_enabled),
        ));
    }

    for (label, p) in &display {
        let bar_len = if max_prob > 0.0 {
            ((*p / max_prob) * bar_width as f64).round() as usize
        } else {
            0
        };

        // Color based on probability relative to max (highest prob = hottest)
        let ratio = if max_prob > 0.0 { *p / max_prob } else { 0.0 };
        let bar: String = "\u{2588}".repeat(bar_len);
        let pad: String = " ".repeat(bar_width.saturating_sub(bar_len));
        let pct = format!("{:4.1}%", p * 100.0);

        let colored_bar = color::color_heat(&bar, ratio, color_enabled);
        let colored_pct = if *p < 0.02 {
            color::dim(&pct, color_enabled)
        } else {
            pct
        };

        out.push_str(&format!(
            "  {:>width$}  {}{} {:>6}\n",
            label,
            colored_bar,
            pad,
            colored_pct,
            width = label_width,
        ));
    }
    out.push('\n');
    out
}

/// Format a strike value: show as integer if whole, otherwise keep one decimal.
fn format_strike(v: f64) -> String {
    if (v - v.round()).abs() < 0.01 {
        format!("{}", v as i64)
    } else {
        format!("{:.1}", v)
    }
}

pub(crate) fn simulate_fill(levels: &[(f64, i64)], mut qty: i64, buying: bool) -> f64 {
    let mut total_cost = 0.0;
    // For buying, walk the ask side (ascending price); for selling, walk the bid side (descending)
    let iter: Box<dyn Iterator<Item = &(f64, i64)>> = if buying {
        Box::new(levels.iter())
    } else {
        Box::new(levels.iter().rev())
    };
    for &(price, available) in iter {
        if qty <= 0 {
            break;
        }
        let fill = qty.min(available);
        total_cost += price * fill as f64;
        qty -= fill;
    }
    total_cost
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── simulate_fill tests ──

    #[test]
    fn simulate_fill_buy_across_multiple_levels() {
        let levels = vec![(10.0, 5), (20.0, 10), (30.0, 5)];
        // Buy 8: fills 5@10 + 3@20 = 50 + 60 = 110
        let cost = simulate_fill(&levels, 8, true);
        assert!((cost - 110.0).abs() < f64::EPSILON);
    }

    #[test]
    fn simulate_fill_buy_empty_levels() {
        let levels: Vec<(f64, i64)> = vec![];
        let cost = simulate_fill(&levels, 10, true);
        assert!((cost - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn simulate_fill_buy_more_than_available() {
        let levels = vec![(10.0, 3), (20.0, 2)];
        // Want 10, only 5 available: 3@10 + 2@20 = 30 + 40 = 70
        let cost = simulate_fill(&levels, 10, true);
        assert!((cost - 70.0).abs() < f64::EPSILON);
    }

    #[test]
    fn simulate_fill_sell_reverses_order() {
        let levels = vec![(10.0, 5), (20.0, 10), (30.0, 5)];
        // Sell 8: walks from end → 5@30 + 3@20 = 150 + 60 = 210
        let proceeds = simulate_fill(&levels, 8, false);
        assert!((proceeds - 210.0).abs() < f64::EPSILON);
    }

    #[test]
    fn simulate_fill_buy_exactly_one_level() {
        let levels = vec![(15.0, 7), (25.0, 3)];
        // Buy exactly 7 → fills entire first level: 7@15 = 105
        let cost = simulate_fill(&levels, 7, true);
        assert!((cost - 105.0).abs() < f64::EPSILON);
    }

    #[test]
    fn simulate_fill_single_level_partial() {
        let levels = vec![(42.0, 100)];
        // Buy 10 from single level: 10@42 = 420
        let cost = simulate_fill(&levels, 10, true);
        assert!((cost - 420.0).abs() < f64::EPSILON);
    }

    #[test]
    fn simulate_fill_zero_quantity() {
        let levels = vec![(10.0, 5)];
        let cost = simulate_fill(&levels, 0, true);
        assert!((cost - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn simulate_fill_sell_empty_levels() {
        let levels: Vec<(f64, i64)> = vec![];
        let proceeds = simulate_fill(&levels, 5, false);
        assert!((proceeds - 0.0).abs() < f64::EPSILON);
    }

    // ── pick_search_page_size tests ──

    #[test]
    fn pick_search_page_size_exact_match() {
        assert_eq!(pick_search_page_size(25), 25);
        assert_eq!(pick_search_page_size(100), 100);
    }

    #[test]
    fn pick_search_page_size_rounds_up_to_nearest() {
        assert_eq!(pick_search_page_size(1), 3);
        assert_eq!(pick_search_page_size(4), 5);
        assert_eq!(pick_search_page_size(6), 8);
        assert_eq!(pick_search_page_size(20), 25);
        assert_eq!(pick_search_page_size(26), 30);
        assert_eq!(pick_search_page_size(51), 70);
        assert_eq!(pick_search_page_size(71), 100);
    }

    #[test]
    fn pick_search_page_size_clamps_to_max() {
        // Anything above 100 gets clamped to 100
        assert_eq!(pick_search_page_size(101), 100);
        assert_eq!(pick_search_page_size(500), 100);
    }

    // ── build_market_query tests ──

    #[test]
    fn build_market_query_excludes_combos_by_default() {
        let query = build_market_query(100, None, &None, &None, &None, false);
        assert_eq!(query.len(), 2);
        assert_eq!(query[0], ("limit".to_string(), "100".to_string()));
        assert_eq!(query[1], ("mve_filter".to_string(), "exclude".to_string()));
    }

    #[test]
    fn build_market_query_include_combos() {
        let query = build_market_query(100, None, &None, &None, &None, true);
        assert_eq!(query.len(), 1);
        assert_eq!(query[0], ("limit".to_string(), "100".to_string()));
    }

    #[test]
    fn build_market_query_with_cursor() {
        let query = build_market_query(50, Some("abc123".to_string()), &None, &None, &None, false);
        assert_eq!(query.len(), 3);
        assert_eq!(query[1], ("cursor".to_string(), "abc123".to_string()));
    }

    #[test]
    fn build_market_query_with_all_params() {
        let status = Some("open".to_string());
        let series = Some("SER-1".to_string());
        let event = Some("EVT-1".to_string());
        let query =
            build_market_query(25, Some("cur".to_string()), &status, &series, &event, false);
        assert_eq!(query.len(), 6);
        assert_eq!(query[0].0, "limit");
        assert_eq!(query[1], ("cursor".to_string(), "cur".to_string()));
        assert_eq!(query[2], ("status".to_string(), "open".to_string()));
        assert_eq!(query[3], ("series_ticker".to_string(), "SER-1".to_string()));
        assert_eq!(query[4], ("event_ticker".to_string(), "EVT-1".to_string()));
        assert_eq!(query[5], ("mve_filter".to_string(), "exclude".to_string()));
    }

    #[test]
    fn build_market_query_with_status_only() {
        let status = Some("closed".to_string());
        let query = build_market_query(10, None, &status, &None, &None, false);
        assert_eq!(query.len(), 3);
        assert_eq!(query[1], ("status".to_string(), "closed".to_string()));
    }

    #[test]
    fn build_market_query_with_event_ticker_only() {
        let event = Some("EVT-ABC".to_string());
        let query = build_market_query(10, None, &None, &None, &event, false);
        assert_eq!(query.len(), 3);
        assert_eq!(
            query[1],
            ("event_ticker".to_string(), "EVT-ABC".to_string())
        );
    }

    // ── render_dist tests ──

    fn make_strike_market(
        strike: f64,
        yes_bid: f64,
        yes_ask: f64,
    ) -> crate::models::market::Market {
        crate::models::market::Market {
            ticker: Some(format!("T-G{}", strike as i64)),
            yes_sub_title: Some(format!("At least {}", strike as i64)),
            floor_strike: Some(strike),
            yes_bid: Some(yes_bid),
            yes_ask: Some(yes_ask),
            ..Default::default()
        }
    }

    #[test]
    fn render_dist_pmf_shows_ranges() {
        // Survival: >=50 @ 90%, >=100 @ 60%, >=200 @ 20%
        // PMF: 50–100: 30%, 100–200: 40%, 200+: 20%
        let markets = vec![
            make_strike_market(50.0, 0.90, 0.90),
            make_strike_market(100.0, 0.60, 0.60),
            make_strike_market(200.0, 0.20, 0.20),
        ];
        let out = render_dist_colored(&markets, false, 20, false, false, false);
        assert!(out.contains("PROBABILITY DENSITY"));
        assert!(out.contains("50\u{2013}100")); // range label with en-dash
        assert!(out.contains("100\u{2013}200"));
        assert!(out.contains("200+")); // last bucket
        assert!(out.contains("30.0%"));
        assert!(out.contains("40.0%"));
        assert!(out.contains("20.0%"));
    }

    #[test]
    fn render_dist_hides_zero_buckets() {
        // First two strikes both at 100% → difference is 0, should be hidden
        let markets = vec![
            make_strike_market(10.0, 1.0, 1.0),
            make_strike_market(20.0, 1.0, 1.0),
            make_strike_market(50.0, 0.50, 0.50),
        ];
        let out = render_dist_colored(&markets, false, 20, false, false, false);
        // "10–20" bucket is 0%, should not appear
        assert!(!out.contains("10\u{2013}20"));
        // "20–50" bucket is 50%, should appear
        assert!(out.contains("20\u{2013}50"));
    }

    #[test]
    fn render_dist_cdf_shows_raw_survival() {
        let markets = vec![
            make_strike_market(50.0, 0.80, 0.80),
            make_strike_market(100.0, 0.50, 0.50),
            make_strike_market(200.0, 0.10, 0.10),
        ];
        let out = render_dist_colored(&markets, true, 20, false, false, false);
        assert!(out.contains("SURVIVAL CDF"));
        assert!(out.contains("80.0%"));
        assert!(out.contains("50.0%"));
        assert!(out.contains("10.0%"));
    }

    #[test]
    fn render_dist_empty_markets() {
        let out = render_dist_colored(&[], false, 20, false, false, false);
        assert!(out.contains("No markets with price data"));
    }

    #[test]
    fn render_dist_no_price_data() {
        let markets = vec![crate::models::market::Market {
            ticker: Some("T-1".to_string()),
            floor_strike: Some(100.0),
            ..Default::default()
        }];
        let out = render_dist_colored(&markets, false, 20, false, false, false);
        assert!(out.contains("No markets with price data"));
    }

    #[test]
    fn render_dist_uses_dollar_extras_fallback() {
        let mut extra = std::collections::HashMap::new();
        extra.insert(
            "yes_bid_dollars".to_string(),
            serde_json::Value::String("0.4500".to_string()),
        );
        extra.insert(
            "yes_ask_dollars".to_string(),
            serde_json::Value::String("0.5500".to_string()),
        );
        let markets = vec![crate::models::market::Market {
            ticker: Some("T-1".to_string()),
            yes_sub_title: Some("At least 100".to_string()),
            floor_strike: Some(100.0),
            extra,
            ..Default::default()
        }];
        // Single market → last bucket shows as "100+"
        let out = render_dist_colored(&markets, false, 20, false, false, false);
        assert!(out.contains("100+"));
        assert!(out.contains("50.0%"));
    }

    #[test]
    fn render_dist_sorts_by_floor_strike() {
        let markets = vec![
            make_strike_market(200.0, 0.30, 0.30),
            make_strike_market(50.0, 0.80, 0.80),
            make_strike_market(100.0, 0.50, 0.50),
        ];
        let out = render_dist_colored(&markets, false, 20, false, false, false);
        let data_lines: Vec<&str> = out
            .lines()
            .filter(|l| l.contains('%') && !l.contains("P("))
            .collect();
        assert_eq!(data_lines.len(), 3);
        assert!(data_lines[0].contains("50\u{2013}100"));
        assert!(data_lines[1].contains("100\u{2013}200"));
        assert!(data_lines[2].contains("200+"));
    }
}
