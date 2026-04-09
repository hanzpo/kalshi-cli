use serde::Deserialize;

#[derive(Debug)]
pub enum KalshiError {
    Api {
        status: u16,
        message: String,
        code: Option<String>,
    },
    AuthRequired,
}

impl std::fmt::Display for KalshiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KalshiError::Api {
                status,
                message,
                code,
            } => match code {
                Some(c) => write!(f, "API error ({status}): {message} [{c}]"),
                None => write!(f, "API error ({status}): {message}"),
            },
            KalshiError::AuthRequired => {
                write!(
                    f,
                    "Authentication required. Run `kalshi config init` to set up your API key."
                )
            }
        }
    }
}

impl std::error::Error for KalshiError {}

#[derive(Debug, Deserialize)]
pub struct ApiErrorInner {
    pub code: Option<String>,
    pub message: Option<String>,
    pub details: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ApiErrorResponse {
    /// Nested error object (e.g. {"error": {"code": "...", "message": "..."}})
    pub error: Option<ApiErrorInner>,
    /// Top-level fields (older API format)
    pub code: Option<String>,
    pub message: Option<String>,
    /// Short message field (e.g. {"msg": "..."})
    pub msg: Option<String>,
}

impl ApiErrorResponse {
    /// Extract the best available error code and message, regardless of nesting.
    pub fn into_parts(self) -> (Option<String>, Option<String>) {
        if let Some(inner) = self.error {
            let message = inner.message.or(self.message).or(self.msg);
            let message = match (message, inner.details) {
                (Some(m), Some(d)) => Some(format!("{m}: {d}")),
                (Some(m), None) => Some(m),
                (None, Some(d)) => Some(d),
                (None, None) => None,
            };
            (inner.code.or(self.code), message)
        } else {
            (self.code, self.message.or(self.msg))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_error_display_with_code() {
        let err = KalshiError::Api {
            status: 404,
            message: "Not found".to_string(),
            code: Some("NOT_FOUND".to_string()),
        };
        assert_eq!(format!("{}", err), "API error (404): Not found [NOT_FOUND]");
    }

    #[test]
    fn test_api_error_display_no_code() {
        let err = KalshiError::Api {
            status: 500,
            message: "Internal error".to_string(),
            code: None,
        };
        assert_eq!(format!("{}", err), "API error (500): Internal error");
    }

    #[test]
    fn test_auth_required_display() {
        let err = KalshiError::AuthRequired;
        let display = format!("{}", err);
        assert!(display.contains("Authentication required"));
        assert!(display.contains("kalshi config init"));
    }

    #[test]
    fn test_api_error_response_top_level() {
        let json = r#"{"code": "UNAUTHORIZED", "message": "Invalid token"}"#;
        let resp: ApiErrorResponse = serde_json::from_str(json).unwrap();
        let (code, message) = resp.into_parts();
        assert_eq!(code, Some("UNAUTHORIZED".to_string()));
        assert_eq!(message, Some("Invalid token".to_string()));
    }

    #[test]
    fn test_api_error_response_nested() {
        let json = r#"{"error": {"code": "invalid_parameters", "message": "invalid parameters"}}"#;
        let resp: ApiErrorResponse = serde_json::from_str(json).unwrap();
        let (code, message) = resp.into_parts();
        assert_eq!(code, Some("invalid_parameters".to_string()));
        assert_eq!(message, Some("invalid parameters".to_string()));
    }

    #[test]
    fn test_api_error_response_empty() {
        let json = r#"{}"#;
        let resp: ApiErrorResponse = serde_json::from_str(json).unwrap();
        let (code, message) = resp.into_parts();
        assert!(code.is_none());
        assert!(message.is_none());
    }

    #[test]
    fn test_api_error_response_partial() {
        let json = r#"{"message": "Something went wrong"}"#;
        let resp: ApiErrorResponse = serde_json::from_str(json).unwrap();
        let (code, message) = resp.into_parts();
        assert!(code.is_none());
        assert_eq!(message, Some("Something went wrong".to_string()));
    }

    #[test]
    fn test_api_error_is_error_trait() {
        let err = KalshiError::Api {
            status: 400,
            message: "Bad request".to_string(),
            code: None,
        };
        let _: &dyn std::error::Error = &err;
    }
}
