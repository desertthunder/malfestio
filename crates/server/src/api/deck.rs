use crate::middleware::auth::UserContext;
use crate::state::SharedState;

use axum::{
    Json,
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use malfestio_core::model::{Deck, Visibility};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct CreateDeckRequest {
    title: String,
    description: String,
    tags: Vec<String>,
    visibility: Visibility,
}

#[derive(Deserialize)]
pub struct PublishDeckRequest {
    pub published: bool,
}

pub async fn create_deck(
    State(state): State<SharedState>, ctx: Option<axum::Extension<UserContext>>, Json(payload): Json<CreateDeckRequest>,
) -> impl IntoResponse {
    let user = match ctx {
        Some(axum::Extension(user)) => user,
        None => return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Unauthorized"}))).into_response(),
    };

    let new_deck = Deck {
        id: uuid::Uuid::new_v4().to_string(),
        owner_did: user.did,
        title: payload.title,
        description: payload.description,
        tags: payload.tags,
        visibility: payload.visibility,
        published_at: None,
        fork_of: None,
    };

    state.decks.write().unwrap().push(new_deck.clone());

    (StatusCode::CREATED, Json(new_deck)).into_response()
}

pub async fn list_decks(
    State(state): State<SharedState>, ctx: Option<axum::Extension<UserContext>>,
) -> impl IntoResponse {
    let user_did = ctx.map(|Extension(u)| u.did);

    let decks = state.decks.read().unwrap();

    let visible_decks: Vec<Deck> = decks
        .iter()
        .filter(|d| {
            if let Some(did) = &user_did
                && &d.owner_did == did
            {
                return true;
            }
            if d.visibility == Visibility::Public {
                return true;
            }
            if let Visibility::SharedWith(dids) = &d.visibility
                && let Some(did) = &user_did
                && dids.contains(did)
            {
                return true;
            }
            false
        })
        .cloned()
        .collect();

    Json(visible_decks).into_response()
}

pub async fn get_deck(
    State(state): State<SharedState>, ctx: Option<axum::Extension<UserContext>>, Path(id): Path<String>,
) -> impl IntoResponse {
    let user_did = ctx.map(|Extension(u)| u.did);
    let decks = state.decks.read().unwrap();

    if let Some(deck) = decks.iter().find(|d| d.id == id) {
        let is_owner = user_did.as_ref() == Some(&deck.owner_did);

        if deck.visibility == Visibility::Public || is_owner {
            return Json(deck).into_response();
        }

        if let Visibility::SharedWith(dids) = &deck.visibility
            && let Some(did) = &user_did
            && dids.contains(did)
        {
            return Json(deck).into_response();
        }

        if deck.visibility == Visibility::Unlisted {
            return Json(deck).into_response();
        }
        return (StatusCode::FORBIDDEN, Json(json!({"error": "Access denied"}))).into_response();
    }

    (StatusCode::NOT_FOUND, Json(json!({"error": "Deck not found"}))).into_response()
}

/// NOTE: Unpublishing sets visibility to Private and clears published_at
pub async fn publish_deck(
    State(state): State<SharedState>, ctx: Option<axum::Extension<UserContext>>, Path(id): Path<String>,
    Json(payload): Json<PublishDeckRequest>,
) -> impl IntoResponse {
    let user = match ctx {
        Some(axum::Extension(user)) => user,
        None => return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Unauthorized"}))).into_response(),
    };

    let mut decks = state.decks.write().unwrap();
    if let Some(deck) = decks.iter_mut().find(|d| d.id == id) {
        if deck.owner_did != user.did {
            return (StatusCode::FORBIDDEN, Json(json!({"error": "Only owner can publish"}))).into_response();
        }

        if payload.published {
            deck.visibility = Visibility::Public;
            deck.published_at = Some(chrono::Utc::now().to_rfc3339());
        } else {
            deck.visibility = Visibility::Private;
            deck.published_at = None;
        }
        return Json(deck.clone()).into_response();
    }

    (StatusCode::NOT_FOUND, Json(json!({"error": "Deck not found"}))).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;
    use axum::extract::State;

    fn mock_state() -> SharedState {
        let state = AppState::new();
        let mut decks = state.decks.write().unwrap();
        *decks = vec![
            Deck {
                id: "deck-public".to_string(),
                owner_did: "did:plc:owner".to_string(),
                title: "Public Deck".to_string(),
                description: "desc".to_string(),
                tags: vec![],
                visibility: Visibility::Public,
                published_at: None,
                fork_of: None,
            },
            Deck {
                id: "deck-private".to_string(),
                owner_did: "did:plc:owner".to_string(),
                title: "Private Deck".to_string(),
                description: "desc".to_string(),
                tags: vec![],
                visibility: Visibility::Private,
                published_at: None,
                fork_of: None,
            },
            Deck {
                id: "deck-shared".to_string(),
                owner_did: "did:plc:owner".to_string(),
                title: "Shared Deck".to_string(),
                description: "desc".to_string(),
                tags: vec![],
                visibility: Visibility::SharedWith(vec!["did:plc:friend".to_string()]),
                published_at: None,
                fork_of: None,
            },
        ];
        state.clone()
    }

    #[tokio::test]
    async fn test_get_public_deck() {
        let state = mock_state();
        let response = get_deck(State(state), None, Path("deck-public".to_string()))
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_private_deck_owner() {
        let state = mock_state();
        let ctx = Some(Extension(UserContext {
            did: "did:plc:owner".to_string(),
            handle: "owner.bsky.social".to_string(),
        }));

        let response = get_deck(State(state), ctx, Path("deck-private".to_string()))
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_private_deck_stranger() {
        let state = mock_state();
        let ctx = Some(Extension(UserContext {
            did: "did:plc:stranger".to_string(),
            handle: "stranger.bsky.social".to_string(),
        }));

        let response = get_deck(State(state), ctx, Path("deck-private".to_string()))
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_get_shared_deck_permitted() {
        let state = mock_state();
        let ctx = Some(Extension(UserContext {
            did: "did:plc:friend".to_string(),
            handle: "friend.bsky.social".to_string(),
        }));

        let response = get_deck(State(state), ctx, Path("deck-shared".to_string()))
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_shared_deck_unpermitted() {
        let state = mock_state();
        let ctx = Some(Extension(UserContext {
            did: "did:plc:stranger".to_string(),
            handle: "stranger.bsky.social".to_string(),
        }));

        let response = get_deck(State(state), ctx, Path("deck-shared".to_string()))
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_publish_deck() {
        let state = mock_state();
        let ctx = Some(Extension(UserContext {
            did: "did:plc:owner".to_string(),
            handle: "owner.bsky.social".to_string(),
        }));

        let response = publish_deck(
            State(state.clone()),
            ctx,
            Path("deck-private".to_string()),
            Json(PublishDeckRequest { published: true }),
        )
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);

        let decks = state.decks.read().unwrap();
        let deck = decks.iter().find(|d| d.id == "deck-private").unwrap();
        assert_eq!(deck.visibility, Visibility::Public);
        assert!(deck.published_at.is_some());
    }
}
