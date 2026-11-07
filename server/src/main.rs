mod types;

use std::sync::Arc;

use axum::{Json, Router, extract::State, http::StatusCode, routing::get};

const SEED_JSON: &str = include_str!("../../seed/cache-valley-honey.json");

#[derive(Clone)]
struct AppState {
    seed: Arc<types::Seed>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let seed: types::Seed = serde_json::from_str(SEED_JSON)?;
    let state = AppState {
        seed: Arc::new(seed),
    };

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/shop", get(shop))
        .fallback(not_found)
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("beehive listening on http://{addr}");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn shop(State(state): State<AppState>) -> Json<types::Seed> {
    Json(state.seed.as_ref().clone())
}

async fn not_found() -> (StatusCode, Json<types::ApiError>) {
    let body = types::ApiError {
        error: "not_found".to_string(),
        message: "That page isn't part of Beehive.".to_string(),
    };
    (StatusCode::NOT_FOUND, Json(body))
}
