use anyhow::Result;
use futures_util::StreamExt;
use tokio_tungstenite::tungstenite::Message;

use crate::alerts::{Alert, AlertStore};
use crate::cli::AlertCmd;
use crate::output::OutputConfig;
use crate::websocket::KalshiWebSocket;

pub async fn execute(cmd: AlertCmd, ws: &KalshiWebSocket, _out: &OutputConfig) -> Result<()> {
    let store = AlertStore::new();

    match cmd {
        AlertCmd::Add {
            ticker,
            above,
            below,
        } => {
            let id = uuid::Uuid::new_v4().to_string();
            let created_at = chrono::Utc::now().to_rfc3339();
            let alert = Alert {
                id: id.clone(),
                ticker: ticker.clone(),
                above,
                below,
                created_at,
            };
            store.add(alert)?;
            eprintln!("Alert {} created for {}", &id[..8], ticker);
        }
        AlertCmd::List => {
            let alerts = store.load()?;
            if alerts.is_empty() {
                eprintln!("No alerts configured.");
            } else {
                use crate::output::print_table;
                print_table(&alerts, _out.no_pager, _out.color)?;
            }
        }
        AlertCmd::Remove { id } => {
            let removed = store.remove(&id)?;
            if removed {
                eprintln!("Alert {} removed.", id);
            } else {
                eprintln!("Alert {} not found.", id);
            }
        }
        AlertCmd::Watch => {
            let alerts = store.load()?;
            if alerts.is_empty() {
                eprintln!("No alerts to watch.");
                return Ok(());
            }

            // Group alerts by ticker
            let mut tickers: Vec<String> = alerts.iter().map(|a| a.ticker.clone()).collect();
            tickers.sort();
            tickers.dedup();

            let (mut sink, mut stream) = ws.connect().await?;

            // Subscribe to ticker channel for all unique tickers at once
            {
                let ticker_refs: Vec<&str> = tickers.iter().map(|s| s.as_str()).collect();
                let sub_msg =
                    KalshiWebSocket::subscribe_msg(1, &["ticker"], &ticker_refs, false);
                use futures_util::SinkExt;
                sink.send(Message::Text(sub_msg.into())).await?;
            }

            eprintln!(
                "Watching {} alerts across {} tickers...",
                alerts.len(),
                tickers.len()
            );

            loop {
                tokio::select! {
                    msg = stream.next() => {
                        match msg {
                            Some(Ok(Message::Text(text))) => {
                                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                                    check_alerts(&alerts, &val);
                                }
                            }
                            Some(Ok(Message::Ping(data))) => {
                                use futures_util::SinkExt;
                                let _ = sink.send(Message::Pong(data)).await;
                            }
                            Some(Err(e)) => {
                                eprintln!("WebSocket error: {}", e);
                                break;
                            }
                            None => {
                                eprintln!("WebSocket closed");
                                break;
                            }
                            _ => {}
                        }
                    }
                    _ = tokio::signal::ctrl_c() => {
                        eprintln!("\nStopping alert watch...");
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

fn check_alerts(alerts: &[Alert], msg: &serde_json::Value) {
    let ticker = msg
        .get("msg")
        .and_then(|m| m.get("ticker"))
        .and_then(|t| t.as_str());
    let yes_price = msg
        .get("msg")
        .and_then(|m| m.get("yes_price"))
        .and_then(|p| p.as_f64());

    if let (Some(ticker), Some(price)) = (ticker, yes_price) {
        for alert in alerts {
            if alert.ticker != ticker {
                continue;
            }
            if let Some(above) = alert.above && price >= above {
                send_notification(&format!("Alert: {} price {}c >= {}c", ticker, price, above));
            }
            if let Some(below) = alert.below && price <= below {
                send_notification(&format!("Alert: {} price {}c <= {}c", ticker, price, below));
            }
        }
    }
}

fn send_notification(message: &str) {
    eprintln!("{}", message);

    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("osascript")
            .args([
                "-e",
                &format!(
                    "display notification \"{}\" with title \"Kalshi Alert\"",
                    message
                ),
            ])
            .spawn();
    }

    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("notify-send")
            .args(["Kalshi Alert", message])
            .spawn();
    }
}
