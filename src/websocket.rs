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
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    tungstenite::Message,
>;
pub type WsStream = futures_util::stream::SplitStream<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
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

        let signer = match (&config.api_key_id, &config.private_key, &config.private_key_path) {
            (Some(key_id), Some(pem), _) => {
                Some(KalshiSigner::from_pem(key_id.clone(), pem)?)
            }
            (Some(key_id), None, Some(pem_path)) => {
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

    /// Build a subscribe message.
    ///
    /// * `market_tickers` – pass a slice of tickers; empty slice omits the field.
    /// * `send_initial_snapshot` – when `true`, asks the server for an initial
    ///   snapshot (useful for `orderbook_delta`).
    pub fn subscribe_msg(
        id: u64,
        channels: &[&str],
        market_tickers: &[&str],
        send_initial_snapshot: bool,
    ) -> String {
        let mut msg = serde_json::json!({
            "id": id,
            "cmd": "subscribe",
            "params": {
                "channels": channels,
            }
        });
        if market_tickers.len() == 1 {
            msg["params"]["market_ticker"] =
                serde_json::Value::String(market_tickers[0].to_string());
        } else if market_tickers.len() > 1 {
            msg["params"]["market_tickers"] = serde_json::json!(market_tickers);
        }
        if send_initial_snapshot {
            msg["params"]["send_initial_snapshot"] = serde_json::Value::Bool(true);
        }
        msg.to_string()
    }

    /// Unsubscribe by subscription IDs (returned as `sid` in `subscribed`
    /// responses).
    #[allow(dead_code)]
    pub fn unsubscribe_msg(id: u64, sids: &[u64]) -> String {
        serde_json::json!({
            "id": id,
            "cmd": "unsubscribe",
            "params": { "sids": sids }
        })
        .to_string()
    }

    /// Ask the server to list current subscriptions.
    #[allow(dead_code)]
    pub fn list_subscriptions_msg(id: u64) -> String {
        serde_json::json!({
            "id": id,
            "cmd": "list_subscriptions"
        })
        .to_string()
    }

    /// Update an existing subscription by adding or removing markets.
    ///
    /// * `action` – `"add_markets"` or `"delete_markets"`
    #[allow(dead_code)]
    pub fn update_subscription_msg(
        id: u64,
        sids: &[u64],
        market_tickers: &[&str],
        action: &str,
    ) -> String {
        serde_json::json!({
            "id": id,
            "cmd": "update_subscription",
            "params": {
                "sids": sids,
                "market_tickers": market_tickers,
                "action": action
            }
        })
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscribe_msg_single_ticker() {
        let msg = KalshiWebSocket::subscribe_msg(1, &["ticker"], &["MARKET"], false);
        let parsed: serde_json::Value = serde_json::from_str(&msg).unwrap();
        assert_eq!(parsed["id"], 1);
        assert_eq!(parsed["cmd"], "subscribe");
        assert_eq!(parsed["params"]["channels"][0], "ticker");
        assert_eq!(parsed["params"]["market_ticker"], "MARKET");
        // Should use singular field, not array
        assert!(parsed["params"].as_object().unwrap().get("market_tickers").is_none());
    }

    #[test]
    fn test_subscribe_msg_multiple_tickers() {
        let msg = KalshiWebSocket::subscribe_msg(2, &["ticker"], &["A", "B", "C"], false);
        let parsed: serde_json::Value = serde_json::from_str(&msg).unwrap();
        assert_eq!(parsed["params"]["market_tickers"][0], "A");
        assert_eq!(parsed["params"]["market_tickers"][1], "B");
        assert_eq!(parsed["params"]["market_tickers"][2], "C");
        // Should use array field, not singular
        assert!(parsed["params"].as_object().unwrap().get("market_ticker").is_none());
    }

    #[test]
    fn test_subscribe_msg_no_tickers() {
        let msg = KalshiWebSocket::subscribe_msg(3, &["trade"], &[], false);
        let parsed: serde_json::Value = serde_json::from_str(&msg).unwrap();
        let params = parsed["params"].as_object().unwrap();
        assert!(params.get("market_ticker").is_none());
        assert!(params.get("market_tickers").is_none());
    }

    #[test]
    fn test_subscribe_msg_with_snapshot() {
        let msg = KalshiWebSocket::subscribe_msg(4, &["orderbook_delta"], &["MKT"], true);
        let parsed: serde_json::Value = serde_json::from_str(&msg).unwrap();
        assert_eq!(parsed["params"]["send_initial_snapshot"], true);
    }

    #[test]
    fn test_subscribe_msg_without_snapshot() {
        let msg = KalshiWebSocket::subscribe_msg(5, &["orderbook_delta"], &["MKT"], false);
        let parsed: serde_json::Value = serde_json::from_str(&msg).unwrap();
        assert!(parsed["params"].as_object().unwrap().get("send_initial_snapshot").is_none());
    }

    #[test]
    fn test_unsubscribe_msg_by_sids() {
        let msg = KalshiWebSocket::unsubscribe_msg(1, &[10, 20]);
        let parsed: serde_json::Value = serde_json::from_str(&msg).unwrap();
        assert_eq!(parsed["cmd"], "unsubscribe");
        assert_eq!(parsed["params"]["sids"][0], 10);
        assert_eq!(parsed["params"]["sids"][1], 20);
    }

    #[test]
    fn test_list_subscriptions_msg() {
        let msg = KalshiWebSocket::list_subscriptions_msg(7);
        let parsed: serde_json::Value = serde_json::from_str(&msg).unwrap();
        assert_eq!(parsed["id"], 7);
        assert_eq!(parsed["cmd"], "list_subscriptions");
    }

    #[test]
    fn test_update_subscription_msg() {
        let msg =
            KalshiWebSocket::update_subscription_msg(8, &[5], &["NEW-MKT"], "add_markets");
        let parsed: serde_json::Value = serde_json::from_str(&msg).unwrap();
        assert_eq!(parsed["id"], 8);
        assert_eq!(parsed["cmd"], "update_subscription");
        assert_eq!(parsed["params"]["sids"][0], 5);
        assert_eq!(parsed["params"]["market_tickers"][0], "NEW-MKT");
        assert_eq!(parsed["params"]["action"], "add_markets");
    }

    #[test]
    fn test_subscribe_msg_has_required_fields() {
        let msg = KalshiWebSocket::subscribe_msg(5, &["orderbook"], &["MKT-1"], false);
        let parsed: serde_json::Value = serde_json::from_str(&msg).unwrap();
        assert!(parsed.get("id").is_some());
        assert!(parsed.get("cmd").is_some());
        assert!(parsed.get("params").is_some());
        assert!(parsed["params"].get("channels").is_some());
    }

    #[test]
    fn test_subscribe_msg_multiple_channels() {
        let msg =
            KalshiWebSocket::subscribe_msg(10, &["ticker", "trade", "orderbook"], &["X"], false);
        let parsed: serde_json::Value = serde_json::from_str(&msg).unwrap();
        let channels = parsed["params"]["channels"].as_array().unwrap();
        assert_eq!(channels.len(), 3);
    }
}
