//! Publishing service for AT Protocol PDS operations.
//!
//! Encapsulates the logic for publishing records to a user's PDS.

use crate::middleware::auth::UserContext;
use crate::pds::client::{PdsClient, PdsError};
use crate::pds::records::{prepare_card_record, prepare_deck_record};
use crate::repository::oauth::{OAuthRepoError, OAuthRepository};
use malfestio_core::model::{Card, Deck};
use std::sync::Arc;

/// Error type for publishing operations.
#[derive(Debug)]
pub enum PublishError {
    /// User has no stored OAuth tokens
    NoTokens(String),
    /// OAuth token retrieval failed
    TokenError(String),
    /// Invalid DPoP keypair
    InvalidKeypair,
    /// PDS operation failed
    PdsError(PdsError),
    /// Database error storing AT-URI
    DatabaseError(String),
}

impl std::fmt::Display for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PublishError::NoTokens(did) => write!(f, "No OAuth tokens for DID: {}", did),
            PublishError::TokenError(e) => write!(f, "Token error: {}", e),
            PublishError::InvalidKeypair => write!(f, "Invalid DPoP keypair"),
            PublishError::PdsError(e) => write!(f, "PDS error: {}", e),
            PublishError::DatabaseError(e) => write!(f, "Database error: {}", e),
        }
    }
}

impl std::error::Error for PublishError {}

impl From<OAuthRepoError> for PublishError {
    fn from(e: OAuthRepoError) -> Self {
        match e {
            OAuthRepoError::NotFound(did) => PublishError::NoTokens(did),
            _ => PublishError::TokenError(e.to_string()),
        }
    }
}

impl From<PdsError> for PublishError {
    fn from(e: PdsError) -> Self {
        PublishError::PdsError(e)
    }
}

/// Result of publishing a deck to PDS.
pub struct PublishDeckResult {
    /// The AT-URI of the published deck
    pub deck_at_uri: String,
    /// The AT-URIs of the published cards
    pub card_at_uris: Vec<String>,
}

/// Publish a deck and its cards to the user's PDS.
///
/// This function:
/// 1. Tries to use OAuth tokens with DPoP if available
/// 2. Falls back to current session (supports app passwords with Bearer auth)
/// 3. Creates a PDS client with appropriate authentication
/// 4. Publishes each card (with placeholder deck ref initially)
/// 5. Publishes the deck with card AT-URIs
///
/// Note: Cards are published with an empty deck_ref since we don't have the
/// deck's AT-URI yet. This is acceptable per the Lexicon - the deck holds
/// the authoritative list of card references.
pub async fn publish_deck_to_pds(
    oauth_repo: Arc<dyn OAuthRepository>, user_ctx: &UserContext, deck: &Deck, cards: &[Card],
) -> Result<PublishDeckResult, PublishError> {
    let pds_client = if user_ctx.has_dpop {
        if let Ok(stored_token) = oauth_repo.get_tokens(&user_ctx.did).await {
            if let Some(dpop_keypair) = stored_token.dpop_keypair() {
                tracing::info!("Using stored OAuth tokens with DPoP for publishing");
                PdsClient::new_with_dpop(
                    stored_token.pds_url.clone(),
                    stored_token.access_token.clone(),
                    dpop_keypair,
                )
            } else {
                tracing::info!(
                    "Current session has DPoP flag but stored token lacks keypair, using current session with Bearer auth"
                );
                PdsClient::new_bearer(user_ctx.pds_url.clone(), user_ctx.access_token.clone())
            }
        } else {
            tracing::info!(
                "Current session has DPoP flag but no stored tokens found, using current session with Bearer auth"
            );
            PdsClient::new_bearer(user_ctx.pds_url.clone(), user_ctx.access_token.clone())
        }
    } else {
        tracing::info!("Using current session with Bearer auth for publishing (app password)");
        PdsClient::new_bearer(user_ctx.pds_url.clone(), user_ctx.access_token.clone())
    };

    let mut card_at_uris = Vec::with_capacity(cards.len());
    for card in cards {
        let prepared = prepare_card_record(card, "");
        let at_uri = pds_client
            .put_record(&user_ctx.did, &prepared.collection, &prepared.rkey, prepared.record)
            .await?;
        card_at_uris.push(at_uri.to_string());
    }

    let prepared = prepare_deck_record(deck, card_at_uris.clone());
    let deck_at_uri = pds_client
        .put_record(&user_ctx.did, &prepared.collection, &prepared.rkey, prepared.record)
        .await?;

    Ok(PublishDeckResult { deck_at_uri: deck_at_uri.to_string(), card_at_uris })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_publish_error_display() {
        let err = PublishError::NoTokens("did:plc:test".to_string());
        assert!(err.to_string().contains("did:plc:test"));

        let err = PublishError::InvalidKeypair;
        assert!(err.to_string().contains("Invalid DPoP keypair"));
    }

    #[test]
    fn test_publish_error_from_oauth_error() {
        let oauth_err = OAuthRepoError::NotFound("did:plc:test".to_string());
        let publish_err: PublishError = oauth_err.into();
        assert!(matches!(publish_err, PublishError::NoTokens(_)));
    }
}
