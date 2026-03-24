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
