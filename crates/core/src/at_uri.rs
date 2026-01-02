//! AT-URI builder and parser for AT Protocol.
//!
//! AT-URIs are the canonical way to reference records in the AT Protocol.
//! Format: `at://<authority>/<collection>/<rkey>`
//!
//! - authority: DID or handle
//! - collection: NSID (e.g., "org.stormlightlabs.malfestio.deck")
//! - rkey: Record key (usually a TID)

use std::fmt;

/// An AT-URI representing a record in the AT Protocol network.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AtUri {
    /// The authority (DID or handle)
    pub authority: String,
    /// The collection NSID (e.g., "org.stormlightlabs.malfestio.deck")
    pub collection: String,
    /// The record key
    pub rkey: String,
}

impl AtUri {
    /// Create a new AT-URI.
    ///
    /// # Arguments
    ///
    /// * `authority` - The DID or handle
    /// * `collection` - The collection NSID
    /// * `rkey` - The record key
    pub fn new(authority: impl Into<String>, collection: impl Into<String>, rkey: impl Into<String>) -> Self {
        Self { authority: authority.into(), collection: collection.into(), rkey: rkey.into() }
    }

    /// Create an AT-URI for a deck record.
    pub fn deck(did: &str, rkey: &str) -> Self {
        Self::new(did, "org.stormlightlabs.malfestio.deck", rkey)
    }

    /// Create an AT-URI for a card record.
    pub fn card(did: &str, rkey: &str) -> Self {
        Self::new(did, "org.stormlightlabs.malfestio.card", rkey)
    }

    /// Create an AT-URI for a note record.
    pub fn note(did: &str, rkey: &str) -> Self {
        Self::new(did, "org.stormlightlabs.malfestio.note", rkey)
    }

    /// Parse an AT-URI string.
    pub fn parse(s: &str) -> Result<Self, AtUriError> {
        let s = s.strip_prefix("at://").ok_or(AtUriError::MissingScheme)?;

        let parts: Vec<&str> = s.splitn(3, '/').collect();
        if parts.len() != 3 {
            return Err(AtUriError::InvalidFormat);
        }

        let authority = parts[0];
        let collection = parts[1];
        let rkey = parts[2];

        if authority.is_empty() {
            return Err(AtUriError::EmptyAuthority);
        }
        if collection.is_empty() {
            return Err(AtUriError::EmptyCollection);
        }
        if rkey.is_empty() {
            return Err(AtUriError::EmptyRkey);
        }

        if !collection.contains('.') {
            return Err(AtUriError::InvalidNsid);
        }

        Ok(Self { authority: authority.to_string(), collection: collection.to_string(), rkey: rkey.to_string() })
    }

    /// Check if the authority is a DID.
    pub fn is_did(&self) -> bool {
        self.authority.starts_with("did:")
    }

    /// Check if the authority is a handle.
    pub fn is_handle(&self) -> bool {
        !self.is_did()
    }
}

impl fmt::Display for AtUri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "at://{}/{}/{}", self.authority, self.collection, self.rkey)
    }
}

/// Error type for AT-URI parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AtUriError {
    MissingScheme,
    InvalidFormat,
    EmptyAuthority,
    EmptyCollection,
    EmptyRkey,
    InvalidNsid,
}

impl fmt::Display for AtUriError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AtUriError::MissingScheme => write!(f, "AT-URI must start with 'at://'"),
            AtUriError::InvalidFormat => write!(f, "AT-URI must have format at://authority/collection/rkey"),
            AtUriError::EmptyAuthority => write!(f, "AT-URI authority cannot be empty"),
            AtUriError::EmptyCollection => write!(f, "AT-URI collection cannot be empty"),
            AtUriError::EmptyRkey => write!(f, "AT-URI rkey cannot be empty"),
            AtUriError::InvalidNsid => write!(f, "Collection must be a valid NSID"),
        }
    }
}

impl std::error::Error for AtUriError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_at_uri() {
        let uri = AtUri::new("did:plc:abc123", "org.stormlightlabs.malfestio.deck", "3k5abc123");
        assert_eq!(uri.authority, "did:plc:abc123");
        assert_eq!(uri.collection, "org.stormlightlabs.malfestio.deck");
        assert_eq!(uri.rkey, "3k5abc123");
    }

    #[test]
    fn test_display() {
        let uri = AtUri::new("did:plc:abc123", "org.stormlightlabs.malfestio.deck", "3k5abc123");
        assert_eq!(
            uri.to_string(),
            "at://did:plc:abc123/org.stormlightlabs.malfestio.deck/3k5abc123"
        );
    }

    #[test]
    fn test_parse_valid() {
        let uri = AtUri::parse("at://did:plc:abc123/org.stormlightlabs.malfestio.deck/3k5abc123").unwrap();
        assert_eq!(uri.authority, "did:plc:abc123");
        assert_eq!(uri.collection, "org.stormlightlabs.malfestio.deck");
        assert_eq!(uri.rkey, "3k5abc123");
    }

    #[test]
    fn test_parse_with_handle() {
        let uri = AtUri::parse("at://alice.bsky.social/org.stormlightlabs.malfestio.note/abc123").unwrap();
        assert_eq!(uri.authority, "alice.bsky.social");
        assert!(uri.is_handle());
        assert!(!uri.is_did());
    }

    #[test]
    fn test_parse_missing_scheme() {
        let result = AtUri::parse("did:plc:abc123/org.stormlightlabs.malfestio.deck/3k5abc123");
        assert_eq!(result, Err(AtUriError::MissingScheme));
    }

    #[test]
    fn test_parse_invalid_format() {
        let result = AtUri::parse("at://did:plc:abc123/org.stormlightlabs.malfestio.deck");
        assert_eq!(result, Err(AtUriError::InvalidFormat));
    }

    #[test]
    fn test_parse_empty_authority() {
        let result = AtUri::parse("at:///org.stormlightlabs.malfestio.deck/rkey");
        assert_eq!(result, Err(AtUriError::EmptyAuthority));
    }

    #[test]
    fn test_parse_invalid_nsid() {
        let result = AtUri::parse("at://did:plc:abc123/notansid/rkey");
        assert_eq!(result, Err(AtUriError::InvalidNsid));
    }

    #[test]
    fn test_roundtrip() {
        let original = "at://did:plc:abc123/org.stormlightlabs.malfestio.deck/3k5abc123";
        let uri = AtUri::parse(original).unwrap();
        assert_eq!(uri.to_string(), original);
    }

    #[test]
    fn test_convenience_constructors() {
        let deck = AtUri::deck("did:plc:abc", "tid123");
        assert_eq!(deck.collection, "org.stormlightlabs.malfestio.deck");

        let card = AtUri::card("did:plc:abc", "tid456");
        assert_eq!(card.collection, "org.stormlightlabs.malfestio.card");

        let note = AtUri::note("did:plc:abc", "tid789");
        assert_eq!(note.collection, "org.stormlightlabs.malfestio.note");
    }

    #[test]
    fn test_is_did() {
        let uri = AtUri::new("did:plc:abc123", "app.test", "rkey");
        assert!(uri.is_did());

        let uri = AtUri::new("alice.bsky.social", "app.test", "rkey");
        assert!(!uri.is_did());
    }
}
