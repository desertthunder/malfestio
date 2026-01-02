use crate::middleware::auth::UserContext;
use crate::state::SharedState;

use crate::repository::deck::{CreateDeckParams, DeckRepoError, UpdateDeckParams};
use axum::{
    Json,
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use malfestio_core::model::Visibility;
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

    let params = CreateDeckParams {
        owner_did: user.did,
        title: payload.title,
        description: payload.description,
        tags: payload.tags,
        visibility: payload.visibility,
    };

    match state.deck_repo.create(params).await {
        Ok(deck) => (StatusCode::CREATED, Json(deck)).into_response(),
        Err(e) => {
            tracing::error!("Failed to create deck: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to create deck"})),
            )
                .into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct RemoteDeckQuery {
    uri: String,
}

pub async fn fetch_remote_deck(
    State(state): State<SharedState>, Query(query): Query<RemoteDeckQuery>,
) -> impl IntoResponse {
    match state.deck_repo.get_remote_deck(&query.uri).await {
        Ok((deck, cards)) => Json(json!({ "deck": deck, "cards": cards })).into_response(),
        Err(DeckRepoError::NotFound(_)) => {
            (StatusCode::NOT_FOUND, Json(json!({"error": "Deck not found"}))).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to fetch remote deck: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to fetch remote deck"})),
            )
                .into_response()
        }
    }
}

pub async fn list_decks(
    State(state): State<SharedState>, ctx: Option<axum::Extension<UserContext>>,
) -> impl IntoResponse {
    let user_did = ctx.map(|Extension(u)| u.did);

    match state.deck_repo.list_visible(user_did.as_deref()).await {
        Ok(decks) => Json(decks).into_response(),
        Err(e) => {
            tracing::error!("Failed to list decks: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to list decks"})),
            )
                .into_response()
        }
    }
}

pub async fn get_deck(
    State(state): State<SharedState>, ctx: Option<axum::Extension<UserContext>>, Path(id): Path<String>,
) -> impl IntoResponse {
    let user_did = ctx.map(|Extension(u)| u.did);

    match state.deck_repo.get(&id).await {
        Ok(deck) => {
            let is_owner = user_did.as_ref() == Some(&deck.owner_did);
            let has_access = match &deck.visibility {
                Visibility::Public | Visibility::Unlisted => true,
                Visibility::Private => is_owner,
                Visibility::SharedWith(dids) => {
                    is_owner || user_did.as_ref().map(|did| dids.contains(did)).unwrap_or(false)
                }
            };

            if !has_access {
                return (StatusCode::FORBIDDEN, Json(json!({"error": "Access denied"}))).into_response();
            }

            Json(deck).into_response()
        }
        Err(crate::repository::deck::DeckRepoError::NotFound(_)) => {
            (StatusCode::NOT_FOUND, Json(json!({"error": "Deck not found"}))).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to get deck: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to retrieve deck"})),
            )
                .into_response()
        }
    }
}

pub async fn publish_deck(
    State(state): State<SharedState>, ctx: Option<axum::Extension<UserContext>>, Path(id): Path<String>,
    Json(payload): Json<PublishDeckRequest>,
) -> impl IntoResponse {
    let user = match ctx {
        Some(axum::Extension(user)) => user,
        None => return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Unauthorized"}))).into_response(),
    };

    let deck = match state.deck_repo.get(&id).await {
        Ok(d) => d,
        Err(DeckRepoError::NotFound(_)) => {
            return (StatusCode::NOT_FOUND, Json(json!({"error": "Deck not found"}))).into_response();
        }
        Err(e) => {
            tracing::error!("Failed to get deck: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to fetch deck"})),
            )
                .into_response();
        }
    };

    if deck.owner_did != user.did {
        return (StatusCode::FORBIDDEN, Json(json!({"error": "Only owner can publish"}))).into_response();
    }

    let mut updated_deck = deck.clone();
    let mut deck_at_uri = None;

    if payload.published {
        let cards = match state.card_repo.list_by_deck(&id).await {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Failed to fetch cards: {:?}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to fetch cards"})),
                )
                    .into_response();
            }
        };

        match crate::pds::publish::publish_deck_to_pds(state.oauth_repo.clone(), &user, &deck, &cards).await {
            Ok(result) => {
                deck_at_uri = Some(result.deck_at_uri.clone());

                let params = UpdateDeckParams {
                    deck_id: id.clone(),
                    visibility: Some(Visibility::Public),
                    published_at: Some(chrono::Utc::now().to_rfc3339()),
                    at_uri: Some(result.deck_at_uri.clone()),
                };

                if let Err(e) = state.deck_repo.update(params).await {
                    tracing::error!("Failed to update deck: {:?}", e);
                }

                updated_deck.visibility = Visibility::Public;
                updated_deck.published_at = Some(chrono::Utc::now().to_rfc3339());

                for (i, at_uri) in result.card_at_uris.iter().enumerate() {
                    if i < cards.len()
                        && let Err(e) = state.card_repo.update_at_uri(&cards[i].id, at_uri).await
                    {
                        tracing::warn!("Failed to store card AT-URI: {:?}", e);
                    }
                }
            }
            Err(e) => {
                tracing::error!("Failed to publish to PDS: {}", e);
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(json!({"error": format!("Failed to publish to PDS: {}", e)})),
                )
                    .into_response();
            }
        }
    } else {
        let params = UpdateDeckParams {
            deck_id: id.clone(),
            visibility: Some(Visibility::Private),
            published_at: None,
            at_uri: None,
        };
        if let Err(e) = state.deck_repo.update(params).await {
            tracing::error!("Failed to update deck: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to update deck"})),
            )
                .into_response();
        }
        updated_deck.visibility = Visibility::Private;
        updated_deck.published_at = None;
    }

    if let Some(at_uri) = deck_at_uri {
        Json(json!({
            "deck": updated_deck,
            "at_uri": at_uri
        }))
        .into_response()
    } else {
        Json(updated_deck).into_response()
    }
}

pub async fn fork_deck(
    State(state): State<SharedState>, ctx: Option<axum::Extension<UserContext>>, Path(id): Path<String>,
) -> impl IntoResponse {
    let user = match ctx {
        Some(axum::Extension(user)) => user,
        None => return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Unauthorized"}))).into_response(),
    };

    match state.deck_repo.fork(&id, &user.did).await {
        Ok(deck) => (StatusCode::CREATED, Json(deck)).into_response(),
        Err(crate::repository::deck::DeckRepoError::NotFound(_)) => {
            (StatusCode::NOT_FOUND, Json(json!({"error": "Deck not found"}))).into_response()
        }
        Err(crate::repository::deck::DeckRepoError::AccessDenied(_)) => (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "Cannot fork private deck"})),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to fork deck: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to fork deck"})),
            )
                .into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::middleware::auth::UserContext;
    use crate::state::AppState;
    use axum::extract::{Extension, State};
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use malfestio_core::model::Visibility;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_create_deck_success() {
        let pool = crate::db::create_pool("postgres://postgres:postgres@localhost:5432/malfestio").unwrap();

        let state = AppState::new_with_repos(
            pool,
            Arc::new(crate::repository::card::mock::MockCardRepository::new()),
            Arc::new(crate::repository::note::mock::MockNoteRepository::new()),
            Arc::new(crate::repository::oauth::mock::MockOAuthRepository::new()),
        );

        let user = UserContext {
            did: "did:plc:alice".to_string(),
            handle: "alice.bsky.social".to_string(),
            access_token: "test_token".to_string(),
            pds_url: "https://bsky.social".to_string(),
            has_dpop: false,
        };

        let payload = CreateDeckRequest {
            title: "My New Deck".to_string(),
            description: "A test deck".to_string(),
            tags: vec!["rust".to_string()],
            visibility: Visibility::Public,
        };

        let response = create_deck(State(state), Some(Extension(user)), Json(payload))
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::CREATED);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(body_json["title"], "My New Deck");
        assert_eq!(body_json["owner_did"], "did:plc:alice");
        assert_eq!(body_json["visibility"]["type"], "Public");
    }

    #[tokio::test]
    async fn test_create_deck_unauthorized() {
        let pool = crate::db::create_pool("postgres://postgres:postgres@localhost:5432/malfestio").unwrap();
        let state = AppState::new_with_repos(
            pool,
            Arc::new(crate::repository::card::mock::MockCardRepository::new()),
            Arc::new(crate::repository::note::mock::MockNoteRepository::new()),
            Arc::new(crate::repository::oauth::mock::MockOAuthRepository::new()),
        );

        let payload = CreateDeckRequest {
            title: "My New Deck".to_string(),
            description: "A test deck".to_string(),
            tags: vec![],
            visibility: Visibility::Public,
        };

        let response = create_deck(State(state), None, Json(payload)).await.into_response();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
