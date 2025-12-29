//! DPoP (Demonstrating Proof of Possession) implementation for OAuth 2.1.
//!
//! AT Protocol requires DPoP tokens to bind access tokens to specific clients.

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

/// A DPoP keypair for proof generation using Ed25519.
#[derive(Clone)]
pub struct DpopKeypair {
    signing_key: SigningKey,
}

/// DPoP proof JWT header.
#[derive(Serialize, Deserialize)]
struct DpopHeader {
    typ: String,
    alg: String,
    jwk: DpopJwk,
}

/// JWK representation for DPoP (Ed25519 public key).
#[derive(Serialize, Deserialize, Clone)]
pub struct DpopJwk {
    kty: String,
    crv: String,
    x: String,
}

/// DPoP proof JWT payload.
#[derive(Serialize, Deserialize)]
struct DpopPayload {
    jti: String,
    htm: String,
    htu: String,
    iat: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    ath: Option<String>,
}

impl DpopKeypair {
    /// Generate a new random Ed25519 DPoP keypair.
    pub fn generate() -> Self {
        let mut rng_bytes = [0u8; 32];
        getrandom::fill(&mut rng_bytes).expect("Failed to generate random bytes");
        let signing_key = SigningKey::from_bytes(&rng_bytes);
        Self { signing_key }
    }

    /// Get the verifying (public) key.
    pub fn verifying_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }

    /// Get the JWK representation of the public key.
    pub fn public_jwk(&self) -> DpopJwk {
        let public_bytes = self.verifying_key().to_bytes();
        DpopJwk { kty: "OKP".to_string(), crv: "Ed25519".to_string(), x: URL_SAFE_NO_PAD.encode(public_bytes) }
    }

    /// Generate a DPoP proof for a request.
    pub fn generate_proof(&self, method: &str, url: &str, access_token: Option<&str>) -> String {
        let header = DpopHeader { typ: "dpop+jwt".to_string(), alg: "EdDSA".to_string(), jwk: self.public_jwk() };

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        let jti = generate_jti();

        let ath = access_token.map(|token| {
            let hash = Sha256::digest(token.as_bytes());
            URL_SAFE_NO_PAD.encode(hash)
        });

        let payload = DpopPayload { jti, htm: method.to_uppercase(), htu: url.to_string(), iat: now, ath };

        let header_b64 = URL_SAFE_NO_PAD.encode(serde_json::to_string(&header).unwrap());
        let payload_b64 = URL_SAFE_NO_PAD.encode(serde_json::to_string(&payload).unwrap());

        let signing_input = format!("{}.{}", header_b64, payload_b64);

        let signature = self.signing_key.sign(signing_input.as_bytes());
        let signature_b64 = URL_SAFE_NO_PAD.encode(signature.to_bytes());

        format!("{}.{}.{}", header_b64, payload_b64, signature_b64)
    }

    /// Create a DpopKeypair from an existing SigningKey.
    pub fn from_signing_key(signing_key: SigningKey) -> Self {
        Self { signing_key }
    }

    /// Get the private key bytes for storage.
    pub fn private_key_bytes(&self) -> Vec<u8> {
        self.signing_key.to_bytes().to_vec()
    }
}

/// Generate a unique JWT ID.
fn generate_jti() -> String {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).expect("Failed to generate random bytes");
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Compute the JWK thumbprint for key binding.
pub fn jwk_thumbprint(jwk: &DpopJwk) -> String {
    let canonical = format!(r#"{{"crv":"{}","kty":"{}","x":"{}"}}"#, jwk.crv, jwk.kty, jwk.x);
    let hash = Sha256::digest(canonical.as_bytes());
    URL_SAFE_NO_PAD.encode(hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::Verifier;

    #[test]
    fn test_generate_keypair() {
        let kp = DpopKeypair::generate();
        let _ = kp.verifying_key();
    }

    #[test]
    fn test_keypair_uniqueness() {
        let kp1 = DpopKeypair::generate();
        let kp2 = DpopKeypair::generate();
        assert_ne!(kp1.verifying_key().to_bytes(), kp2.verifying_key().to_bytes());
    }

    #[test]
    fn test_public_jwk() {
        let kp = DpopKeypair::generate();
        let jwk = kp.public_jwk();

        assert_eq!(jwk.kty, "OKP");
        assert_eq!(jwk.crv, "Ed25519");
        assert!(!jwk.x.is_empty());
        assert_eq!(jwk.x.len(), 43);
    }

    #[test]
    fn test_generate_proof() {
        let kp = DpopKeypair::generate();
        let proof = kp.generate_proof("POST", "https://example.com/token", None);

        let parts: Vec<&str> = proof.split('.').collect();
        assert_eq!(parts.len(), 3);

        let header_json = URL_SAFE_NO_PAD.decode(parts[0]).unwrap();
        let header: serde_json::Value = serde_json::from_slice(&header_json).unwrap();
        assert_eq!(header["typ"], "dpop+jwt");
        assert_eq!(header["alg"], "EdDSA");
    }

    #[test]
    fn test_proof_signature_verifies() {
        let kp = DpopKeypair::generate();
        let proof = kp.generate_proof("GET", "https://example.com/resource", None);

        let parts: Vec<&str> = proof.split('.').collect();
        let signing_input = format!("{}.{}", parts[0], parts[1]);
        let signature_bytes = URL_SAFE_NO_PAD.decode(parts[2]).unwrap();

        let signature = ed25519_dalek::Signature::from_slice(&signature_bytes).unwrap();
        let result = kp.verifying_key().verify(signing_input.as_bytes(), &signature);

        assert!(result.is_ok(), "Signature should verify");
    }

    #[test]
    fn test_generate_proof_with_token() {
        let kp = DpopKeypair::generate();
        let proof = kp.generate_proof("GET", "https://example.com/resource", Some("access_token_123"));

        let parts: Vec<&str> = proof.split('.').collect();
        let payload_json = URL_SAFE_NO_PAD.decode(parts[1]).unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&payload_json).unwrap();

        assert!(payload.get("ath").is_some());
    }

    #[test]
    fn test_jwk_thumbprint() {
        let kp = DpopKeypair::generate();
        let jwk = kp.public_jwk();
        let thumbprint = jwk_thumbprint(&jwk);

        assert_eq!(thumbprint.len(), 43);
    }
}
