use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use anyhow::{Result, bail};
use rand::Rng;
use reqwest::{Client, Method, Response, StatusCode};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::auth::KalshiSigner;
use crate::config::Config;
use crate::error::{ApiErrorResponse, KalshiError};

const PROD_BASE_URL: &str = "https://api.elections.kalshi.com/trade-api/v2";
const DEMO_BASE_URL: &str = "https://demo-api.kalshi.co/trade-api/v2";

const MAX_RETRIES: u32 = 5;
const INITIAL_BACKOFF_MS: u64 = 500;

/// Minimum interval between requests (5 req/s to stay under basic tier limits).
const MIN_REQUEST_INTERVAL: Duration = Duration::from_millis(200);

pub struct KalshiClient {
    http: Client,
    base_url: String,
    signer: Option<KalshiSigner>,
    last_request: Mutex<Option<Instant>>,
}

impl KalshiClient {
    pub fn new(config: &Config, demo: bool) -> Result<Self> {
        let base_url = if demo {
            DEMO_BASE_URL.to_string()
        } else {
            PROD_BASE_URL.to_string()
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

        let http = Client::builder().user_agent("kalshi-cli/0.1.0").build()?;

        Ok(Self {
            http,
            base_url,
            signer,
            last_request: Mutex::new(None),
        })
    }

    pub fn host(&self) -> &str {
        // base_url is like "https://api.elections.kalshi.com/trade-api/v2"
        // extract "https://api.elections.kalshi.com"
        self.base_url
            .find("/trade-api")
            .map(|i| &self.base_url[..i])
            .unwrap_or(&self.base_url)
    }

    #[cfg(test)]
    pub(crate) fn base_url(&self) -> &str {
        &self.base_url
    }

    #[cfg(test)]
    pub(crate) fn has_auth(&self) -> bool {
        self.signer.is_some()
    }

    pub fn require_auth(&self) -> Result<()> {
        if self.signer.is_none() {
            bail!(KalshiError::AuthRequired);
        }
        Ok(())
    }

    pub async fn get<T: DeserializeOwned>(&self, path: &str, query: &[(&str, &str)]) -> Result<T> {
        let resp = self
            .send(Method::GET, path, query, Option::<&()>::None)
            .await?;
        self.parse_response(resp).await
    }

    /// GET request to an absolute URL (not relative to base_url).
    /// Used for internal/undocumented endpoints on different base paths.
    pub async fn get_absolute<T: DeserializeOwned>(
        &self,
        url: &str,
        query: &[(&str, &str)],
    ) -> Result<T> {
        self.throttle().await;
        let mut req = self.http.request(Method::GET, url);
        if !query.is_empty() {
            req = req.query(query);
        }
        let resp = req.send().await?;
        self.parse_response(resp).await
    }

    pub async fn post<B: Serialize, T: DeserializeOwned>(&self, path: &str, body: &B) -> Result<T> {
        let resp = self.send(Method::POST, path, &[], Some(body)).await?;
        self.parse_response(resp).await
    }

    pub async fn put<B: Serialize, T: DeserializeOwned>(&self, path: &str, body: &B) -> Result<T> {
        let resp = self.send(Method::PUT, path, &[], Some(body)).await?;
        self.parse_response(resp).await
    }

    pub async fn delete<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let resp = self
            .send(Method::DELETE, path, &[], Option::<&()>::None)
            .await?;
        self.parse_response(resp).await
    }

    pub async fn delete_with_body<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let resp = self.send(Method::DELETE, path, &[], Some(body)).await?;
        self.parse_response(resp).await
    }

    fn build_request<B: Serialize>(
        &self,
        method: &Method,
        path: &str,
        query: &[(&str, &str)],
        body: Option<&B>,
    ) -> Result<reqwest::RequestBuilder> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.http.request(method.clone(), &url);

        if let Some(signer) = &self.signer {
            let full_path = format!("/trade-api/v2{}", path);
            let (key_id, timestamp, signature) =
                signer.sign_request(method.as_str(), &full_path)?;
            req = req
                .header("KALSHI-ACCESS-KEY", key_id)
                .header("KALSHI-ACCESS-TIMESTAMP", timestamp)
                .header("KALSHI-ACCESS-SIGNATURE", signature);
        }

        if !query.is_empty() {
            req = req.query(query);
        }
        if let Some(body) = body {
            req = req.json(body);
        }

        Ok(req)
    }

    /// Throttle to stay under rate limits.
    async fn throttle(&self) {
        let wait = {
            let mut last = self.last_request.lock().unwrap();
            let now = Instant::now();
            let wait = match *last {
                Some(prev) => {
                    let elapsed = now.duration_since(prev);
                    MIN_REQUEST_INTERVAL.saturating_sub(elapsed)
                }
                None => Duration::ZERO,
            };
            *last = Some(now + wait);
            wait
        };
        if !wait.is_zero() {
            tokio::time::sleep(wait).await;
        }
    }

    async fn send<B: Serialize>(
        &self,
        method: Method,
        path: &str,
        query: &[(&str, &str)],
        body: Option<&B>,
    ) -> Result<Response> {
        let mut backoff_ms = INITIAL_BACKOFF_MS;

        for attempt in 0..=MAX_RETRIES {
            self.throttle().await;

            let req = self.build_request(&method, path, query, body)?;
            let resp = req.send().await?;

            let status = resp.status();
            let retryable = status == StatusCode::TOO_MANY_REQUESTS
                || (status.is_server_error() && status != StatusCode::NOT_IMPLEMENTED);

            if retryable {
                if attempt == MAX_RETRIES {
                    return Ok(resp);
                }
                // Add jitter: 50-150% of the base backoff
                let jitter_ms = {
                    let mut rng = rand::rng();
                    let half = backoff_ms / 2;
                    rng.random_range(half..=backoff_ms + half)
                };
                let label = if status == StatusCode::TOO_MANY_REQUESTS {
                    "Rate limited"
                } else {
                    "Server error"
                };
                eprintln!(
                    "{} ({}), waiting {:.1}s... (retry {}/{})",
                    label,
                    status.as_u16(),
                    jitter_ms as f64 / 1000.0,
                    attempt + 1,
                    MAX_RETRIES
                );
                tokio::time::sleep(Duration::from_millis(jitter_ms)).await;
                backoff_ms *= 2;
                continue;
            }

            return Ok(resp);
        }

        unreachable!()
    }

    async fn parse_response<T: DeserializeOwned>(&self, resp: Response) -> Result<T> {
        let status = resp.status();

        if status.is_success() {
            let body = resp.text().await?;
            if body.is_empty() {
                return Ok(serde_json::from_str("{}")?);
            }
            Ok(serde_json::from_str(&body)?)
        } else {
            let status_code = status.as_u16();
            let body = resp.text().await.unwrap_or_default();

            if let Ok(api_err) = serde_json::from_str::<ApiErrorResponse>(&body) {
                let (code, message) = api_err.into_parts();
                bail!(KalshiError::Api {
                    status: status_code,
                    message: message
                        .unwrap_or_else(|| "Unknown error".to_string()),
                    code,
                });
            }

            bail!(KalshiError::Api {
                status: status_code,
                message: if body.is_empty() {
                    status
                        .canonical_reason()
                        .unwrap_or("Unknown error")
                        .to_string()
                } else {
                    body
                },
                code: None,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    fn empty_config() -> Config {
        Config {
            api_key_id: None,
            private_key_path: None,
            private_key: None,
            default_output: None,
            demo: None,
            profiles: Default::default(),
        }
    }

    #[test]
    fn new_selects_demo_or_prod_url() {
        let demo = KalshiClient::new(&empty_config(), true).unwrap();
        assert_eq!(demo.base_url(), DEMO_BASE_URL);

        let prod = KalshiClient::new(&empty_config(), false).unwrap();
        assert_eq!(prod.base_url(), PROD_BASE_URL);
    }

    #[test]
    fn new_without_credentials_has_no_auth() {
        let client = KalshiClient::new(&empty_config(), false).unwrap();
        assert!(!client.has_auth());
    }

    #[test]
    fn require_auth_fails_without_signer() {
        let client = KalshiClient::new(&empty_config(), false).unwrap();
        let err = client.require_auth().unwrap_err();
        assert!(err.to_string().contains("Authentication required"));
    }

    #[test]
    fn host_extracts_base_from_prod_url() {
        let client = KalshiClient::new(&empty_config(), false).unwrap();
        assert_eq!(client.host(), "https://api.elections.kalshi.com");
    }

    #[test]
    fn host_extracts_base_from_demo_url() {
        let client = KalshiClient::new(&empty_config(), true).unwrap();
        assert_eq!(client.host(), "https://demo-api.kalshi.co");
    }

    #[tokio::test]
    async fn parse_response_success_with_json_body() {
        let client = KalshiClient::new(&empty_config(), false).unwrap();
        let resp = http::Response::builder()
            .status(200)
            .body(r#"{"value": 42}"#)
            .unwrap();
        let resp = Response::from(resp);
        let result: serde_json::Value = client.parse_response(resp).await.unwrap();
        assert_eq!(result["value"], 42);
    }

    #[tokio::test]
    async fn parse_response_success_with_empty_body() {
        let client = KalshiClient::new(&empty_config(), false).unwrap();
        let resp = http::Response::builder()
            .status(200)
            .body("")
            .unwrap();
        let resp = Response::from(resp);
        // Empty body should parse as empty JSON object
        let result: serde_json::Value = client.parse_response(resp).await.unwrap();
        assert_eq!(result, serde_json::json!({}));
    }

    #[tokio::test]
    async fn parse_response_api_error_with_structured_json() {
        let client = KalshiClient::new(&empty_config(), false).unwrap();
        let resp = http::Response::builder()
            .status(403)
            .body(r#"{"code": "FORBIDDEN", "message": "Not allowed"}"#)
            .unwrap();
        let resp = Response::from(resp);
        let err = client.parse_response::<serde_json::Value>(resp).await.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("403"));
        assert!(msg.contains("Not allowed"));
    }

    #[tokio::test]
    async fn parse_response_error_with_plain_text_body() {
        let client = KalshiClient::new(&empty_config(), false).unwrap();
        let resp = http::Response::builder()
            .status(502)
            .body("Bad Gateway upstream")
            .unwrap();
        let resp = Response::from(resp);
        let err = client.parse_response::<serde_json::Value>(resp).await.unwrap_err();
        assert!(err.to_string().contains("Bad Gateway upstream"));
    }

    #[tokio::test]
    async fn parse_response_error_with_empty_body_uses_reason() {
        let client = KalshiClient::new(&empty_config(), false).unwrap();
        let resp = http::Response::builder()
            .status(404)
            .body("")
            .unwrap();
        let resp = Response::from(resp);
        let err = client.parse_response::<serde_json::Value>(resp).await.unwrap_err();
        assert!(err.to_string().contains("Not Found"));
    }

    #[tokio::test]
    async fn throttle_enforces_minimum_interval() {
        let client = KalshiClient::new(&empty_config(), false).unwrap();
        let start = Instant::now();
        client.throttle().await; // first call — no wait
        client.throttle().await; // second call — should wait ~200ms
        let elapsed = start.elapsed();
        assert!(elapsed >= Duration::from_millis(180)); // allow small timing slack
    }

    #[test]
    fn new_with_inline_pem_creates_signer() {
        // Generate a test key
        use rsa::RsaPrivateKey;
        use rsa::pkcs8::{EncodePrivateKey, LineEnding};
        let key = RsaPrivateKey::new(&mut rsa::rand_core::OsRng, 2048).unwrap();
        let pem = key.to_pkcs8_pem(LineEnding::LF).unwrap().to_string();

        let config = Config {
            api_key_id: Some("test-key".to_string()),
            private_key: Some(pem),
            private_key_path: None,
            default_output: None,
            demo: None,
            profiles: Default::default(),
        };
        let client = KalshiClient::new(&config, false).unwrap();
        assert!(client.has_auth());
    }
}
