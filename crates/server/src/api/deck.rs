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

    let pool = &state.pool;
    let client = match pool.get().await {
        Ok(client) => client,
        Err(e) => {
            tracing::error!("Failed to get database connection: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Database connection failed"})),
            )
                .into_response();
        }
    };

    let deck_id = uuid::Uuid::new_v4();
    let visibility_json = match serde_json::to_value(&payload.visibility) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Failed to serialize visibility: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to serialize visibility"})),
            )
                .into_response();
        }
    };

    let result = client
        .execute(
            "INSERT INTO decks (id, owner_did, title, description, tags, visibility)
             VALUES ($1, $2, $3, $4, $5, $6)",
            &[
                &deck_id,
                &user.did,
                &payload.title,
                &payload.description,
                &payload.tags,
                &visibility_json,
            ],
        )
        .await;

    if let Err(e) = result {
        tracing::error!("Failed to insert deck: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to create deck"})),
        )
            .into_response();
    }

    let new_deck = Deck {
        id: deck_id.to_string(),
        owner_did: user.did,
        title: payload.title,
        description: payload.description,
        tags: payload.tags,
        visibility: payload.visibility,
        published_at: None,
        fork_of: None,
    };

    (StatusCode::CREATED, Json(new_deck)).into_response()
}

pub async fn list_decks(
    State(state): State<SharedState>, ctx: Option<axum::Extension<UserContext>>,
) -> impl IntoResponse {
    let user_did = ctx.map(|Extension(u)| u.did);

    let pool = &state.pool;
    let client = match pool.get().await {
        Ok(client) => client,
        Err(e) => {
            tracing::error!("Failed to get database connection: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Database connection failed"})),
            )
                .into_response();
        }
    };

    let query = if let Some(ref _did) = user_did {
        "SELECT id, owner_did, title, description, tags, visibility, published_at, fork_of, created_at, updated_at
         FROM decks
         WHERE owner_did = $1
            OR visibility->>'type' = 'Public'
            OR visibility->>'type' = 'Unlisted'
            OR (visibility->>'type' = 'SharedWith' AND visibility->'content' ? $1)
         ORDER BY created_at DESC"
    } else {
        "SELECT id, owner_did, title, description, tags, visibility, published_at, fork_of, created_at, updated_at
         FROM decks
         WHERE visibility->>'type' IN ('Public', 'Unlisted')
         ORDER BY created_at DESC"
    };

    let rows = if let Some(ref did) = user_did {
        client.query(query, &[did]).await
    } else {
        client.query(query, &[]).await
    };

    let rows = match rows {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("Failed to query decks: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to retrieve decks"})),
            )
                .into_response();
        }
    };

    let mut decks = Vec::new();
    for row in rows {
        let visibility_json: serde_json::Value = row.get("visibility");
        let visibility: Visibility = match serde_json::from_value(visibility_json) {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("Failed to deserialize visibility: {}", e);
                continue;
            }
        };

        let id: uuid::Uuid = row.get("id");
        let fork_of: Option<uuid::Uuid> = row.get("fork_of");

        decks.push(Deck {
            id: id.to_string(),
            owner_did: row.get("owner_did"),
            title: row.get("title"),
            description: row.get("description"),
            tags: row.get("tags"),
            visibility,
            published_at: row
                .get::<_, Option<chrono::DateTime<chrono::Utc>>>("published_at")
                .map(|dt| dt.to_rfc3339()),
            fork_of: fork_of.map(|u| u.to_string()),
        });
    }

    Json(decks).into_response()
}

pub async fn get_deck(
    State(state): State<SharedState>, ctx: Option<axum::Extension<UserContext>>, Path(id): Path<String>,
) -> impl IntoResponse {
    let user_did = ctx.map(|Extension(u)| u.did);

    let pool = &state.pool;
    let client = match pool.get().await {
        Ok(client) => client,
        Err(e) => {
            tracing::error!("Failed to get database connection: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Database connection failed"})),
            )
                .into_response();
        }
    };

    let deck_id = match uuid::Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid deck ID"}))).into_response(),
    };

    let row = match client
        .query_opt(
            "SELECT id, owner_did, title, description, tags, visibility, published_at, fork_of, created_at, updated_at
             FROM decks WHERE id = $1",
            &[&deck_id],
        )
        .await
    {
        Ok(Some(row)) => row,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(json!({"error": "Deck not found"}))).into_response(),
        Err(e) => {
            tracing::error!("Failed to query deck: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to retrieve deck"})),
            )
                .into_response();
        }
    };

    let visibility_json: serde_json::Value = row.get("visibility");
    let visibility: Visibility = match serde_json::from_value(visibility_json) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Failed to deserialize visibility: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to parse deck visibility"})),
            )
                .into_response();
        }
    };

    let owner_did: String = row.get("owner_did");
    let is_owner = user_did.as_ref() == Some(&owner_did);

    let has_access = match &visibility {
        Visibility::Public | Visibility::Unlisted => true,
        Visibility::Private => is_owner,
        Visibility::SharedWith(dids) => is_owner || user_did.as_ref().map(|did| dids.contains(did)).unwrap_or(false),
    };

    if !has_access {
        return (StatusCode::FORBIDDEN, Json(json!({"error": "Access denied"}))).into_response();
    }

    let uuid_id: uuid::Uuid = row.get("id");
    let fork_of: Option<uuid::Uuid> = row.get("fork_of");

    let deck = Deck {
        id: uuid_id.to_string(),
        owner_did,
        title: row.get("title"),
        description: row.get("description"),
        tags: row.get("tags"),
        visibility,
        published_at: row
            .get::<_, Option<chrono::DateTime<chrono::Utc>>>("published_at")
            .map(|dt| dt.to_rfc3339()),
        fork_of: fork_of.map(|u| u.to_string()),
    };

    Json(deck).into_response()
}

pub async fn publish_deck(
    State(state): State<SharedState>, ctx: Option<axum::Extension<UserContext>>, Path(id): Path<String>,
    Json(payload): Json<PublishDeckRequest>,
) -> impl IntoResponse {
    let user = match ctx {
        Some(axum::Extension(user)) => user,
        None => return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Unauthorized"}))).into_response(),
    };

    let pool = &state.pool;
    let client = match pool.get().await {
        Ok(client) => client,
        Err(e) => {
            tracing::error!("Failed to get database connection: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Database connection failed"})),
            )
                .into_response();
        }
    };

    let deck_id = match uuid::Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid deck ID"}))).into_response(),
    };

    let deck_row = match client
        .query_opt("SELECT owner_did FROM decks WHERE id = $1", &[&deck_id])
        .await
    {
        Ok(Some(row)) => row,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(json!({"error": "Deck not found"}))).into_response(),
        Err(e) => {
            tracing::error!("Failed to query deck: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Database error"})),
            )
                .into_response();
        }
    };

    let owner_did: String = deck_row.get("owner_did");
    if owner_did != user.did {
        return (StatusCode::FORBIDDEN, Json(json!({"error": "Only owner can publish"}))).into_response();
    }

    let (new_visibility, published_at) = if payload.published {
        (
            serde_json::to_value(&Visibility::Public).unwrap(),
            Some(chrono::Utc::now()),
        )
    } else {
        (serde_json::to_value(&Visibility::Private).unwrap(), None)
    };

    match client
        .execute(
            "UPDATE decks SET visibility = $1, published_at = $2 WHERE id = $3",
            &[&new_visibility, &published_at, &deck_id],
        )
        .await
    {
        Ok(_) => (),
        Err(e) => {
            tracing::error!("Failed to update deck: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to update deck"})),
            )
                .into_response();
        }
    }

    let row = match client
        .query_one(
            "SELECT id, owner_did, title, description, tags, visibility, published_at, fork_of
             FROM decks WHERE id = $1",
            &[&deck_id],
        )
        .await
    {
        Ok(row) => row,
        Err(e) => {
            tracing::error!("Failed to fetch updated deck: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to retrieve updated deck"})),
            )
                .into_response();
        }
    };

    let visibility_json: serde_json::Value = row.get("visibility");
    let visibility: Visibility = serde_json::from_value(visibility_json).unwrap();
    let uuid_id: uuid::Uuid = row.get("id");
    let fork_of: Option<uuid::Uuid> = row.get("fork_of");

    let deck = Deck {
        id: uuid_id.to_string(),
        owner_did: row.get("owner_did"),
        title: row.get("title"),
        description: row.get("description"),
        tags: row.get("tags"),
        visibility,
        published_at: row
            .get::<_, Option<chrono::DateTime<chrono::Utc>>>("published_at")
            .map(|dt| dt.to_rfc3339()),
        fork_of: fork_of.map(|u| u.to_string()),
    };

    Json(deck).into_response()
}
