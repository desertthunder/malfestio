use crate::middleware::auth::UserContext;
use crate::state::SharedState;
use axum::{
    Json,
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::json;

/// GET /api/export/:collection
/// Export data for the authenticated user
pub async fn export_collection(
    State(state): State<SharedState>, Extension(user): Extension<UserContext>, Path(collection): Path<String>,
) -> impl IntoResponse {
    match collection.as_str() {
        "decks" => match state.deck_repo.get_decks_by_user(&user.did).await {
            Ok(decks) => Json(json!(decks)).into_response(),
            Err(e) => {
                tracing::error!("Failed to export decks: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to export decks"})),
                )
                    .into_response()
            }
        },
        "notes" => match state.note_repo.get_notes_by_user(&user.did).await {
            Ok(notes) => Json(json!(notes)).into_response(),
            Err(e) => {
                tracing::error!("Failed to export notes: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to export notes"})),
                )
                    .into_response()
            }
        },
        _ => (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid collection"}))).into_response(),
    }
}
