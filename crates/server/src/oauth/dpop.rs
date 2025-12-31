//! DPoP (Demonstrating Proof of Possession) implementation for OAuth 2.1.
//!
//! AT Protocol requires DPoP tokens to bind access tokens to specific clients.
//! This module provides both proof generation (for client use) and verification
//! (for server use) per RFC 9449.

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use malfestio_core::Error as CoreError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

/// Maximum allowed clock skew for DPoP proof validation (5 minutes).
const MAX_CLOCK_SKEW_SECS: u64 = 300;

/// Maximum age for a DPoP proof to be considered valid (5 minutes).
const MAX_PROOF_AGE_SECS: u64 = 300;

/// A DPoP keypair for proof generation using Ed25519.
#[derive(Clone)]
pub struct DpopKeypair {
    signing_key: SigningKey,
}

/// DPoP proof JWT header.
#[derive(Serialize, Deserialize, Debug)]
pub struct DpopHeader {
    pub typ: String,
    pub alg: String,
    pub jwk: DpopJwk,
}

/// JWK representation for DPoP (Ed25519 public key).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DpopJwk {
    pub kty: String,
    pub crv: String,
    pub x: String,
}

/// DPoP proof JWT payload.
#[derive(Serialize, Deserialize, Debug)]
pub struct DpopPayload {
    pub jti: String,
    pub htm: String,
    pub htu: String,
    pub iat: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ath: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
}

/// Parsed DPoP proof for verification.
#[derive(Debug)]
pub struct ParsedDpopProof {
    pub header: DpopHeader,
    pub payload: DpopPayload,
    pub signature: Vec<u8>,
    pub signing_input: String,
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
        self.generate_proof_with_nonce(method, url, access_token, None)
    }

    /// Generate a DPoP proof with an optional server-provided nonce.
    pub fn generate_proof_with_nonce(
        &self, method: &str, url: &str, access_token: Option<&str>, nonce: Option<&str>,
    ) -> String {
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

        let payload = DpopPayload {
            jti,
            htm: method.to_uppercase(),
            htu: url.to_string(),
            iat: now,
            ath,
            nonce: nonce.map(String::from),
        };

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

/// Generate a server nonce for DPoP.
pub fn generate_nonce() -> String {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).expect("Failed to generate random bytes");
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Parse a DPoP proof JWT into its components.
pub fn parse_proof(proof: &str) -> Result<ParsedDpopProof, CoreError> {
    let parts: Vec<&str> = proof.split('.').collect();
    if parts.len() != 3 {
        return Err(CoreError::DPoP("Invalid proof format: expected 3 parts".to_string()));
    }

    let header_json = URL_SAFE_NO_PAD
        .decode(parts[0])
        .map_err(|e| CoreError::DPoP(format!("Invalid header encoding: {}", e)))?;

    let header: DpopHeader =
        serde_json::from_slice(&header_json).map_err(|e| CoreError::DPoP(format!("Invalid header JSON: {}", e)))?;

    let payload_json = URL_SAFE_NO_PAD
        .decode(parts[1])
        .map_err(|e| CoreError::DPoP(format!("Invalid payload encoding: {}", e)))?;

    let payload: DpopPayload =
        serde_json::from_slice(&payload_json).map_err(|e| CoreError::DPoP(format!("Invalid payload JSON: {}", e)))?;

    let signature = URL_SAFE_NO_PAD
        .decode(parts[2])
        .map_err(|e| CoreError::DPoP(format!("Invalid signature encoding: {}", e)))?;

    let signing_input = format!("{}.{}", parts[0], parts[1]);

    Ok(ParsedDpopProof { header, payload, signature, signing_input })
}

/// Request context for DPoP verification.
pub struct DpopVerifyRequest<'a> {
    /// The DPoP proof JWT string
    pub proof: &'a str,
    /// Expected HTTP method (e.g., "GET", "POST")
    pub method: &'a str,
    /// Expected request URI (without query/fragment)
    pub uri: &'a str,
    /// The access token (for ath verification)
    pub access_token: Option<&'a str>,
    /// Expected server nonce (if required)
    pub expected_nonce: Option<&'a str>,
}

impl<'a> DpopVerifyRequest<'a> {
    pub fn new(
        proof: &'a str, method: &'a str, uri: &'a str, access_token: Option<&'a str>, expected_nonce: Option<&'a str>,
    ) -> Self {
        Self { proof, method, uri, access_token, expected_nonce }
    }
}

/// Verify a DPoP proof.
///
/// Returns the parsed proof if valid, with the JWK for binding verification.
pub fn verify_proof(req: DpopVerifyRequest<'_>) -> Result<ParsedDpopProof, CoreError> {
    let parsed = parse_proof(req.proof)?;

    if parsed.header.typ != "dpop+jwt" {
        return Err(CoreError::DPoP(format!(
            "Invalid typ: expected 'dpop+jwt', got '{}'",
            parsed.header.typ
        )));
    }

    if parsed.header.alg != "EdDSA" {
        return Err(CoreError::DPoP(format!(
            "Unsupported alg: expected 'EdDSA', got '{}'",
            parsed.header.alg
        )));
    }

    if parsed.header.jwk.kty != "OKP" || parsed.header.jwk.crv != "Ed25519" {
        return Err(CoreError::DPoP("Invalid JWK: expected Ed25519 key".to_string()));
    }
    let public_key_bytes = URL_SAFE_NO_PAD
        .decode(&parsed.header.jwk.x)
        .map_err(|e| CoreError::DPoP(format!("Invalid JWK x value: {}", e)))?;

    if public_key_bytes.len() != 32 {
        return Err(CoreError::DPoP("Invalid public key length".to_string()));
    }

    let mut key_bytes = [0u8; 32];
    key_bytes.copy_from_slice(&public_key_bytes);

    let verifying_key =
        VerifyingKey::from_bytes(&key_bytes).map_err(|e| CoreError::DPoP(format!("Invalid public key: {}", e)))?;

    let signature = Signature::from_slice(&parsed.signature)
        .map_err(|e| CoreError::DPoP(format!("Invalid signature format: {}", e)))?;

    verifying_key
        .verify(parsed.signing_input.as_bytes(), &signature)
        .map_err(|_| CoreError::DPoP("Signature verification failed".to_string()))?;

    if parsed.payload.htm.to_uppercase() != req.method.to_uppercase() {
        return Err(CoreError::DPoP(format!(
            "HTTP method mismatch: expected '{}', got '{}'",
            req.method, parsed.payload.htm
        )));
    }

    let expected_uri = normalize_uri(req.uri);
    let proof_uri = normalize_uri(&parsed.payload.htu);
    if expected_uri != proof_uri {
        return Err(CoreError::DPoP(format!(
            "URI mismatch: expected '{}', got '{}'",
            expected_uri, proof_uri
        )));
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    if parsed.payload.iat > now + MAX_CLOCK_SKEW_SECS {
        return Err(CoreError::DPoP("Proof issued in the future".to_string()));
    }

    if now > parsed.payload.iat + MAX_PROOF_AGE_SECS {
        return Err(CoreError::DPoP("Proof has expired".to_string()));
    }

    if let Some(access_token) = req.access_token {
        let expected_ath = {
            let hash = Sha256::digest(access_token.as_bytes());
            URL_SAFE_NO_PAD.encode(hash)
        };

        match &parsed.payload.ath {
            Some(ath) if ath == &expected_ath => {}
            Some(ath) => {
                return Err(CoreError::DPoP(format!(
                    "Access token hash mismatch: expected '{}', got '{}'",
                    expected_ath, ath
                )));
            }
            None => {
                return Err(CoreError::DPoP("Missing access token hash (ath) claim".to_string()));
            }
        }
    }

    if let Some(expected_nonce) = req.expected_nonce {
        match &parsed.payload.nonce {
            Some(nonce) if nonce == expected_nonce => {}
            Some(nonce) => {
                return Err(CoreError::DPoP(format!(
                    "Nonce mismatch: expected '{}', got '{}'",
                    expected_nonce, nonce
                )));
            }
            None => {
                return Err(CoreError::DPoP("Missing nonce claim".to_string()));
            }
        }
    }

    Ok(parsed)
}

/// Normalize a URI by removing query string and fragment.
fn normalize_uri(uri: &str) -> String {
    uri.split('?')
        .next()
        .unwrap_or(uri)
        .split('#')
        .next()
        .unwrap_or(uri)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

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

        let signature = Signature::from_slice(&signature_bytes).unwrap();
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

    #[test]
    fn test_generate_nonce() {
        let nonce1 = generate_nonce();
        let nonce2 = generate_nonce();

        assert_ne!(nonce1, nonce2);
        assert_eq!(nonce1.len(), 22); // 16 bytes base64url encoded
    }

    #[test]
    fn test_verify_proof_valid() {
        let kp = DpopKeypair::generate();
        let proof = kp.generate_proof("POST", "https://example.com/api", None);
        let req = DpopVerifyRequest::new(&proof, "POST", "https://example.com/api", None, None);
        let result = verify_proof(req);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_proof_invalid_signature() {
        let kp1 = DpopKeypair::generate();
        let kp2 = DpopKeypair::generate();

        let proof = kp1.generate_proof("POST", "https://example.com/api", None);
        let parts: Vec<&str> = proof.split('.').collect();

        let mut header: DpopHeader = serde_json::from_slice(&URL_SAFE_NO_PAD.decode(parts[0]).unwrap()).unwrap();
        header.jwk = kp2.public_jwk();

        let new_header_b64 = URL_SAFE_NO_PAD.encode(serde_json::to_string(&header).unwrap());
        let tampered_proof = format!("{}.{}.{}", new_header_b64, parts[1], parts[2]);
        let req = DpopVerifyRequest::new(&tampered_proof, "POST", "https://example.com/api", None, None);
        let result = verify_proof(req);

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Signature verification failed")
        );
    }

    #[test]
    fn test_verify_proof_wrong_method() {
        let kp = DpopKeypair::generate();
        let proof = kp.generate_proof("POST", "https://example.com/api", None);
        let req = DpopVerifyRequest::new(&proof, "GET", "https://example.com/api", None, None);
        let result = verify_proof(req);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("HTTP method mismatch"));
    }

    #[test]
    fn test_verify_proof_wrong_uri() {
        let kp = DpopKeypair::generate();
        let proof = kp.generate_proof("POST", "https://example.com/api", None);
        let req = DpopVerifyRequest::new(&proof, "POST", "https://example.com/other", None, None);
        let result = verify_proof(req);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("URI mismatch"));
    }

    #[test]
    fn test_verify_proof_with_token_hash() {
        let kp = DpopKeypair::generate();
        let token = "my_access_token_123";
        let proof = kp.generate_proof("GET", "https://example.com/resource", Some(token));
        let req = DpopVerifyRequest::new(&proof, "GET", "https://example.com/resource", Some(token), None);
        let result = verify_proof(req);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_proof_wrong_token_hash() {
        let kp = DpopKeypair::generate();
        let proof = kp.generate_proof("GET", "https://example.com/resource", Some("token_a"));
        let req = DpopVerifyRequest::new(&proof, "GET", "https://example.com/resource", Some("token_b"), None);
        let result = verify_proof(req);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Access token hash mismatch"));
    }

    #[test]
    fn test_verify_proof_with_nonce() {
        let kp = DpopKeypair::generate();
        let nonce = generate_nonce();
        let proof = kp.generate_proof_with_nonce("POST", "https://example.com/api", None, Some(&nonce));
        let req = DpopVerifyRequest::new(&proof, "POST", "https://example.com/api", None, Some(&nonce));
        let result = verify_proof(req);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_proof_wrong_nonce() {
        let kp = DpopKeypair::generate();
        let nonce1 = generate_nonce();
        let nonce2 = generate_nonce();
        let proof = kp.generate_proof_with_nonce("POST", "https://example.com/api", None, Some(&nonce1));
        let req = DpopVerifyRequest::new(&proof, "POST", "https://example.com/api", None, Some(&nonce2));
        let result = verify_proof(req);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Nonce mismatch"));
    }

    #[test]
    fn test_verify_proof_missing_nonce() {
        let kp = DpopKeypair::generate();
        let proof = kp.generate_proof("POST", "https://example.com/api", None); // No nonce
        let req = DpopVerifyRequest::new(&proof, "POST", "https://example.com/api", None, Some("required_nonce"));
        let result = verify_proof(req);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing nonce claim"));
    }

    #[test]
    fn test_normalize_uri() {
        assert_eq!(normalize_uri("https://example.com/api"), "https://example.com/api");
        assert_eq!(
            normalize_uri("https://example.com/api?foo=bar"),
            "https://example.com/api"
        );
        assert_eq!(
            normalize_uri("https://example.com/api#section"),
            "https://example.com/api"
        );
        assert_eq!(
            normalize_uri("https://example.com/api?foo=bar#section"),
            "https://example.com/api"
        );
    }

    #[test]
    fn test_parse_proof_invalid_format() {
        let result = parse_proof("not.a.valid.jwt.with.too.many.parts");
        assert!(result.is_err());

        let result = parse_proof("only.two");
        assert!(result.is_err());
    }
}
