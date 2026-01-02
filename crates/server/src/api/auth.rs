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

/// Login with app password.
///
/// Resolves the user's PDS from their handle/DID, authenticates with that PDS,
/// and stores the session for future requests.
pub async fn login(State(state): State<SharedState>, Json(payload): Json<LoginRequest>) -> impl IntoResponse {
    use crate::oauth::resolver::{is_valid_did, is_valid_handle};
    use crate::repository::oauth::StoreAppPasswordSessionRequest;

    let client = reqwest::Client::new();

    let pds_url = if is_valid_did(&payload.identifier) {
        match state.identity_resolver.resolve_did(&payload.identifier).await {
            Ok(identity) => identity.pds_url,
            Err(e) => {
                tracing::error!("Failed to resolve DID {}: {}", payload.identifier, e);
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error": format!("Failed to resolve DID: {}", e) })),
                )
                    .into_response();
            }
        }
    } else if is_valid_handle(&payload.identifier) {
        let did = match state.identity_resolver.resolve_handle(&payload.identifier).await {
            Ok(did) => did,
            Err(e) => {
                tracing::error!("Failed to resolve handle {}: {}", payload.identifier, e);
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error": format!("Failed to resolve handle: {}", e) })),
                )
                    .into_response();
            }
        };

        match state.identity_resolver.resolve_did(&did).await {
            Ok(identity) => identity.pds_url,
            Err(e) => {
                tracing::error!("Failed to resolve DID {} for handle {}: {}", did, payload.identifier, e);
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error": format!("Failed to resolve DID: {}", e) })),
                )
                    .into_response();
            }
        }
    } else {
        tracing::warn!("Invalid identifier format: {}, using default PDS", payload.identifier);
        state.config.pds_url.clone()
    };

    tracing::info!("Authenticating {} with PDS: {}", payload.identifier, pds_url);

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

                // Store the session with the resolved PDS URL
                let store_result = state
                    .oauth_repo
                    .store_app_password_session(StoreAppPasswordSessionRequest {
                        did: &did,
                        pds_url: &pds_url,
                        access_token: &access_jwt,
                        refresh_token: Some(&refresh_jwt),
                        expires_at: None,
                    })
                    .await;

                if let Err(e) = store_result {
                    tracing::error!("Failed to store app password session: {}", e);
                }

                (
                    StatusCode::OK,
                    Json(json!({
                        "accessJwt": access_jwt,
                        "refreshJwt": refresh_jwt,
                        "did": did,
                        "handle": handle
                    })),
                )
                    .into_response()
            } else {
                let error_body: serde_json::Value = response.json().await.unwrap_or_default();
                (StatusCode::UNAUTHORIZED, Json(error_body)).into_response()
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn me(ctx: Option<axum::Extension<crate::middleware::auth::UserContext>>) -> impl IntoResponse {
    match ctx {
        Some(axum::Extension(user)) => (
            StatusCode::OK,
            Json(json!({
                "status": "authenticated",
                "did": user.did,
                "handle": user.handle
            })),
        )
            .into_response(),
        None => (StatusCode::UNAUTHORIZED, Json(json!({ "error": "Unauthorized" }))).into_response(),
    }
}
