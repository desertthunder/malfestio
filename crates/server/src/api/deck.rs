use crate::middleware::auth::UserContext;

use axum::{
    Json,
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use malfestio_core::model::{Deck, Visibility};
use serde::Deserialize;
use serde_json::json;
use std::sync::{Arc, RwLock};

type Db = Arc<RwLock<Vec<Deck>>>;

#[derive(Deserialize)]
pub struct CreateDeckRequest {
    title: String,
    description: String,
    tags: Vec<String>,
    visibility: Visibility,
}

pub fn init_db() -> Db {
    Arc::new(RwLock::new(Vec::new()))
}

pub async fn create_deck(
    State(db): State<Db>, ctx: Option<axum::Extension<UserContext>>, Json(payload): Json<CreateDeckRequest>,
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
    };

    db.write().unwrap().push(new_deck.clone());

    (StatusCode::CREATED, Json(new_deck)).into_response()
}

pub async fn list_decks(State(db): State<Db>, ctx: Option<axum::Extension<UserContext>>) -> impl IntoResponse {
    let user_did = ctx.map(|Extension(u)| u.did);

    let decks = db.read().unwrap();

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
            false
        })
        .cloned()
        .collect();

    Json(visible_decks).into_response()
}

pub async fn get_deck(
    State(db): State<Db>, ctx: Option<axum::Extension<UserContext>>, Path(id): Path<String>,
) -> impl IntoResponse {
    let user_did = ctx.map(|Extension(u)| u.did);
    let decks = db.read().unwrap();

    if let Some(deck) = decks.iter().find(|d| d.id == id) {
        let is_owner = user_did.as_ref() == Some(&deck.owner_did);

        if deck.visibility == Visibility::Public || is_owner {
            return Json(deck).into_response();
        }

        if deck.visibility == Visibility::Unlisted {
            return Json(deck).into_response();
        }
        return (StatusCode::FORBIDDEN, Json(json!({"error": "Access denied"}))).into_response();
    }

    (StatusCode::NOT_FOUND, Json(json!({"error": "Deck not found"}))).into_response()
}
