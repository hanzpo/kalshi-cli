use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use rsa::RsaPrivateKey;
use rsa::pkcs1::DecodeRsaPrivateKey;
use rsa::pkcs8::DecodePrivateKey;
use rsa::pss::BlindedSigningKey;
use rsa::rand_core::OsRng;
use rsa::signature::{RandomizedSigner, SignatureEncoding};
use sha2::Sha256;

pub struct KalshiSigner {
    signing_key: BlindedSigningKey<Sha256>,
    api_key_id: String,
}

impl KalshiSigner {
    pub fn new(api_key_id: String, pem_path: &Path) -> Result<Self> {
        let pem_contents = std::fs::read_to_string(pem_path)
            .with_context(|| format!("Failed to read private key from {}", pem_path.display()))?;

        Self::from_pem(api_key_id, &pem_contents)
    }

    pub fn from_pem(api_key_id: String, pem_contents: &str) -> Result<Self> {
        let private_key = RsaPrivateKey::from_pkcs8_pem(pem_contents)
            .or_else(|_| RsaPrivateKey::from_pkcs1_pem(pem_contents))
            .context("Failed to parse RSA private key (expected PKCS#8 or PKCS#1 PEM format)")?;

        let signing_key = BlindedSigningKey::<Sha256>::new(private_key);

        Ok(Self {
            signing_key,
            api_key_id,
        })
    }

    /// Sign a request and return (key_id, timestamp_ms, base64_signature).
    /// `method` should be uppercase (GET, POST, etc.).
    /// `path` should be the URL path WITHOUT query parameters (e.g. "/trade-api/v2/markets").
    pub fn sign_request(&self, method: &str, path: &str) -> Result<(String, String, String)> {
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_millis()
            .to_string();

        let message = format!("{}{}{}", timestamp_ms, method, path);
        let mut rng = OsRng;
        let signature = self.signing_key.sign_with_rng(&mut rng, message.as_bytes());
        let sig_b64 = BASE64.encode(signature.to_bytes());

        Ok((self.api_key_id.clone(), timestamp_ms, sig_b64))
    }
}
