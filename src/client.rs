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

        let signer = match (&config.api_key_id, &config.private_key_path) {
            (Some(key_id), Some(pem_path)) => {
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
                bail!(KalshiError::Api {
                    status: status_code,
                    message: api_err
                        .message
                        .unwrap_or_else(|| "Unknown error".to_string()),
                    code: api_err.code,
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
            default_output: None,
            demo: None,
            profiles: Default::default(),
        }
    }

    #[test]
    fn prod_base_url_value() {
        assert_eq!(
            PROD_BASE_URL,
            "https://api.elections.kalshi.com/trade-api/v2"
        );
    }

    #[test]
    fn demo_base_url_value() {
        assert_eq!(DEMO_BASE_URL, "https://demo-api.kalshi.co/trade-api/v2");
    }

    #[test]
    fn new_with_demo_true_uses_demo_url() {
        let client = KalshiClient::new(&empty_config(), true).unwrap();
        assert_eq!(client.base_url(), DEMO_BASE_URL);
    }

    #[test]
    fn new_with_demo_false_uses_prod_url() {
        let client = KalshiClient::new(&empty_config(), false).unwrap();
        assert_eq!(client.base_url(), PROD_BASE_URL);
    }

    #[test]
    fn new_with_empty_config_has_no_auth() {
        let client = KalshiClient::new(&empty_config(), false).unwrap();
        assert!(!client.has_auth());
    }

    #[test]
    fn require_auth_without_signer_returns_error() {
        let client = KalshiClient::new(&empty_config(), false).unwrap();
        let result = client.require_auth();
        assert!(result.is_err());
    }

    #[test]
    fn url_construction_concatenates_base_and_path() {
        let client = KalshiClient::new(&empty_config(), false).unwrap();
        let expected = format!("{}/markets", PROD_BASE_URL);
        let actual = format!("{}{}", client.base_url(), "/markets");
        assert_eq!(actual, expected);
    }

    #[test]
    fn url_construction_demo_with_path() {
        let client = KalshiClient::new(&empty_config(), true).unwrap();
        let expected = format!("{}/markets/ABC-123/orderbook", DEMO_BASE_URL);
        let actual = format!("{}{}", client.base_url(), "/markets/ABC-123/orderbook");
        assert_eq!(actual, expected);
    }
}
