use crate::middleware::auth::UserContext;
use crate::repository::note::NoteRepoError;
use crate::state::SharedState;

use axum::{
    Json,
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use malfestio_core::model::Visibility;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct CreateNoteRequest {
    title: String,
    body: String,
    tags: Vec<String>,
    visibility: Visibility,
}

pub async fn create_note(
    State(state): State<SharedState>, ctx: Option<Extension<UserContext>>, Json(payload): Json<CreateNoteRequest>,
) -> impl IntoResponse {
    let user = match ctx {
        Some(Extension(user)) => user,
        None => return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Unauthorized"}))).into_response(),
    };

    let result = state
        .note_repo
        .create(
            &user.did,
            &payload.title,
            &payload.body,
            payload.tags,
            payload.visibility,
        )
        .await;

    match result {
        Ok(note) => (StatusCode::CREATED, Json(note)).into_response(),
        Err(NoteRepoError::SerializationError(msg)) => {
            tracing::error!("Serialization error: {}", msg);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to create note"})),
            )
                .into_response()
        }
        Err(NoteRepoError::DatabaseError(msg)) => {
            tracing::error!("Database error: {}", msg);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to create note"})),
            )
                .into_response()
        }
        Err(NoteRepoError::NotFound(msg)) => (StatusCode::NOT_FOUND, Json(json!({"error": msg}))).into_response(),
        Err(NoteRepoError::InvalidArgument(msg)) => {
            (StatusCode::BAD_REQUEST, Json(json!({"error": msg}))).into_response()
        }
    }
}

pub async fn list_notes(State(state): State<SharedState>, ctx: Option<Extension<UserContext>>) -> impl IntoResponse {
    let viewer_did = ctx.map(|Extension(u)| u.did);

    let result = state.note_repo.list(viewer_did.as_deref()).await;

    match result {
        Ok(notes) => Json(notes).into_response(),
        Err(NoteRepoError::SerializationError(msg)) => {
            tracing::error!("Serialization error: {}", msg);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to retrieve notes"})),
            )
                .into_response()
        }
        Err(NoteRepoError::DatabaseError(msg)) => {
            tracing::error!("Database error: {}", msg);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to retrieve notes"})),
            )
                .into_response()
        }
        Err(NoteRepoError::NotFound(msg)) => (StatusCode::NOT_FOUND, Json(json!({"error": msg}))).into_response(),
        Err(NoteRepoError::InvalidArgument(msg)) => {
            (StatusCode::BAD_REQUEST, Json(json!({"error": msg}))).into_response()
        }
    }
}

pub async fn get_note(
    State(state): State<SharedState>, ctx: Option<Extension<UserContext>>, Path(id): Path<String>,
) -> impl IntoResponse {
    let viewer_did = ctx.map(|Extension(u)| u.did);

    let result = state.note_repo.get(&id, viewer_did.as_deref()).await;

    match result {
        Ok(note) => Json(note).into_response(),
        Err(NoteRepoError::NotFound(msg)) => (StatusCode::NOT_FOUND, Json(json!({"error": msg}))).into_response(),
        Err(NoteRepoError::InvalidArgument(msg)) => {
            if msg.contains("Access denied") {
                (StatusCode::FORBIDDEN, Json(json!({"error": msg}))).into_response()
            } else {
                (StatusCode::BAD_REQUEST, Json(json!({"error": msg}))).into_response()
            }
        }
        Err(NoteRepoError::SerializationError(msg)) => {
            tracing::error!("Serialization error: {}", msg);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to retrieve note"})),
            )
                .into_response()
        }
        Err(NoteRepoError::DatabaseError(msg)) => {
            tracing::error!("Database error: {}", msg);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to retrieve note"})),
            )
                .into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::middleware::auth::UserContext;
    use crate::repository::note::mock::MockNoteRepository;
    use crate::state::AppState;
    use malfestio_core::model::{Note, Visibility};
    use std::sync::Arc;

    fn create_test_state() -> SharedState {
        let pool = crate::db::create_pool().unwrap_or_else(|_| panic!("For testing without DB, use mock pool"));
        let card_repo = Arc::new(crate::repository::card::mock::MockCardRepository::new())
            as Arc<dyn crate::repository::card::CardRepository>;
        let note_repo = Arc::new(MockNoteRepository::new()) as Arc<dyn crate::repository::note::NoteRepository>;
        AppState::new_with_repos(pool, card_repo, note_repo)
    }

    #[tokio::test]
    async fn test_create_note_success() {
        let state = create_test_state();
        let user = UserContext { did: "did:plc:test123".to_string(), handle: "test.handle".to_string() };

        let payload = CreateNoteRequest {
            title: "Test Note".to_string(),
            body: "This is a test note".to_string(),
            tags: vec!["test".to_string()],
            visibility: Visibility::Private,
        };

        let response = create_note(axum::extract::State(state), Some(Extension(user)), Json(payload))
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_create_note_unauthorized() {
        let state = create_test_state();

        let payload = CreateNoteRequest {
            title: "Test Note".to_string(),
            body: "This is a test note".to_string(),
            tags: vec!["test".to_string()],
            visibility: Visibility::Private,
        };

        let response = create_note(axum::extract::State(state), None, Json(payload))
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_list_notes_with_visibility_filtering() {
        let pool = crate::db::create_pool().unwrap_or_else(|_| panic!("For testing without DB, use mock pool"));

        let test_notes = vec![
            Note {
                id: "note-1".to_string(),
                owner_did: "did:plc:test".to_string(),
                title: "Public Note".to_string(),
                body: "Public content".to_string(),
                tags: vec![],
                visibility: Visibility::Public,
                published_at: None,
                links: vec![],
            },
            Note {
                id: "note-2".to_string(),
                owner_did: "did:plc:test".to_string(),
                title: "Private Note".to_string(),
                body: "Private content".to_string(),
                tags: vec![],
                visibility: Visibility::Private,
                published_at: None,
                links: vec![],
            },
        ];

        let note_repo =
            Arc::new(MockNoteRepository::with_notes(test_notes)) as Arc<dyn crate::repository::note::NoteRepository>;
        let card_repo = Arc::new(crate::repository::card::mock::MockCardRepository::new())
            as Arc<dyn crate::repository::card::CardRepository>;

        let state = AppState::new_with_repos(pool, card_repo, note_repo);

        let response = list_notes(axum::extract::State(state.clone()), None)
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_note_access_control() {
        let pool = crate::db::create_pool().unwrap_or_else(|_| panic!("For testing without DB, use mock pool"));

        let note_id = "test-note-id".to_string();
        let test_notes = vec![Note {
            id: note_id.clone(),
            owner_did: "did:plc:owner".to_string(),
            title: "Private Note".to_string(),
            body: "Private content".to_string(),
            tags: vec![],
            visibility: Visibility::Private,
            published_at: None,
            links: vec![],
        }];

        let note_repo =
            Arc::new(MockNoteRepository::with_notes(test_notes)) as Arc<dyn crate::repository::note::NoteRepository>;
        let card_repo = Arc::new(crate::repository::card::mock::MockCardRepository::new())
            as Arc<dyn crate::repository::card::CardRepository>;

        let state = AppState::new_with_repos(pool, card_repo, note_repo);

        let owner = UserContext { did: "did:plc:owner".to_string(), handle: "owner.handle".to_string() };

        let response = get_note(
            axum::extract::State(state.clone()),
            Some(Extension(owner)),
            Path(note_id.clone()),
        )
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);

        let other_user = UserContext { did: "did:plc:other".to_string(), handle: "other.handle".to_string() };
        let response = get_note(axum::extract::State(state), Some(Extension(other_user)), Path(note_id))
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}
