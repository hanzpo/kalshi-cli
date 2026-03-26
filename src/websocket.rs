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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscribe_msg_with_market_ticker() {
        let msg = KalshiWebSocket::subscribe_msg(1, &["ticker"], Some("MARKET"));
        let parsed: serde_json::Value = serde_json::from_str(&msg).unwrap();
        assert_eq!(parsed["id"], 1);
        assert_eq!(parsed["cmd"], "subscribe");
        assert_eq!(parsed["params"]["channels"][0], "ticker");
        assert_eq!(parsed["params"]["market_ticker"], "MARKET");
    }

    #[test]
    fn test_subscribe_msg_without_market_ticker() {
        let msg = KalshiWebSocket::subscribe_msg(2, &["ticker", "trade"], None);
        let parsed: serde_json::Value = serde_json::from_str(&msg).unwrap();
        assert_eq!(parsed["id"], 2);
        assert_eq!(parsed["cmd"], "subscribe");
        assert_eq!(parsed["params"]["channels"][0], "ticker");
        assert_eq!(parsed["params"]["channels"][1], "trade");
        assert!(parsed["params"]["market_ticker"].is_null());
    }

    #[test]
    fn test_unsubscribe_msg_cmd() {
        let msg = KalshiWebSocket::unsubscribe_msg(1, &["ticker"], Some("MARKET"));
        let parsed: serde_json::Value = serde_json::from_str(&msg).unwrap();
        assert_eq!(parsed["cmd"], "unsubscribe");
        assert_eq!(parsed["params"]["market_ticker"], "MARKET");
    }

    #[test]
    fn test_subscribe_msg_has_required_fields() {
        let msg = KalshiWebSocket::subscribe_msg(5, &["orderbook"], Some("MKT-1"));
        let parsed: serde_json::Value = serde_json::from_str(&msg).unwrap();
        assert!(parsed.get("id").is_some());
        assert!(parsed.get("cmd").is_some());
        assert!(parsed.get("params").is_some());
        assert!(parsed["params"].get("channels").is_some());
    }

    #[test]
    fn test_market_ticker_absent_when_none() {
        let msg = KalshiWebSocket::subscribe_msg(1, &["ticker"], None);
        let parsed: serde_json::Value = serde_json::from_str(&msg).unwrap();
        // The key should not exist in the JSON object
        assert!(parsed["params"].as_object().unwrap().get("market_ticker").is_none());
    }

    #[test]
    fn test_unsubscribe_msg_without_market_ticker() {
        let msg = KalshiWebSocket::unsubscribe_msg(3, &["trade"], None);
        let parsed: serde_json::Value = serde_json::from_str(&msg).unwrap();
        assert_eq!(parsed["cmd"], "unsubscribe");
        assert_eq!(parsed["id"], 3);
        assert!(parsed["params"].as_object().unwrap().get("market_ticker").is_none());
    }

    #[test]
    fn test_subscribe_msg_multiple_channels() {
        let msg = KalshiWebSocket::subscribe_msg(10, &["ticker", "trade", "orderbook"], Some("X"));
        let parsed: serde_json::Value = serde_json::from_str(&msg).unwrap();
        let channels = parsed["params"]["channels"].as_array().unwrap();
        assert_eq!(channels.len(), 3);
    }
}
