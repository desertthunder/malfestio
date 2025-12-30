use crate::state::SharedState;
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};

use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Deserialize)]
pub struct LoginRequest {
    identifier: String,
    password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    access_jwt: String,
    refresh_jwt: String,
    did: String,
    handle: String,
}

/// TODO: Make PDS URL configurable (bluesky users can use their own PDS)
pub async fn login(State(state): State<SharedState>, Json(payload): Json<LoginRequest>) -> impl IntoResponse {
    let client = reqwest::Client::new();
    let pds_url = &state.config.pds_url;

    let resp = client
        .post(format!("{}/xrpc/com.atproto.server.createSession", pds_url))
        .json(&json!({
            "identifier": payload.identifier,
            "password": payload.password
        }))
        .send()
        .await;

    match resp {
        Ok(response) => {
            if response.status().is_success() {
                let body: serde_json::Value = response.json().await.unwrap_or_default();
                let access_jwt = body["accessJwt"].as_str().unwrap_or("").to_string();
                let refresh_jwt = body["refreshJwt"].as_str().unwrap_or("").to_string();
                let did = body["did"].as_str().unwrap_or("").to_string();
                let handle = body["handle"].as_str().unwrap_or("").to_string();

                (
                    StatusCode::OK,
                    Json(json!({
                        "accessJwt": access_jwt,
                        "refreshJwt": refresh_jwt,
                        "did": did,
                        "handle": handle
                    })),
                )
            } else {
                let error_body: serde_json::Value = response.json().await.unwrap_or_default();
                (StatusCode::UNAUTHORIZED, Json(error_body))
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

/// TODO: replace with middleware
pub async fn me() -> impl IntoResponse {
    Json(json!({ "status": "authenticated" }))
}
