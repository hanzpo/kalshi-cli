use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;

use crate::cli::WatchCmd;
use crate::output::OutputConfig;
use crate::websocket::KalshiWebSocket;

pub async fn execute(ws: &KalshiWebSocket, cmd: WatchCmd, _out: &OutputConfig) -> Result<()> {
    let (channel, tickers, snapshot) = match &cmd {
        WatchCmd::Ticker { markets } => ("ticker", markets.clone(), false),
        WatchCmd::Trade { markets } => ("trade", markets.clone(), false),
        WatchCmd::Orderbook { markets, snapshot } => {
            ("orderbook_delta", markets.clone(), *snapshot)
        }
        WatchCmd::Fill { markets } => ("fill", markets.clone(), false),
        WatchCmd::Position { markets } => ("market_positions", markets.clone(), false),
        WatchCmd::Orders { markets } => ("user_orders", markets.clone(), false),
        WatchCmd::Lifecycle => ("market_lifecycle_v2", vec![], false),
        WatchCmd::Communications => ("communications", vec![], false),
        WatchCmd::OrderGroupUpdates => ("order_group_updates", vec![], false),
        WatchCmd::MultivarLifecycle => ("multivariate_market_lifecycle", vec![], false),
        WatchCmd::Multivariate => ("multivariate", vec![], false),
    };

    let (mut sink, mut stream) = ws.connect().await?;

    let ticker_refs: Vec<&str> = tickers.iter().map(|s| s.as_str()).collect();
    let sub_msg = KalshiWebSocket::subscribe_msg(1, &[channel], &ticker_refs, snapshot);
    sink.send(Message::Text(sub_msg.into())).await?;

    let display_label = if tickers.is_empty() {
        "all".to_string()
    } else {
        tickers.join(", ")
    };

    loop {
        tokio::select! {
            msg = stream.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        let val: serde_json::Value = serde_json::from_str(&text)?;
                        // Clear screen
                        print!("\x1B[2J\x1B[H");
                        println!("Watching {} on {}", channel, display_label);
                        println!("{}", serde_json::to_string_pretty(&val)?);
                    }
                    Some(Ok(Message::Ping(data))) => {
                        sink.send(Message::Pong(data)).await?;
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
                eprintln!("\nDisconnecting...");
                break;
            }
        }
    }

    Ok(())
}
