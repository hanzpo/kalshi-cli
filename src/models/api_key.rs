use serde::{Deserialize, Serialize};

use crate::models::common::format_opt;
use crate::output::TableDisplay;

#[derive(Debug, Serialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: Option<String>,
    pub name: Option<String>,
    pub created_time: Option<String>,
    pub status: Option<String>,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ApiKeysResponse {
    pub api_keys: Option<Vec<ApiKey>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiKeyResponse {
    pub api_key: Option<ApiKey>,
}

#[derive(Debug, Serialize)]
pub struct GenerateApiKeyRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<String>>,
}

impl TableDisplay for ApiKey {
    fn headers() -> Vec<&'static str> {
        vec!["ID", "Name", "Status", "Created"]
    }

    fn row(&self) -> Vec<String> {
        vec![
            format_opt(&self.id),
            format_opt(&self.name),
            format_opt(&self.status),
            format_opt(&self.created_time),
        ]
    }
}
