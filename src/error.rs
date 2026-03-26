use serde::Deserialize;

#[derive(Debug, thiserror::Error)]
pub enum KalshiError {
    #[error("API error ({status}): {message}")]
    Api {
        status: u16,
        message: String,
        code: Option<String>,
    },

    #[error("Authentication required. Run `kalshi config init` to set up your API key.")]
    AuthRequired,
}

#[derive(Debug, Deserialize)]
pub struct ApiErrorResponse {
    pub code: Option<String>,
    pub message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_error_display() {
        let err = KalshiError::Api {
            status: 404,
            message: "Not found".to_string(),
            code: Some("NOT_FOUND".to_string()),
        };
        let display = format!("{}", err);
        assert_eq!(display, "API error (404): Not found");
    }

    #[test]
    fn test_api_error_display_no_code() {
        let err = KalshiError::Api {
            status: 500,
            message: "Internal error".to_string(),
            code: None,
        };
        let display = format!("{}", err);
        assert_eq!(display, "API error (500): Internal error");
    }

    #[test]
    fn test_auth_required_display() {
        let err = KalshiError::AuthRequired;
        let display = format!("{}", err);
        assert!(display.contains("Authentication required"));
        assert!(display.contains("kalshi config init"));
    }

    #[test]
    fn test_api_error_response_deserialization_full() {
        let json = r#"{"code": "UNAUTHORIZED", "message": "Invalid token"}"#;
        let resp: ApiErrorResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.code, Some("UNAUTHORIZED".to_string()));
        assert_eq!(resp.message, Some("Invalid token".to_string()));
    }

    #[test]
    fn test_api_error_response_deserialization_partial() {
        let json = r#"{"message": "Something went wrong"}"#;
        let resp: ApiErrorResponse = serde_json::from_str(json).unwrap();
        assert!(resp.code.is_none());
        assert_eq!(resp.message, Some("Something went wrong".to_string()));
    }

    #[test]
    fn test_api_error_response_deserialization_empty() {
        let json = r#"{}"#;
        let resp: ApiErrorResponse = serde_json::from_str(json).unwrap();
        assert!(resp.code.is_none());
        assert!(resp.message.is_none());
    }

    #[test]
    fn test_api_error_is_error_trait() {
        let err = KalshiError::Api {
            status: 400,
            message: "Bad request".to_string(),
            code: None,
        };
        // Verify it implements std::error::Error
        let _: &dyn std::error::Error = &err;
    }
}
