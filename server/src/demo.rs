//! Rehearsal helpers. Everything is in memory, so a reset is a fresh `Store`, and a seed
//! replays the demo path through the real gate so a cold dashboard already shows the wow.

use std::collections::HashMap;

use axum::{
    Json,
    extract::{Query, State},
};
use serde_json::json;

use crate::{
    AppState, Store, gate, session,
    types::{GateRequest, ReturnList},
};

/// `POST /demo/reset`: forget every session, return and evidence link, and the replay and
/// rate-limit memory, then re-seed the cold open so any viewer can undo a tamper.
/// `?empty=1` (the agent's `--reset`) skips the re-seed, so the agent's own $36 can pass.
pub async fn reset(
    State(state): State<AppState>,
    Query(query): Query<HashMap<String, String>>,
) -> Json<ReturnList> {
    *state.store() = Store::default();
    if query.get("empty").is_some_and(|v| v == "1") {
        tracing::info!("demo reset: store cleared");
    } else {
        match seed(&state).await {
            Ok(()) => tracing::info!("demo reset: store re-seeded"),
            Err(error) => tracing::warn!(%error, "demo re-seed failed; the dashboard is empty"),
        }
    }
    let _ = state.returns_changed.send(());
    gate::list_returns(State(state)).await
}

/// `BEEHIVE_DEMO_SEED=1`: the $36 pass, the $120 cap block and the replay block.
pub async fn seed(state: &AppState) -> anyhow::Result<()> {
    let (session, token) = session::demo_session(&state.keys)?;
    state.store().sessions.insert(session.id.clone(), session);
    let request = |request_id: &str, order_id: &str, reason: &str, amount: f64| {
        serde_json::from_value::<GateRequest>(json!({
            "request_id": request_id, "session_token": token, "agent_id": "muse-demo",
            "order_id": order_id, "reason": reason, "amount": amount,
        }))
    };
    let candles_id = uuid::Uuid::new_v4().to_string();
    let crate_id = uuid::Uuid::new_v4().to_string();
    let demo = [
        request(&candles_id, "o_5", "damaged", 36.0)?,
        request(&crate_id, "o_6", "changed_mind", 120.0)?,
        request(&candles_id, "o_5", "damaged", 36.0)?, // the replay
    ];
    for req in demo {
        // The gate only errors when Redo refuses the passed return; its message is logged there.
        gate::run(state, req)
            .await
            .map_err(|_| anyhow::anyhow!("Redo didn't take the seeded $36 return"))?;
    }
    // The seeded requests shouldn't use up the live agent's rate limit.
    state.store().agent_hits.clear();
    let _ = state.returns_changed.send(());
    Ok(())
}
