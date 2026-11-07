//! Rehearsal helpers. Everything is in memory, so a reset is a fresh `Store`, and a seed
//! replays the demo path through the real gate so a cold dashboard already shows the wow.

use axum::{Json, extract::State};
use serde_json::json;

use crate::{
    AppState, Store, gate, session,
    types::{GateRequest, ReturnList},
};

/// `POST /demo/reset`: forget every session, return and evidence link, and the replay and
/// rate-limit memory. Doesn't re-seed: the next agent run would block its own $36.
pub async fn reset(State(state): State<AppState>) -> Json<ReturnList> {
    *state.store() = Store::default();
    tracing::info!("demo reset: store cleared");
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
        request(&candles_id, "o_1", "damaged", 36.0)?,
        request(&crate_id, "o_2", "changed_mind", 120.0)?,
        request(&candles_id, "o_1", "damaged", 36.0)?, // the replay
    ];
    for req in demo {
        // The gate only errors when Redo refuses the passed return; its message is logged there.
        gate::run(state, req)
            .await
            .map_err(|_| anyhow::anyhow!("Redo didn't take the seeded $36 return"))?;
    }
    let _ = state.returns_changed.send(());
    Ok(())
}
