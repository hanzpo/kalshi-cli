use std::path::Path;

use anyhow::{Result, bail};
use reqwest::{Client, Method, Response};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::auth::KalshiSigner;
use crate::config::Config;
use crate::error::{ApiErrorResponse, KalshiError};

const PROD_BASE_URL: &str = "https://api.elections.kalshi.com/trade-api/v2";
const DEMO_BASE_URL: &str = "https://demo-api.kalshi.co/trade-api/v2";

pub struct KalshiClient {
    http: Client,
    base_url: String,
    signer: Option<KalshiSigner>,
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

        let http = Client::builder()
            .user_agent("kalshi-cli/0.1.0")
            .build()?;

        Ok(Self {
            http,
            base_url,
            signer,
        })
    }

    pub fn require_auth(&self) -> Result<()> {
        if self.signer.is_none() {
            bail!(KalshiError::AuthRequired);
        }
        Ok(())
    }

    pub async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, &str)],
    ) -> Result<T> {
        let resp = self.send(Method::GET, path, query, Option::<&()>::None).await?;
        self.parse_response(resp).await
    }

    pub async fn post<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let resp = self.send(Method::POST, path, &[], Some(body)).await?;
        self.parse_response(resp).await
    }

    pub async fn put<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
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

    async fn send<B: Serialize>(
        &self,
        method: Method,
        path: &str,
        query: &[(&str, &str)],
        body: Option<&B>,
    ) -> Result<Response> {
        let url = format!("{}{}", self.base_url, path);

        let mut req = self.http.request(method.clone(), &url);

        // Auth headers — sign the path only (no query params)
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

        Ok(req.send().await?)
    }

    async fn parse_response<T: DeserializeOwned>(&self, resp: Response) -> Result<T> {
        let status = resp.status();

        if status.is_success() {
            let body = resp.text().await?;
            // Handle empty responses
            if body.is_empty() {
                // Try to deserialize from empty JSON object
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
                    status.canonical_reason().unwrap_or("Unknown error").to_string()
                } else {
                    body
                },
                code: None,
            });
        }
    }
}
