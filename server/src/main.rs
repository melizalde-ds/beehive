mod error;
mod evidence;
mod gate;
mod redo;
mod session;
mod types;

use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex, MutexGuard, PoisonError},
    time::Instant,
};

use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use tower_http::cors::CorsLayer;

use crate::{error::AppError, session::Session};

const SEED_JSON: &str = include_str!("../../seed/cache-valley-honey.json");

#[derive(Clone)]
pub struct AppState {
    seed: Arc<types::Seed>,
    keys: Arc<session::Keys>,
    redo: Arc<redo::Redo>,
    store: Arc<Mutex<Store>>,
}

impl AppState {
    /// A panic while holding the lock leaves the data usable, so recover instead of failing.
    fn store(&self) -> MutexGuard<'_, Store> {
        self.store.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// In memory on purpose: a restart gives a clean demo.
#[derive(Default)]
pub struct Store {
    sessions: HashMap<String, Session>,
    /// Oldest first; `/returns` lists them newest first.
    attempts: Vec<Attempt>,
    /// request_id → return_id of the attempt that passed with it.
    by_request: HashMap<String, String>,
    /// order_id → return_id of the attempt that passed for it.
    passed_orders: HashMap<String, String>,
    agent_hits: HashMap<String, VecDeque<Instant>>,
    next_id: u32,
}

pub struct Attempt {
    record: types::ReturnRecord,
    links: Vec<types::EvidenceLink>,
}

impl Store {
    fn next_return_id(&mut self) -> String {
        self.next_id += 1;
        format!("bh_{:04}", self.next_id)
    }

    fn attempt(&self, return_id: &str) -> Result<&Attempt, AppError> {
        self.attempts
            .iter()
            .find(|a| a.record.return_id == return_id)
            .ok_or_else(|| AppError::not_found(format!("No return {return_id} at this shop.")))
    }

    fn attempt_mut(&mut self, return_id: &str) -> Result<&mut Attempt, AppError> {
        self.attempts
            .iter_mut()
            .find(|a| a.record.return_id == return_id)
            .ok_or_else(|| AppError::not_found(format!("No return {return_id} at this shop.")))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let seed: types::Seed = serde_json::from_str(SEED_JSON)?;
    let keys = session::Keys::from_env();
    let mut store = Store::default();
    if std::env::var("BEEHIVE_DEMO_SESSION").is_ok_and(|v| v == "1") {
        let (session, token) = session::demo_session(&keys)?;
        tracing::info!(
            session_id = session.id,
            "demo session for muse-demo (c_1, $50 cap)"
        );
        tracing::info!("demo session token: {token}");
        store.sessions.insert(session.id.clone(), session);
    }
    let redo = redo::Redo::from_env(&seed).await;
    let state = AppState {
        seed: Arc::new(seed),
        keys: Arc::new(keys),
        redo: Arc::new(redo),
        store: Arc::new(Mutex::new(store)),
    };

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/shop", get(shop))
        .route("/gate/returns", post(gate::file_return))
        .route("/returns", get(gate::list_returns))
        .route("/evidence/{return_id}", get(evidence::get_evidence))
        .route("/evidence/{return_id}/tamper", post(evidence::tamper_link))
        .route("/verify/{return_id}", get(evidence::verify_chain))
        .fallback(not_found)
        .layer(CorsLayer::permissive())
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

async fn not_found() -> AppError {
    AppError::not_found("That page isn't part of Beehive.")
}
