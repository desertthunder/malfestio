//! PKCE (Proof Key for Code Exchange) implementation for OAuth 2.1.
//!
//! AT Protocol requires PKCE with S256 challenge method.

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use sha2::{Digest, Sha256};

/// Length of the code verifier in bytes (before base64 encoding).
const CODE_VERIFIER_LENGTH: usize = 32;

/// Generate a cryptographically random code verifier.
///
/// The verifier is a high-entropy random string used in PKCE flow.
pub fn generate_code_verifier() -> String {
    let mut bytes = [0u8; CODE_VERIFIER_LENGTH];
    getrandom::fill(&mut bytes).expect("Failed to generate random bytes");
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Derive the S256 code challenge from a code verifier.
///
/// The challenge is the base64url-encoded SHA-256 hash of the verifier.
pub fn derive_code_challenge(verifier: &str) -> String {
    let hash = Sha256::digest(verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(hash)
}

/// Verify that a code challenge matches a code verifier.
pub fn verify_challenge(verifier: &str, challenge: &str) -> bool {
    derive_code_challenge(verifier) == challenge
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_verifier_length() {
        let verifier = generate_code_verifier();
        assert_eq!(verifier.len(), 43);
    }

    #[test]
    fn test_generate_verifier_uniqueness() {
        let v1 = generate_code_verifier();
        let v2 = generate_code_verifier();
        assert_ne!(v1, v2);
    }

    #[test]
    fn test_challenge_derivation() {
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let challenge = derive_code_challenge(verifier);

        assert!(!challenge.is_empty());
        assert_eq!(challenge.len(), 43);
    }

    #[test]
    fn test_verify_challenge() {
        let verifier = generate_code_verifier();
        let challenge = derive_code_challenge(&verifier);

        assert!(verify_challenge(&verifier, &challenge));
        assert!(!verify_challenge(&verifier, "wrong_challenge"));
    }

    #[test]
    fn test_challenge_is_url_safe() {
        let verifier = generate_code_verifier();
        let challenge = derive_code_challenge(&verifier);
        assert!(!challenge.contains('+'));
        assert!(!challenge.contains('/'));
        assert!(!challenge.contains('='));
    }
}
