use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use anyhow::Result;

use crate::cli::{ExportCmd, ExportFormat};
use crate::client::KalshiClient;
use crate::models::portfolio::{FillsResponse, PositionsResponse, SettlementsResponse};
use crate::output::{OutputConfig, TableDisplay};
use crate::pagination::{MARKETS_PAGE_SIZE, PaginationOpts, auto_paginate};

pub async fn execute(client: &KalshiClient, cmd: ExportCmd, _out: &OutputConfig) -> Result<()> {
    client.require_auth()?;

    match cmd {
        ExportCmd::Fill {
            format,
            since,
            file: output,
        } => {
            let opts = PaginationOpts {
                limit: None,
                cursor: None,
                all: true,
                max_page_size: None,
            };
            let since_str = since.map(|ts| ts.to_string());
            let mut query_params: Vec<(&str, String)> = Vec::new();
            if let Some(ref ts) = since_str {
                query_params.push(("min_ts", ts.clone()));
            }

            let result = auto_paginate(&opts, |limit, cursor| {
                let qp = query_params.clone();
                async move {
                    let mut q: Vec<(&str, String)> = vec![("limit", limit.to_string())];
                    if let Some(c) = cursor {
                        q.push(("cursor", c));
                    }
                    for (k, v) in &qp {
                        q.push((k, v.clone()));
                    }
                    let query_refs: Vec<(&str, &str)> =
                        q.iter().map(|(k, v)| (*k, v.as_str())).collect();
                    let resp: FillsResponse = client.get("/portfolio/fills", &query_refs).await?;
                    let items = resp.fills.unwrap_or_default();
                    Ok((items, resp.cursor))
                }
            })
            .await?;

            write_export(&result.items, &format, &output, "fills")?;
        }
        ExportCmd::Position { format, file: output } => {
            let opts = PaginationOpts {
                limit: None,
                cursor: None,
                all: true,
                max_page_size: Some(MARKETS_PAGE_SIZE),
            };

            let result = auto_paginate(&opts, |limit, cursor| async move {
                let mut q: Vec<(&str, String)> = vec![("limit", limit.to_string())];
                if let Some(c) = cursor {
                    q.push(("cursor", c));
                }
                let query_refs: Vec<(&str, &str)> =
                    q.iter().map(|(k, v)| (*k, v.as_str())).collect();
                let resp: PositionsResponse =
                    client.get("/portfolio/positions", &query_refs).await?;
                let items = resp.market_positions.unwrap_or_default();
                Ok((items, resp.cursor))
            })
            .await?;

            write_export(&result.items, &format, &output, "positions")?;
        }
        ExportCmd::Settlement { format, file: output } => {
            let opts = PaginationOpts {
                limit: None,
                cursor: None,
                all: true,
                max_page_size: None,
            };

            let result = auto_paginate(&opts, |limit, cursor| async move {
                let mut q: Vec<(&str, String)> = vec![("limit", limit.to_string())];
                if let Some(c) = cursor {
                    q.push(("cursor", c));
                }
                let query_refs: Vec<(&str, &str)> =
                    q.iter().map(|(k, v)| (*k, v.as_str())).collect();
                let resp: SettlementsResponse =
                    client.get("/portfolio/settlements", &query_refs).await?;
                let items = resp.settlements.unwrap_or_default();
                Ok((items, resp.cursor))
            })
            .await?;

            write_export(&result.items, &format, &output, "settlements")?;
        }
    }

    Ok(())
}

fn write_export<T: serde::Serialize + TableDisplay>(
    items: &[T],
    format: &ExportFormat,
    path: &Path,
    type_name: &str,
) -> Result<()> {
    match format {
        ExportFormat::Csv => {
            let file = File::create(path)?;
            let mut wtr = csv::Writer::from_writer(file);
            wtr.write_record(T::headers())?;
            for item in items {
                wtr.write_record(item.row())?;
            }
            wtr.flush()?;
        }
        ExportFormat::Json => {
            let file = File::create(path)?;
            serde_json::to_writer_pretty(file, items)?;
        }
        ExportFormat::Jsonl => {
            let file = File::create(path)?;
            let mut writer = BufWriter::new(file);
            for item in items {
                let line = serde_json::to_string(item)?;
                writeln!(writer, "{}", line)?;
            }
            writer.flush()?;
        }
    }

    eprintln!(
        "Exported {} {} to {}",
        items.len(),
        type_name,
        path.display()
    );
    Ok(())
}
