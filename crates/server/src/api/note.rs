use crate::middleware::auth::UserContext;
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
#[allow(dead_code)]
pub struct CreateNoteRequest {
    title: String,
    body: String,
    tags: Vec<String>,
    visibility: Visibility,
}

pub async fn create_note(
    State(_state): State<SharedState>, _ctx: Option<Extension<UserContext>>, Json(_payload): Json<CreateNoteRequest>,
) -> impl IntoResponse {
    // TODO: Implement database-backed note creation
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({"error": "Note creation not yet implemented with database"})),
    )
        .into_response()
}

pub async fn list_notes(State(_state): State<SharedState>, _ctx: Option<Extension<UserContext>>) -> impl IntoResponse {
    // TODO: Implement database-backed note listing
    Json(Vec::<serde_json::Value>::new()).into_response()
}

pub async fn get_note(
    State(_state): State<SharedState>, _ctx: Option<Extension<UserContext>>, Path(_id): Path<String>,
) -> impl IntoResponse {
    // TODO: Implement database-backed note retrieval
    (StatusCode::NOT_FOUND, Json(json!({"error": "Note not found"}))).into_response()
}
