use anyhow::Result;
use futures_util::StreamExt;
use tokio_tungstenite::tungstenite::Message;

use crate::cli::WatchCmd;
use crate::output::OutputConfig;
use crate::websocket::KalshiWebSocket;

pub async fn execute(ws: &KalshiWebSocket, cmd: WatchCmd, _out: &OutputConfig) -> Result<()> {
    let (channel, market_ticker) = match &cmd {
        WatchCmd::Ticker { market } => ("ticker", Some(market.as_str())),
        WatchCmd::Trade { market } => ("trade", Some(market.as_str())),
        WatchCmd::Orderbook { market } => ("orderbook_delta", Some(market.as_str())),
        WatchCmd::Fill => ("fill", None),
        WatchCmd::Position => ("position", None),
    };

    // Private channels require auth
    match &cmd {
        WatchCmd::Fill | WatchCmd::Position => {}
        _ => {}
    }

    let (_sink, mut stream) = ws.connect().await?;

    // Send subscribe message
    let sub_msg = KalshiWebSocket::subscribe_msg(1, &[channel], market_ticker);
    {
        use futures_util::SinkExt;
        let (mut sink, new_stream) = {
            // We need to reconstruct since we split already
            drop(stream);
            let (s, st) = ws.connect().await?;
            (s, st)
        };
        sink.send(Message::Text(sub_msg.into())).await?;

        stream = new_stream;

        let display_label = market_ticker.unwrap_or("all");

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
    }

    Ok(())
}
