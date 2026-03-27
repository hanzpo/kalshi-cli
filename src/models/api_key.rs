use serde::{Deserialize, Serialize};

use crate::color;
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

    fn colored_row(&self, c: bool) -> Vec<String> {
        let mut row = self.row();
        if let Some(ref status) = self.status {
            row[2] = color::color_status(status, c);
        }
        row
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_headers() {
        let headers = ApiKey::headers();
        assert_eq!(headers, vec!["ID", "Name", "Status", "Created"]);
    }

    #[test]
    fn test_api_key_row() {
        let key = ApiKey {
            id: Some("key-123".to_string()),
            name: Some("my-key".to_string()),
            created_time: Some("2026-01-01".to_string()),
            status: Some("active".to_string()),
            extra: std::collections::HashMap::new(),
        };
        let row = key.row();
        assert_eq!(row[0], "key-123");
        assert_eq!(row[1], "my-key");
        assert_eq!(row[2], "active");
        assert_eq!(row[3], "2026-01-01");
    }

    #[test]
    fn test_api_key_row_all_none() {
        let key = ApiKey {
            id: None,
            name: None,
            created_time: None,
            status: None,
            extra: std::collections::HashMap::new(),
        };
        let row = key.row();
        for cell in &row {
            assert_eq!(cell, "-");
        }
    }

    #[test]
    fn test_generate_api_key_request_scopes_none_omitted() {
        let req = GenerateApiKeyRequest {
            name: "test-key".to_string(),
            scopes: None,
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["name"], "test-key");
        assert!(json.get("scopes").is_none());
    }

    #[test]
    fn test_generate_api_key_request_scopes_present() {
        let req = GenerateApiKeyRequest {
            name: "test-key".to_string(),
            scopes: Some(vec!["read".to_string(), "write".to_string()]),
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["name"], "test-key");
        let scopes = json["scopes"].as_array().unwrap();
        assert_eq!(scopes.len(), 2);
        assert_eq!(scopes[0], "read");
        assert_eq!(scopes[1], "write");
    }

    #[test]
    fn test_generate_api_key_request_empty_scopes() {
        let req = GenerateApiKeyRequest {
            name: "key".to_string(),
            scopes: Some(vec![]),
        };
        let json = serde_json::to_value(&req).unwrap();
        assert!(json["scopes"].as_array().unwrap().is_empty());
    }
}
