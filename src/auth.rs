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
        let sig_b64 = self.sign_message(message.as_bytes());

        Ok((self.api_key_id.clone(), timestamp_ms, sig_b64))
    }

    fn sign_message(&self, message: &[u8]) -> String {
        let mut rng = OsRng;
        let signature = self.signing_key.sign_with_rng(&mut rng, message);
        BASE64.encode(signature.to_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rsa::pkcs1::EncodeRsaPrivateKey;
    use rsa::pkcs8::EncodePrivateKey;
    use rsa::pkcs8::LineEnding;
    use rsa::pss::{Signature, VerifyingKey};
    use rsa::signature::Verifier;

    fn generate_test_key() -> RsaPrivateKey {
        let mut rng = OsRng;
        RsaPrivateKey::new(&mut rng, 2048).unwrap()
    }

    fn pkcs8_pem(key: &RsaPrivateKey) -> String {
        key.to_pkcs8_pem(LineEnding::LF).unwrap().to_string()
    }

    fn pkcs1_pem(key: &RsaPrivateKey) -> String {
        key.to_pkcs1_pem(LineEnding::LF).unwrap().to_string()
    }

    #[test]
    fn from_pem_accepts_pkcs8() {
        let key = generate_test_key();
        let pem = pkcs8_pem(&key);
        let signer = KalshiSigner::from_pem("key-123".to_string(), &pem).unwrap();
        assert_eq!(signer.api_key_id, "key-123");
    }

    #[test]
    fn from_pem_accepts_pkcs1() {
        let key = generate_test_key();
        let pem = pkcs1_pem(&key);
        KalshiSigner::from_pem("key-456".to_string(), &pem).unwrap();
    }

    #[test]
    fn from_pem_rejects_invalid_pem() {
        let result = KalshiSigner::from_pem("key".to_string(), "not a valid PEM");
        assert!(result.is_err());
        assert!(result.err().unwrap().to_string().contains("parse RSA"));
    }

    #[test]
    fn sign_request_returns_key_id_timestamp_and_signature() {
        let key = generate_test_key();
        let signer = KalshiSigner::from_pem("my-key".to_string(), &pkcs8_pem(&key)).unwrap();

        let (key_id, timestamp, signature) =
            signer.sign_request("GET", "/trade-api/v2/markets").unwrap();

        assert_eq!(key_id, "my-key");
        // Timestamp should be a valid integer (milliseconds since epoch)
        let ts: u128 = timestamp.parse().expect("timestamp should be numeric");
        assert!(ts > 1_000_000_000_000); // after 2001 in ms
        // Signature should be valid base64
        assert!(BASE64.decode(&signature).is_ok());
    }

    #[test]
    fn sign_request_produces_verifiable_signature() {
        let key = generate_test_key();
        let signer = KalshiSigner::from_pem("k".to_string(), &pkcs8_pem(&key)).unwrap();

        let (_, timestamp, sig_b64) =
            signer.sign_request("POST", "/trade-api/v2/portfolio/orders").unwrap();

        // Reconstruct the message that was signed
        let message = format!("{}POST/trade-api/v2/portfolio/orders", timestamp);
        let sig_bytes = BASE64.decode(&sig_b64).unwrap();
        let signature = Signature::try_from(sig_bytes.as_slice()).unwrap();

        // Verify using the public key
        let verifying_key = VerifyingKey::<Sha256>::new(key.to_public_key());
        assert!(verifying_key.verify(message.as_bytes(), &signature).is_ok());
    }

    #[test]
    fn sign_request_different_calls_produce_different_signatures() {
        let key = generate_test_key();
        let signer = KalshiSigner::from_pem("k".to_string(), &pkcs8_pem(&key)).unwrap();

        let (_, _, sig1) = signer.sign_request("GET", "/markets").unwrap();
        let (_, _, sig2) = signer.sign_request("GET", "/markets").unwrap();

        // PSS is randomized, so signatures should differ even for same input
        assert_ne!(sig1, sig2);
    }

    #[test]
    fn from_file_reads_pem_from_disk() {
        let key = generate_test_key();
        let pem = pkcs8_pem(&key);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.pem");
        std::fs::write(&path, &pem).unwrap();

        let signer = KalshiSigner::new("file-key".to_string(), &path);
        assert!(signer.is_ok());
    }

    #[test]
    fn from_file_nonexistent_path_errors() {
        let result = KalshiSigner::new("k".to_string(), std::path::Path::new("/no/such/key.pem"));
        assert!(result.is_err());
        assert!(result.err().unwrap().to_string().contains("read private key"));
    }
}
