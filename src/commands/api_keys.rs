use anyhow::Result;

use crate::cli::ApiKeyCmd;
use crate::client::KalshiClient;
use crate::models::api_key::{ApiKeyResponse, ApiKeysResponse, CreateApiKeyRequest};
use crate::output::{OutputConfig, output, print_json};

pub async fn execute(
    client: &KalshiClient,
    cmd: ApiKeyCmd,
    out: &OutputConfig,
) -> Result<()> {
    client.require_auth()?;

    match cmd {
        ApiKeyCmd::List => {
            let resp: ApiKeysResponse = client.get("/api_keys", &[]).await?;
            output(&resp.api_keys.unwrap_or_default(), out)?;
        }
        ApiKeyCmd::Create { name } => {
            let req = CreateApiKeyRequest { name };
            let resp: ApiKeyResponse = client.post("/api_keys", &req).await?;
            print_json(&resp, out.no_pager)?;
        }
        ApiKeyCmd::Delete { key_id } => {
            let path = format!("/api_keys/{}", key_id);
            let resp: serde_json::Value = client.delete(&path).await?;
            print_json(&resp, out.no_pager)?;
        }
    }
    Ok(())
}
