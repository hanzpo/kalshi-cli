use anyhow::Result;
use futures_util::StreamExt;
use tokio_tungstenite::{connect_async, tungstenite};
use tungstenite::client::IntoClientRequest;

use std::path::Path;

use crate::auth::KalshiSigner;
use crate::config::Config;

const PROD_WS_URL: &str = "wss://api.elections.kalshi.com/trade-api/ws/v2";
const DEMO_WS_URL: &str = "wss://demo-api.kalshi.co/trade-api/ws/v2";

pub type WsSink = futures_util::stream::SplitSink<
    tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    tungstenite::Message,
>;
pub type WsStream = futures_util::stream::SplitStream<
    tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
>;

pub struct KalshiWebSocket {
    ws_url: String,
    signer: Option<KalshiSigner>,
}

impl KalshiWebSocket {
    pub fn new(config: &Config, demo: bool) -> Result<Self> {
        let ws_url = if demo {
            DEMO_WS_URL.to_string()
        } else {
            PROD_WS_URL.to_string()
        };

        let signer = match (&config.api_key_id, &config.private_key_path) {
            (Some(key_id), Some(pem_path)) => {
                Some(KalshiSigner::new(key_id.clone(), Path::new(pem_path))?)
            }
            _ => None,
        };

        Ok(Self { ws_url, signer })
    }

    pub async fn connect(&self) -> Result<(WsSink, WsStream)> {
        let mut req = self.ws_url.as_str().into_client_request()?;

        if let Some(signer) = &self.signer {
            let (key, ts, sig) = signer.sign_request("GET", "/trade-api/ws/v2")?;
            let headers = req.headers_mut();
            headers.insert("KALSHI-ACCESS-KEY", key.parse()?);
            headers.insert("KALSHI-ACCESS-TIMESTAMP", ts.parse()?);
            headers.insert("KALSHI-ACCESS-SIGNATURE", sig.parse()?);
        }

        let (ws, _) = connect_async(req).await?;
        let (sink, stream) = ws.split();
        Ok((sink, stream))
    }

    pub fn subscribe_msg(
        id: u64,
        channels: &[&str],
        market_ticker: Option<&str>,
    ) -> String {
        let mut msg = serde_json::json!({
            "id": id,
            "cmd": "subscribe",
            "params": {
                "channels": channels,
            }
        });
        if let Some(ticker) = market_ticker {
            msg["params"]["market_ticker"] = serde_json::Value::String(ticker.to_string());
        }
        msg.to_string()
    }

    pub fn unsubscribe_msg(
        id: u64,
        channels: &[&str],
        market_ticker: Option<&str>,
    ) -> String {
        let mut msg = serde_json::json!({
            "id": id,
            "cmd": "unsubscribe",
            "params": {
                "channels": channels,
            }
        });
        if let Some(ticker) = market_ticker {
            msg["params"]["market_ticker"] = serde_json::Value::String(ticker.to_string());
        }
        msg.to_string()
    }
}
