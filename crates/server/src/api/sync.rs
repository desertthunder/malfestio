//! Sync API endpoints for bi-directional PDS synchronization.
//!
//! Provides endpoints for pushing local changes to PDS, getting sync status,
//! and resolving conflicts.

use crate::middleware::auth::UserContext;
use crate::state::SharedState;
use crate::sync_service::{ConflictStrategy, SyncError, SyncService};
use axum::{
    Json,
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::str::FromStr;

/// Response for sync push operation.
#[derive(Debug, Clone, Serialize)]
pub struct PushResponse {
    pub entity_type: String,
    pub entity_id: String,
    pub pds_uri: Option<String>,
    pub pds_cid: Option<String>,
    pub version: i32,
    pub status: String,
}

/// Response for sync status query.
#[derive(Debug, Clone, Serialize)]
pub struct SyncStatusResponse {
    pub pending_count: usize,
    pub conflict_count: usize,
    pub pending_items: Vec<PendingItem>,
    pub conflicts: Vec<ConflictItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PendingItem {
    pub entity_type: String,
    pub entity_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConflictItem {
    pub entity_type: String,
    pub entity_id: String,
    pub local_version: i32,
    pub remote_version: Option<i32>,
}

/// Request for conflict resolution.
#[derive(Debug, Clone, Deserialize)]
pub struct ResolveConflictRequest {
    pub strategy: String,
}

/// Push a deck to the user's PDS.
///
/// POST /api/sync/push/deck/:id
pub async fn push_deck(
    State(state): State<SharedState>, ctx: Option<Extension<UserContext>>, Path(deck_id): Path<String>,
) -> impl IntoResponse {
    let user = match ctx {
        Some(Extension(user)) => user,
        None => return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Unauthorized"}))).into_response(),
    };

    let sync_service = create_sync_service(&state);

    match sync_service.push_deck(&deck_id, &user).await {
        Ok(result) => (
            StatusCode::OK,
            Json(PushResponse {
                entity_type: result.entity_type,
                entity_id: result.entity_id,
                pds_uri: result.pds_uri,
                pds_cid: result.pds_cid,
                version: result.new_version,
                status: result.status.to_string(),
            }),
        )
            .into_response(),
        Err(e) => sync_error_response(e),
    }
}

/// Push a note to the user's PDS.
///
/// POST /api/sync/push/note/:id
pub async fn push_note(
    State(state): State<SharedState>, ctx: Option<Extension<UserContext>>, Path(note_id): Path<String>,
) -> impl IntoResponse {
    let user = match ctx {
        Some(Extension(user)) => user,
        None => return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Unauthorized"}))).into_response(),
    };

    let sync_service = create_sync_service(&state);

    match sync_service.push_note(&note_id, &user).await {
        Ok(result) => (
            StatusCode::OK,
            Json(PushResponse {
                entity_type: result.entity_type,
                entity_id: result.entity_id,
                pds_uri: result.pds_uri,
                pds_cid: result.pds_cid,
                version: result.new_version,
                status: result.status.to_string(),
            }),
        )
            .into_response(),
        Err(e) => sync_error_response(e),
    }
}

/// Get the current sync status for the authenticated user.
///
/// GET /api/sync/status
pub async fn get_sync_status(
    State(state): State<SharedState>, ctx: Option<Extension<UserContext>>,
) -> impl IntoResponse {
    let user = match ctx {
        Some(Extension(user)) => user,
        None => return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Unauthorized"}))).into_response(),
    };

    let sync_service = create_sync_service(&state);

    match sync_service.get_sync_status(&user).await {
        Ok(summary) => (
            StatusCode::OK,
            Json(SyncStatusResponse {
                pending_count: summary.pending_count,
                conflict_count: summary.conflict_count,
                pending_items: summary
                    .pending_items
                    .into_iter()
                    .map(|(entity_type, entity_id)| PendingItem { entity_type, entity_id })
                    .collect(),
                conflicts: summary
                    .conflicts
                    .into_iter()
                    .map(|c| ConflictItem {
                        entity_type: c.entity_type,
                        entity_id: c.entity_id,
                        local_version: c.local_version,
                        remote_version: c.remote_version,
                    })
                    .collect(),
            }),
        )
            .into_response(),
        Err(e) => sync_error_response(e),
    }
}

/// Resolve a sync conflict.
///
/// POST /api/sync/resolve/:entity_type/:id
pub async fn resolve_conflict(
    State(state): State<SharedState>, ctx: Option<Extension<UserContext>>,
    Path((entity_type, entity_id)): Path<(String, String)>, Json(payload): Json<ResolveConflictRequest>,
) -> impl IntoResponse {
    let user = match ctx {
        Some(Extension(user)) => user,
        None => return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Unauthorized"}))).into_response(),
    };

    let strategy = match ConflictStrategy::from_str(&payload.strategy) {
        Ok(s) => s,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Invalid strategy. Use: last_write_wins, keep_local, or keep_remote"})),
            )
                .into_response();
        }
    };

    let sync_service = create_sync_service(&state);

    match sync_service
        .resolve_conflict(&entity_type, &entity_id, strategy, &user)
        .await
    {
        Ok(result) => (
            StatusCode::OK,
            Json(PushResponse {
                entity_type: result.entity_type,
                entity_id: result.entity_id,
                pds_uri: result.pds_uri,
                pds_cid: result.pds_cid,
                version: result.new_version,
                status: result.status.to_string(),
            }),
        )
            .into_response(),
        Err(e) => sync_error_response(e),
    }
}

/// Create a SyncService from the app state.
fn create_sync_service(state: &SharedState) -> SyncService {
    SyncService::new(
        state.sync_repo.clone(),
        state.deck_repo.clone(),
        state.card_repo.clone(),
        state.note_repo.clone(),
        state.oauth_repo.clone(),
    )
}

/// Convert SyncError to HTTP response.
fn sync_error_response(error: SyncError) -> axum::response::Response {
    let (status, message) = match &error {
        SyncError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
        SyncError::AuthRequired(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
        SyncError::NoTokens(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
        SyncError::InvalidArgument(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
        SyncError::ConflictDetected(info) => (
            StatusCode::CONFLICT,
            format!("Conflict for {}:{}", info.entity_type, info.entity_id),
        ),
        SyncError::PdsError(e) => (StatusCode::BAD_GATEWAY, e.to_string()),
        SyncError::RepoError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    };

    tracing::error!("Sync error: {}", error);
    (status, Json(json!({"error": message}))).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_response_serialization() {
        let response = PushResponse {
            entity_type: "deck".to_string(),
            entity_id: "123".to_string(),
            pds_uri: Some("at://did:plc:test/deck/tid".to_string()),
            pds_cid: Some("bafycid".to_string()),
            version: 2,
            status: "synced".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"entity_type\":\"deck\""));
        assert!(json.contains("\"version\":2"));
    }

    #[test]
    fn test_sync_status_response_serialization() {
        let response = SyncStatusResponse {
            pending_count: 2,
            conflict_count: 1,
            pending_items: vec![
                PendingItem { entity_type: "deck".to_string(), entity_id: "1".to_string() },
                PendingItem { entity_type: "note".to_string(), entity_id: "2".to_string() },
            ],
            conflicts: vec![ConflictItem {
                entity_type: "deck".to_string(),
                entity_id: "3".to_string(),
                local_version: 5,
                remote_version: Some(6),
            }],
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"pending_count\":2"));
        assert!(json.contains("\"conflict_count\":1"));
    }

    #[test]
    fn test_resolve_conflict_request_deserialization() {
        let json = r#"{"strategy": "last_write_wins"}"#;
        let request: ResolveConflictRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.strategy, "last_write_wins");

        let json = r#"{"strategy": "keep_local"}"#;
        let request: ResolveConflictRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.strategy, "keep_local");
    }

    #[test]
    fn test_sync_error_response_not_found() {
        let error = SyncError::NotFound("deck:123".to_string());
        let response = sync_error_response(error);
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_sync_error_response_unauthorized() {
        let error = SyncError::AuthRequired("missing token".to_string());
        let response = sync_error_response(error);
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_sync_error_response_bad_request() {
        let error = SyncError::InvalidArgument("bad entity type".to_string());
        let response = sync_error_response(error);
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_sync_error_response_conflict() {
        let error = SyncError::ConflictDetected(crate::sync_service::ConflictInfo {
            entity_type: "deck".to_string(),
            entity_id: "123".to_string(),
            local_version: 5,
            remote_version: Some(6),
            local_updated_at: None,
            remote_updated_at: None,
        });
        let response = sync_error_response(error);
        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[test]
    fn test_pending_item_serialization() {
        let item = PendingItem { entity_type: "note".to_string(), entity_id: "456".to_string() };

        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("\"entity_type\":\"note\""));
        assert!(json.contains("\"entity_id\":\"456\""));
    }

    #[test]
    fn test_conflict_item_serialization() {
        let item = ConflictItem {
            entity_type: "deck".to_string(),
            entity_id: "789".to_string(),
            local_version: 3,
            remote_version: Some(4),
        };

        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("\"local_version\":3"));
        assert!(json.contains("\"remote_version\":4"));
    }

    #[test]
    fn test_conflict_item_no_remote_version() {
        let item = ConflictItem {
            entity_type: "note".to_string(),
            entity_id: "abc".to_string(),
            local_version: 1,
            remote_version: None,
        };

        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("\"remote_version\":null"));
    }

    #[tokio::test]
    async fn test_push_deck_unauthorized() {
        let pool = crate::db::create_mock_pool();
        let repos = crate::state::Repositories::default();
        let config = crate::state::AppConfig { pds_url: "https://test.example.com".to_string() };
        let state = crate::state::AppState::new(pool, repos, config);

        let response = push_deck(State(state), None, Path("deck-123".to_string()))
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_push_note_unauthorized() {
        let pool = crate::db::create_mock_pool();
        let repos = crate::state::Repositories::default();
        let config = crate::state::AppConfig { pds_url: "https://test.example.com".to_string() };
        let state = crate::state::AppState::new(pool, repos, config);

        let response = push_note(State(state), None, Path("note-456".to_string()))
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_get_sync_status_unauthorized() {
        let pool = crate::db::create_mock_pool();
        let repos = crate::state::Repositories::default();
        let config = crate::state::AppConfig { pds_url: "https://test.example.com".to_string() };
        let state = crate::state::AppState::new(pool, repos, config);

        let response = get_sync_status(State(state), None).await.into_response();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_resolve_conflict_unauthorized() {
        let pool = crate::db::create_mock_pool();
        let repos = crate::state::Repositories::default();
        let config = crate::state::AppConfig { pds_url: "https://test.example.com".to_string() };
        let state = crate::state::AppState::new(pool, repos, config);

        let response = resolve_conflict(
            State(state),
            None,
            Path(("deck".to_string(), "123".to_string())),
            Json(ResolveConflictRequest { strategy: "last_write_wins".to_string() }),
        )
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_resolve_conflict_invalid_strategy() {
        let pool = crate::db::create_mock_pool();
        let repos = crate::state::Repositories::default();
        let config = crate::state::AppConfig { pds_url: "https://test.example.com".to_string() };
        let state = crate::state::AppState::new(pool, repos, config);

        let user = UserContext {
            did: "did:plc:alice".to_string(),
            handle: "alice.bsky.social".to_string(),
            access_token: "test_token".to_string(),
            pds_url: "https://bsky.social".to_string(),
            has_dpop: false,
        };

        let response = resolve_conflict(
            State(state),
            Some(Extension(user)),
            Path(("deck".to_string(), "123".to_string())),
            Json(ResolveConflictRequest { strategy: "invalid_strategy".to_string() }),
        )
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
