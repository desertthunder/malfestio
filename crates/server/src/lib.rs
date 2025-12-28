pub mod api;
pub mod db;
pub mod middleware;
pub mod repository;
pub mod state;

use axum::http::Method;
use axum::{
    Json, Router,
    http::StatusCode,
    middleware as axum_middleware,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde_json::json;
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

    let pool = db::create_pool().map_err(|e| {
        tracing::error!("Failed to create database pool: {}", e);
        malfestio_core::Error::Database(format!("Failed to create database pool: {}", e))
    })?;

    tracing::info!("Database connection pool created");

    let state = state::AppState::new(pool);

    let auth_routes = Router::new()
        .route("/me", get(api::auth::me))
        .route("/decks", post(api::deck::create_deck))
        .route("/notes", post(api::note::create_note))
        .route("/cards", post(api::card::create_card))
        .layer(axum_middleware::from_fn(middleware::auth::auth_middleware));

    let optional_auth_routes = Router::new()
        .route("/decks", get(api::deck::list_decks))
        .route("/decks/{id}", get(api::deck::get_deck))
        .route("/decks/{id}/cards", get(api::card::list_cards))
        .route("/notes", get(api::note::list_notes))
        .route("/notes/{id}", get(api::note::get_note))
        .layer(axum_middleware::from_fn(middleware::auth::optional_auth_middleware));

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/auth/login", post(api::auth::login))
        .route("/api/import/article", post(api::importer::import_article))
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

async fn health_check() -> impl IntoResponse {
    Json(json!({ "status": "ok", "version": env!("CARGO_PKG_VERSION") }))
}

pub struct AppError(malfestio_core::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self.0 {
            malfestio_core::Error::NotFound(_) => (StatusCode::NOT_FOUND, self.0.to_string()),
            malfestio_core::Error::InvalidArgument(_) => (StatusCode::BAD_REQUEST, self.0.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error".to_string()),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}
