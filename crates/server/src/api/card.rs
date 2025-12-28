use crate::middleware::auth::UserContext;
use crate::state::SharedState;

use axum::{
    Json,
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct CreateCardRequest {
    deck_id: String,
    front: String,
    back: String,
    media_url: Option<String>,
}

pub async fn create_card(
    State(_state): State<SharedState>, _ctx: Option<Extension<UserContext>>, Json(_payload): Json<CreateCardRequest>,
) -> impl IntoResponse {
    // TODO: Implement database-backed card creation
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({"error": "Card creation not yet implemented with database"})),
    )
        .into_response()
}

pub async fn list_cards(
    State(_state): State<SharedState>, _ctx: Option<Extension<UserContext>>, Path(_deck_id): Path<String>,
) -> impl IntoResponse {
    // TODO: Implement database-backed card listing
    Json(Vec::<serde_json::Value>::new()).into_response()
}
