use crate::middleware::auth::UserContext;
use crate::repository::card::CardRepoError;
use crate::state::SharedState;

use axum::{
    Json,
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use malfestio_core::model::CardType;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct CreateCardRequest {
    deck_id: String,
    front: String,
    back: String,
    media_url: Option<String>,
    #[serde(default)]
    card_type: CardType,
    #[serde(default)]
    hints: Vec<String>,
}

pub async fn create_card(
    State(state): State<SharedState>, ctx: Option<Extension<UserContext>>, Json(payload): Json<CreateCardRequest>,
) -> impl IntoResponse {
    let user = match ctx {
        Some(Extension(user)) => user,
        None => return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Unauthorized"}))).into_response(),
    };

    let result = state
        .card_repo
        .create(crate::repository::card::CreateCardParams {
            owner_did: user.did.clone(),
            deck_id: payload.deck_id,
            front: payload.front,
            back: payload.back,
            media_url: payload.media_url,
            card_type: payload.card_type,
            hints: payload.hints,
        })
        .await;

    match result {
        Ok(card) => (StatusCode::CREATED, Json(card)).into_response(),
        Err(CardRepoError::NotFound(msg)) => (StatusCode::NOT_FOUND, Json(json!({"error": msg}))).into_response(),
        Err(CardRepoError::InvalidArgument(msg)) => {
            (StatusCode::BAD_REQUEST, Json(json!({"error": msg}))).into_response()
        }
        Err(CardRepoError::DatabaseError(msg)) => {
            tracing::error!("Database error: {}", msg);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to create card"})),
            )
                .into_response()
        }
    }
}

pub async fn list_cards(
    State(state): State<SharedState>, _ctx: Option<Extension<UserContext>>, Path(deck_id): Path<String>,
) -> impl IntoResponse {
    let result = state.card_repo.list_by_deck(&deck_id).await;

    match result {
        Ok(cards) => Json(cards).into_response(),
        Err(CardRepoError::NotFound(msg)) => (StatusCode::NOT_FOUND, Json(json!({"error": msg}))).into_response(),
        Err(CardRepoError::InvalidArgument(msg)) => {
            (StatusCode::BAD_REQUEST, Json(json!({"error": msg}))).into_response()
        }
        Err(CardRepoError::DatabaseError(msg)) => {
            tracing::error!("Database error: {}", msg);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to retrieve cards"})),
            )
                .into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::middleware::auth::UserContext;
    use crate::repository::card::mock::MockCardRepository;
    use crate::state::AppState;
    use malfestio_core::model::{Card, CardType};
    use std::sync::Arc;

    fn create_test_state() -> SharedState {
        let pool = crate::db::create_mock_pool();
        let card_repo = Arc::new(MockCardRepository::new()) as Arc<dyn crate::repository::card::CardRepository>;
        let note_repo = Arc::new(crate::repository::note::mock::MockNoteRepository::new())
            as Arc<dyn crate::repository::note::NoteRepository>;
        let oauth_repo = Arc::new(crate::repository::oauth::mock::MockOAuthRepository::new())
            as Arc<dyn crate::repository::oauth::OAuthRepository>;
        AppState::new_with_repos(pool, card_repo, note_repo, oauth_repo)
    }

    #[tokio::test]
    async fn test_create_card_success() {
        let state = create_test_state();
        let user = UserContext {
            did: "did:plc:test123".to_string(),
            handle: "test.handle".to_string(),
            access_token: "test_token".to_string(),
            pds_url: "https://bsky.social".to_string(),
            has_dpop: false,
        };

        let payload = CreateCardRequest {
            deck_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            front: "Question".to_string(),
            back: "Answer".to_string(),
            media_url: None,
            card_type: CardType::default(),
            hints: vec![],
        };

        let response = create_card(axum::extract::State(state), Some(Extension(user)), Json(payload))
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_create_card_unauthorized() {
        let state = create_test_state();

        let payload = CreateCardRequest {
            deck_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            front: "Question".to_string(),
            back: "Answer".to_string(),
            media_url: None,
            card_type: CardType::default(),
            hints: vec![],
        };

        let response = create_card(axum::extract::State(state), None, Json(payload))
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_list_cards_success() {
        let pool = crate::db::create_mock_pool();

        let test_deck_id = "550e8400-e29b-41d4-a716-446655440000".to_string();
        let test_cards = vec![
            Card {
                id: "card-1".to_string(),
                owner_did: "did:plc:test".to_string(),
                deck_id: test_deck_id.clone(),
                front: "Q1".to_string(),
                back: "A1".to_string(),
                media_url: None,
                card_type: CardType::default(),
                hints: vec![],
            },
            Card {
                id: "card-2".to_string(),
                owner_did: "did:plc:test".to_string(),
                deck_id: test_deck_id.clone(),
                front: "Q2".to_string(),
                back: "A2".to_string(),
                media_url: None,
                card_type: CardType::default(),
                hints: vec![],
            },
        ];

        let card_repo =
            Arc::new(MockCardRepository::with_cards(test_cards)) as Arc<dyn crate::repository::card::CardRepository>;
        let note_repo = Arc::new(crate::repository::note::mock::MockNoteRepository::new())
            as Arc<dyn crate::repository::note::NoteRepository>;
        let oauth_repo = Arc::new(crate::repository::oauth::mock::MockOAuthRepository::new())
            as Arc<dyn crate::repository::oauth::OAuthRepository>;

        let state = AppState::new_with_repos(pool, card_repo, note_repo, oauth_repo);

        let response = list_cards(axum::extract::State(state), None, Path(test_deck_id))
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
