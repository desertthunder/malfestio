use crate::middleware::auth::UserContext;
use crate::state::SharedState;
use axum::{
    Json,
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use malfestio_core::model::{Note, Visibility};
use regex::Regex;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct CreateNoteRequest {
    title: String,
    body: String,
    tags: Vec<String>,
    visibility: Visibility,
}

fn extract_links(body: &str) -> Vec<String> {
    let re = Regex::new(r"\[\[(.*?)\]\]").unwrap();
    re.captures_iter(body).map(|cap| cap[1].to_string()).collect()
}

pub async fn create_note(
    State(state): State<SharedState>, ctx: Option<axum::Extension<UserContext>>, Json(payload): Json<CreateNoteRequest>,
) -> impl IntoResponse {
    let user = match ctx {
        Some(axum::Extension(user)) => user,
        None => return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Unauthorized"}))).into_response(),
    };

    let links = extract_links(&payload.body);

    let new_note = Note {
        id: uuid::Uuid::new_v4().to_string(),
        owner_did: user.did,
        title: payload.title,
        body: payload.body,
        tags: payload.tags,
        visibility: payload.visibility,
        published_at: None,
        links,
    };

    state.notes.write().unwrap().push(new_note.clone());

    (StatusCode::CREATED, Json(new_note)).into_response()
}

pub async fn list_notes(
    State(state): State<SharedState>, ctx: Option<axum::Extension<UserContext>>,
) -> impl IntoResponse {
    let user_did = ctx.map(|Extension(u)| u.did);
    let notes = state.notes.read().unwrap();

    let visible_notes: Vec<Note> = notes
        .iter()
        .filter(|n| {
            if let Some(did) = &user_did
                && &n.owner_did == did
            {
                return true;
            }
            if n.visibility == Visibility::Public {
                return true;
            }
            if let Visibility::SharedWith(dids) = &n.visibility
                && let Some(did) = &user_did
                && dids.contains(did)
            {
                return true;
            }
            false
        })
        .cloned()
        .collect();

    Json(visible_notes).into_response()
}

pub async fn get_note(
    State(state): State<SharedState>, ctx: Option<axum::Extension<UserContext>>, Path(id): Path<String>,
) -> impl IntoResponse {
    let user_did = ctx.map(|Extension(u)| u.did);
    let notes = state.notes.read().unwrap();

    if let Some(note) = notes.iter().find(|n| n.id == id) {
        let is_owner = user_did.as_ref() == Some(&note.owner_did);

        if note.visibility == Visibility::Public || is_owner {
            let backlinks: Vec<String> = notes
                .iter()
                .filter(|n| n.links.contains(&note.title) && n.id != note.id) // Naive matching by title
                .map(|n| n.id.clone())
                .collect();

            let mut response = serde_json::to_value(note).unwrap();
            response["backlinks"] = json!(backlinks);

            return Json(response).into_response();
        }

        if let Visibility::SharedWith(dids) = &note.visibility
            && let Some(did) = &user_did
            && dids.contains(did)
        {
            return Json(note).into_response();
        }

        return (StatusCode::FORBIDDEN, Json(json!({"error": "Access denied"}))).into_response();
    }

    (StatusCode::NOT_FOUND, Json(json!({"error": "Note not found"}))).into_response()
}
