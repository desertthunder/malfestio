use crate::middleware::auth::UserContext;
use crate::state::SharedState;
use axum::{
    Json,
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use malfestio_core::model::{Card, Visibility};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct CreateCardRequest {
    deck_id: String,
    front: String,
    back: String,
    media_url: Option<String>,
}

pub async fn create_card(
    State(state): State<SharedState>, ctx: Option<axum::Extension<UserContext>>, Json(payload): Json<CreateCardRequest>,
) -> impl IntoResponse {
    let user = match ctx {
        Some(axum::Extension(user)) => user,
        None => return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Unauthorized"}))).into_response(),
    };

    {
        let decks = state.decks.read().unwrap();
        let deck = decks.iter().find(|d| d.id == payload.deck_id);
        match deck {
            Some(d) => {
                if d.owner_did != user.did {
                    return (StatusCode::FORBIDDEN, Json(json!({"error": "Not deck owner"}))).into_response();
                }
            }
            None => return (StatusCode::NOT_FOUND, Json(json!({"error": "Deck not found"}))).into_response(),
        }
    }

    let new_card = Card {
        id: uuid::Uuid::new_v4().to_string(),
        owner_did: user.did,
        deck_id: payload.deck_id,
        front: payload.front,
        back: payload.back,
        media_url: payload.media_url,
    };

    state.cards.write().unwrap().push(new_card.clone());

    (StatusCode::CREATED, Json(new_card)).into_response()
}

pub async fn list_cards(
    State(state): State<SharedState>, Path(deck_id): Path<String>, ctx: Option<axum::Extension<UserContext>>,
) -> impl IntoResponse {
    let user_did = ctx.map(|Extension(u)| u.did);

    {
        let decks = state.decks.read().unwrap();
        if let Some(deck) = decks.iter().find(|d| d.id == deck_id) {
            let is_owner = user_did.as_ref() == Some(&deck.owner_did);
            if deck.visibility == Visibility::Private && !is_owner {
                return (StatusCode::FORBIDDEN, Json(json!({"error": "Private deck"}))).into_response();
            }

            if let Visibility::SharedWith(dids) = &deck.visibility
                && !is_owner
                && (user_did.is_none() || !dids.contains(user_did.as_ref().unwrap()))
            {
                return (StatusCode::FORBIDDEN, Json(json!({"error": "Access denied"}))).into_response();
            }
        } else {
            return (StatusCode::NOT_FOUND, Json(json!({"error": "Deck not found"}))).into_response();
        }
    }

    let cards = state.cards.read().unwrap();
    let deck_cards: Vec<Card> = cards.iter().filter(|c| c.deck_id == deck_id).cloned().collect();

    Json(deck_cards).into_response()
}
