pub mod api;
pub mod db;
pub mod firehose;
pub mod middleware;
pub mod oauth;
pub mod pds;
pub mod repository;
pub mod state;
pub mod well_known;

use axum::http::Method;
use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    middleware as axum_middleware,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde_json::{Value, json};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub async fn start() -> malfestio_core::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "malfestio_server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Malfestio Server...");

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| std::env::var("DB_URL").expect("DATABASE_URL or DB_URL must be set"));
    let pool = db::create_pool(&database_url).map_err(|e| {
        tracing::error!("Failed to create database pool: {}", e);
        malfestio_core::Error::Database(format!("Failed to create database pool: {}", e))
    })?;

    tracing::info!("Database connection pool created");

    let pds_url = std::env::var("PDS_URL").unwrap_or_else(|_| "https://bsky.social".to_string());
    let config = state::AppConfig { pds_url };
    let repos = state::Repositories::from(&pool);
    let state = state::AppState::new(pool, repos, config);
    let oauth_state = std::sync::Arc::new(api::oauth::OAuthState::new());

    let auth_routes = Router::new()
        .route("/me", get(api::auth::me))
        .route("/decks", post(api::deck::create_deck))
        .route("/decks/{id}/publish", post(api::deck::publish_deck))
        .route("/decks/{id}/fork", post(api::deck::fork_deck))
        .route("/notes", post(api::note::create_note))
        .route("/cards", post(api::card::create_card))
        .route("/review/due", get(api::review::get_due_cards))
        .route("/review/submit", post(api::review::submit_review))
        .route("/review/stats", get(api::review::get_stats))
        .route("/social/follow/{did}", post(api::social::follow))
        .route("/social/unfollow/{did}", post(api::social::unfollow))
        .route("/decks/{id}/comments", post(api::social::add_comment))
        .route("/feeds/follows", get(api::feed::get_feed_follows))
        .route("/preferences", get(api::preferences::get_preferences))
        .route("/preferences", axum::routing::put(api::preferences::update_preferences))
        .route("/export/{collection}", get(api::export::export_collection))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::auth::auth_middleware,
        ));

    let optional_auth_routes = Router::new()
        .route("/decks", get(api::deck::list_decks))
        .route("/decks/{id}", get(api::deck::get_deck))
        .route("/decks/{id}/cards", get(api::card::list_cards))
        .route("/notes", get(api::note::list_notes))
        .route("/notes/{id}", get(api::note::get_note))
        .route("/social/followers/{did}", get(api::social::get_followers))
        .route("/social/following/{did}", get(api::social::get_following))
        .route("/decks/{id}/comments", get(api::social::get_comments))
        .route("/feeds/trending", get(api::feed::get_feed_trending))
        .route("/search", get(api::search::search))
        .route("/discovery", get(api::search::discovery))
        .route("/users/{did}/profile", get(api::users::get_profile))
        .route("/remote/deck", get(api::deck::fetch_remote_deck))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::auth::optional_auth_middleware,
        ));

    let oauth_routes = Router::new()
        .route("/authorize", post(api::oauth::authorize))
        .route("/callback", get(api::oauth::callback))
        .route("/refresh", post(api::oauth::refresh))
        .with_state(oauth_state.clone());

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/health/ready", get(readiness_check))
        .route(
            "/.well-known/oauth-client-metadata",
            get(oauth::client_metadata::client_metadata_handler),
        )
        .route("/.well-known/atproto-did", get(well_known::atproto_did_handler))
        .route("/api/auth/login", post(api::auth::login))
        .route("/api/import/article", post(api::importer::import_article))
        .nest("/api/oauth", oauth_routes)
        .nest("/api", optional_auth_routes)
        .nest("/api", auth_routes)
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                .allow_headers(Any),
        )
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    tracing::info!("Listening on {}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}

/// Basic liveness check - returns 200 if the server is running.
///
/// For simple uptime monitoring and should always respond quickly without checking external dependencies.
async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "service": "malfestio-server",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// Readiness check - verifies the server can handle requests.
///
/// Checks database connectivity and other critical dependencies (load balancer health checks and deployment readiness probes).
async fn readiness_check(State(state): State<state::SharedState>) -> (StatusCode, Json<Value>) {
    match state.pool.get().await {
        Ok(client) => match client.query("SELECT 1", &[]).await {
            Ok(_) => (
                StatusCode::OK,
                Json(json!({
                    "status": "ready",
                    "service": "malfestio-server",
                    "version": env!("CARGO_PKG_VERSION"),
                    "checks": { "database": "ok" }
                })),
            ),
            Err(e) => {
                tracing::error!("Readiness check failed: database query error: {}", e);
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(json!({
                        "status": "not_ready",
                        "service": "malfestio-server",
                        "version": env!("CARGO_PKG_VERSION"),
                        "checks": { "database": "query_failed" }
                    })),
                )
            }
        },
        Err(e) => {
            tracing::error!("Readiness check failed: unable to get database connection: {}", e);
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "status": "not_ready",
                    "service": "malfestio-server",
                    "version": env!("CARGO_PKG_VERSION"),
                    "checks": { "database": "connection_failed" }
                })),
            )
        }
    }
}

pub struct AppError(malfestio_core::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self.0 {
            malfestio_core::Error::NotFound(_) => (StatusCode::NOT_FOUND, self.0.to_string()),
            malfestio_core::Error::InvalidArgument(_) => (StatusCode::BAD_REQUEST, self.0.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error".to_string()),
        };

        (status, Json(json!({ "error": error_message }))).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check_response_format() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let response = health_check().await.into_response();
            assert_eq!(response.status(), StatusCode::OK);
        });
    }

    #[test]
    fn test_readiness_check_with_unavailable_db() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let pool = db::create_mock_pool();
            let repos = state::Repositories::default();
            let config = state::AppConfig { pds_url: "https://test.example.com".to_string() };
            let app_state = state::AppState::new(pool, repos, config);
            let (status, _json) = readiness_check(State(app_state)).await;
            assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        });
    }
}
