//! The gate: decides whether an agent may ask Redo for a return, in a fixed order of checks.
//! The first failed check blocks. A block is still a 200 with a `GateResponse` body.

use std::{
    convert::Infallible,
    time::{Duration, Instant},
};

use axum::{
    Json,
    extract::{State, rejection::JsonRejection},
    http::StatusCode,
    response::sse::{self, KeepAlive, Sse},
};
use serde_json::json;
use tokio_stream::{Stream, StreamExt, wrappers::BroadcastStream};

use crate::{
    AppState, Attempt, Store,
    error::AppError,
    evidence::{self, Event},
    session::{Keys, Session},
    types::{
        EvidenceStep, GateCheck, GateDecision, GateRequest, GateResponse, RedoResult, ReturnList,
        ReturnRecord, Scope, Seed, SessionStatus,
    },
};

const RATE_LIMIT: usize = 5;
const RATE_WINDOW: Duration = Duration::from_secs(60);

pub async fn file_return(
    State(state): State<AppState>,
    body: Result<Json<GateRequest>, JsonRejection>,
) -> Result<Json<GateResponse>, AppError> {
    let Json(req) = body?;
    let response = run(&state, req).await?;
    // Err only means no dashboard is listening.
    let _ = state.returns_changed.send(());
    Ok(Json(response))
}

/// SSE: one message per gate decision. The dashboard refetches `/returns` on each.
pub async fn stream_returns(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<sse::Event, Infallible>>> {
    // A lagged receiver yields an Err; it still means "the list changed", so send it on too.
    let changes = BroadcastStream::new(state.returns_changed.subscribe())
        .map(|_| Ok(sse::Event::default().data("changed")));
    Sse::new(changes).keep_alive(KeepAlive::default())
}

pub async fn list_returns(State(state): State<AppState>) -> Json<ReturnList> {
    let store = state.store();
    Json(ReturnList {
        shop_id: state.seed.shop.id.clone(),
        returns: store
            .attempts
            .iter()
            .rev()
            .map(|a| a.record.clone())
            .collect(),
    })
}

/// Shared by `/gate/returns` and phase 04's `/poppy/returns`.
pub async fn run(state: &AppState, req: GateRequest) -> Result<GateResponse, AppError> {
    let request_event = Event::now(
        EvidenceStep::Request,
        json!({ "request_id": *req.request_id, "order_id": *req.order_id,
                "reason": req.reason, "amount": req.amount }),
    );

    // Decide under the lock, but never hold it across the Redo call.
    let (return_id, mut events, mut record, reason) = {
        let mut store = state.store();
        let verdict = check(&mut store, &state.seed, &state.keys, &req);
        let return_id = store.next_return_id();
        let session = verdict
            .session_id
            .as_ref()
            .and_then(|id| store.sessions.get(id));
        let mut events = session.map(|s| s.events.clone()).unwrap_or_default();
        let mut record = record(&state.seed, &req, session, &return_id);
        events.push(request_event);
        events.push(decision_event(&verdict));

        let Some(reason) = verdict.pass_reason() else {
            let (check, reason) = verdict
                .blocked
                .clone()
                .unwrap_or((GateCheck::Session, String::new()));
            record.gate = GateDecision::Block;
            record.blocked_by = Some(check);
            record.gate_reason = reason.clone();
            // A request id is single-use once the token checks out: resending a blocked
            // request blocks as a replay, not on the same rule again.
            if verdict.passed.contains(&GateCheck::Idempotency) {
                store
                    .by_request
                    .insert(req.request_id.to_string(), return_id.clone());
            }
            return finish(&mut store, record, events, None, reason);
        };
        // Reserve so a duplicate sent while Redo is busy is still blocked.
        store
            .by_request
            .insert(req.request_id.to_string(), return_id.clone());
        store
            .passed_orders
            .insert(req.order_id.to_string(), return_id.clone());
        record.gate_reason = reason.clone();
        (return_id, events, record, reason)
    };

    // Redo has no field for "an agent filed this" (docs/findings.md), so the note carries
    // Beehive's proof: the attempt id, the approved cap, and the gate decision's chain hash.
    let gate_hash = evidence::head(&evidence::chain(events.clone()));
    let notes = format!(
        "Filed by agent {} for {} through Beehive {return_id}, within a ${:.2} cap the customer approved. Proof: gate decision hash {gate_hash}",
        req.agent_id.as_str(),
        record.customer_name,
        record.refund_cap,
    );
    let redo = state.redo.import_return(&state.seed, &req, &notes).await;
    let mut store = state.store();
    match redo {
        Ok(redo) => {
            events.push(Event::now(
                EvidenceStep::RedoResult,
                json!({ "return_id": *redo.return_id, "status": redo.status, "label_url": redo.label_url,
                        "redo_mode": state.redo.mode() }),
            ));
            record.redo = Some(redo.clone());
            finish(&mut store, record, events, Some(redo), reason)
        }
        Err(error) => {
            tracing::error!(return_id, %error, "Redo return import failed");
            store.by_request.remove(req.request_id.as_str());
            store.passed_orders.remove(req.order_id.as_str());
            Err(AppError::new(
                StatusCode::BAD_GATEWAY,
                "redo_unavailable",
                "Redo didn't take the return just now. Nothing was filed, so the agent can retry.",
            ))
        }
    }
}

fn finish(
    store: &mut Store,
    mut record: ReturnRecord,
    events: Vec<Event>,
    redo: Option<RedoResult>,
    reason: String,
) -> Result<GateResponse, AppError> {
    let links = evidence::chain(events);
    let chain_head = evidence::head(&links);
    record.chain_head = chain_head.clone();
    let response = GateResponse {
        return_id: record.return_id.clone().try_into().map_err(|_| {
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal",
                "Couldn't record the return.",
            )
        })?,
        gate: record.gate,
        blocked_by: record.blocked_by,
        reason,
        redo,
        chain_head,
    };
    tracing::info!(return_id = record.return_id, gate = ?record.gate, blocked_by = ?record.blocked_by, "gate decision");
    store.attempts.push(Attempt { record, links });
    Ok(response)
}

#[derive(Default)]
struct Verdict {
    passed: Vec<GateCheck>,
    blocked: Option<(GateCheck, String)>,
    session_id: Option<String>,
    cap_cents: i64,
}

impl Verdict {
    fn pass_reason(&self) -> Option<String> {
        self.blocked.is_none().then(|| {
            format!(
                "Within {} cap approved by customer",
                cap_label(self.cap_cents)
            )
        })
    }
}

fn check(store: &mut Store, seed: &Seed, keys: &Keys, req: &GateRequest) -> Verdict {
    let mut v = Verdict::default();
    macro_rules! block {
        ($check:expr, $($reason:tt)+) => {{
            v.blocked = Some(($check, format!($($reason)+)));
            return v;
        }};
    }
    let agent = req.agent_id.as_str();

    let Some(claims) = keys.verify(&req.session_token) else {
        block!(
            GateCheck::Session,
            "The session token isn't one this shop issued."
        );
    };
    let Some(session) = store.sessions.get(&claims.sid) else {
        block!(GateCheck::Session, "That session isn't known to this shop.");
    };
    v.session_id = Some(session.id.clone());
    v.cap_cents = claims.cap_cents;
    v.passed.push(GateCheck::Session);

    // Counted per signed token agent, after the session check, so requests without a
    // valid token can't use up a real agent's limit.
    let limited = claims.agent_id.as_str();
    let now = Instant::now();
    let hits = store.agent_hits.entry(limited.to_string()).or_default();
    hits.retain(|t| now.duration_since(*t) < RATE_WINDOW);
    if hits.len() >= RATE_LIMIT {
        block!(
            GateCheck::RateLimit,
            "{limited} sent more than {RATE_LIMIT} requests in a minute. Try again shortly."
        );
    }
    hits.push_back(now);
    v.passed.push(GateCheck::RateLimit);

    if !seed.agents.iter().any(|a| a.agent_id == agent) {
        block!(
            GateCheck::AgentIdentity,
            "{agent} isn't an agent this shop recognizes."
        );
    }
    v.passed.push(GateCheck::AgentIdentity);

    if claims.exp <= chrono::Utc::now().timestamp() {
        block!(
            GateCheck::Expiry,
            "The session expired. The customer needs to approve a new one."
        );
    }
    v.passed.push(GateCheck::Expiry);

    if claims.agent_id != agent {
        block!(
            GateCheck::Replay,
            "This token was issued to {}, not {agent}.",
            claims.agent_id
        );
    }
    if session.token_id.as_deref() != Some(claims.jti.as_str()) {
        block!(
            GateCheck::Replay,
            "This token was replaced by a newer one. Old tokens can't be reused."
        );
    }
    if session.status != SessionStatus::Active {
        block!(
            GateCheck::Replay,
            "This session has ended. Its token can't be reused."
        );
    }
    v.passed.push(GateCheck::Replay);

    if !claims.scopes.contains(&Scope::PoppyWrite) {
        block!(
            GateCheck::Scope,
            "The customer didn't allow this agent to file returns (no poppy:write)."
        );
    }
    v.passed.push(GateCheck::Scope);

    if let Some(original) = store.by_request.get(req.request_id.as_str()) {
        block!(
            GateCheck::Idempotency,
            "Duplicate of a request already filed as {original}. No second refund."
        );
    }
    v.passed.push(GateCheck::Idempotency);

    let amount = cents(req.amount);
    let order_id = req.order_id.as_str();
    // NaN rounds to 0 cents, so it lands here too.
    if amount <= 0 {
        block!(
            GateCheck::Order,
            "The return amount has to be more than $0."
        );
    }
    let Some(order) = seed
        .orders
        .iter()
        .find(|o| o.id == order_id && o.customer_id == claims.sub)
    else {
        block!(
            GateCheck::Order,
            "Order {order_id} isn't one of this customer's orders."
        );
    };
    // The price comes from the order, not the agent: a $36 claim on the $120 crate
    // would otherwise slip under a $50 cap. shortcut: whole-order returns only; price per
    // item once a return can name items.
    if amount != cents(order.total) {
        block!(
            GateCheck::Order,
            "{} doesn't match order {order_id}'s {} total. Returns cover the whole order.",
            usd(amount),
            usd(cents(order.total))
        );
    }
    if let Some(earlier) = store.passed_orders.get(order_id) {
        block!(
            GateCheck::Order,
            "Order {order_id} already has a return ({earlier})."
        );
    }
    v.passed.push(GateCheck::Order);

    if amount > claims.cap_cents {
        block!(
            GateCheck::RefundCap,
            "{} is over the {} cap the customer approved",
            usd(amount),
            cap_label(claims.cap_cents)
        );
    }
    v.passed.push(GateCheck::RefundCap);
    v
}

fn decision_event(v: &Verdict) -> Event {
    let mut checks: Vec<_> = v
        .passed
        .iter()
        .map(|c| json!({ "check": c, "ok": true }))
        .collect();
    if let Some((check, _)) = &v.blocked {
        checks.push(json!({ "check": check, "ok": false }));
    }
    let (gate, blocked_by, reason) = match &v.blocked {
        Some((check, reason)) => ("block", Some(*check), reason.clone()),
        None => ("pass", None, v.pass_reason().unwrap_or_default()),
    };
    Event::now(
        EvidenceStep::GateDecision,
        json!({ "gate": gate, "blocked_by": blocked_by, "reason": reason, "checks": checks }),
    )
}

/// Dashboard row. Names come from the seed; gate fields are filled in by the caller.
fn record(
    seed: &Seed,
    req: &GateRequest,
    session: Option<&Session>,
    return_id: &str,
) -> ReturnRecord {
    let agent_name = seed
        .agents
        .iter()
        .find(|a| a.agent_id == req.agent_id.as_str())
        .map_or_else(|| req.agent_id.to_string(), |a| a.name.clone());
    let customer_id = session.map(|s| s.customer_id.clone()).unwrap_or_default();
    let customer_name = seed
        .customers
        .iter()
        .find(|c| c.id == customer_id)
        .map_or_else(|| "Unknown".to_string(), |c| c.name.clone());
    let product_name = seed
        .orders
        .iter()
        .find(|o| o.id == req.order_id.as_str())
        .and_then(|o| o.items.first())
        .and_then(|item| seed.products.iter().find(|p| p.id == item.product_id))
        .map_or_else(|| "Unknown order".to_string(), |p| p.name.clone());
    ReturnRecord {
        return_id: return_id.to_string(),
        at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        agent_id: req.agent_id.to_string(),
        agent_name,
        customer_id,
        customer_name,
        order_id: req.order_id.to_string(),
        product_name,
        amount: req.amount,
        reason: req.reason,
        allowed_scopes: session.map(|s| s.scopes.clone()).unwrap_or_default(),
        refund_cap: session.map_or(0.0, |s| s.cap_cents as f64 / 100.0),
        gate: GateDecision::Pass,
        blocked_by: None,
        gate_reason: String::new(),
        redo: None,
        chain_head: String::new(),
    }
}

fn cents(usd: f64) -> i64 {
    (usd * 100.0).round() as i64
}

fn usd(cents: i64) -> String {
    format!("${}.{:02}", cents / 100, cents % 100)
}

/// "$50" for whole dollars, "$49.50" otherwise.
fn cap_label(cents: i64) -> String {
    if cents % 100 == 0 {
        format!("${}", cents / 100)
    } else {
        usd(cents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::{self, Session};

    struct Shop {
        store: Store,
        seed: Seed,
        keys: Keys,
        token: String,
    }

    /// Kaylee has approved muse-demo with a $50 cap, as in the demo.
    fn shop() -> Shop {
        let seed: Seed = serde_json::from_str(crate::SEED_JSON).unwrap();
        let keys = Keys::from_env();
        let (session, token) = session::demo_session(&keys).unwrap();
        let mut store = Store::default();
        store.sessions.insert(session.id.clone(), session);
        Shop {
            store,
            seed,
            keys,
            token,
        }
    }

    fn request(token: &str, request_id: &str, order_id: &str, amount: f64) -> GateRequest {
        serde_json::from_value(json!({
            "request_id": request_id, "session_token": token, "agent_id": "muse-demo",
            "order_id": order_id, "reason": "damaged", "amount": amount,
        }))
        .unwrap()
    }

    fn blocked_by(shop: &mut Shop, req: &GateRequest) -> Option<GateCheck> {
        check(&mut shop.store, &shop.seed, &shop.keys, req)
            .blocked
            .map(|(check, _)| check)
    }

    fn id() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    #[test]
    fn within_the_cap_passes() {
        let mut shop = shop();
        let req = request(&shop.token.clone(), &id(), "o_1", 36.0);
        assert_eq!(blocked_by(&mut shop, &req), None);
    }

    #[test]
    fn over_the_cap_blocks() {
        let mut shop = shop();
        let req = request(&shop.token.clone(), &id(), "o_2", 120.0);
        assert_eq!(blocked_by(&mut shop, &req), Some(GateCheck::RefundCap));
    }

    #[test]
    fn a_resent_request_id_blocks_as_a_duplicate() {
        let mut shop = shop();
        let request_id = id();
        shop.store
            .by_request
            .insert(request_id.clone(), "bh_0001".into());
        let req = request(&shop.token.clone(), &request_id, "o_1", 36.0);
        assert_eq!(blocked_by(&mut shop, &req), Some(GateCheck::Idempotency));
    }

    #[test]
    fn an_expired_token_blocks() {
        let mut shop = shop();
        let scopes = vec![Scope::PoppyRead, Scope::PoppyWrite];
        let mut session = Session::pending("muse-demo", "c_1", scopes.clone(), 5000);
        let token = session
            .activate(&shop.keys, scopes, 5000, chrono::Duration::minutes(-1))
            .unwrap();
        shop.store.sessions.insert(session.id.clone(), session);
        let req = request(&token, &id(), "o_1", 36.0);
        assert_eq!(blocked_by(&mut shop, &req), Some(GateCheck::Expiry));
    }

    #[test]
    fn a_claim_below_the_order_price_blocks() {
        let mut shop = shop();
        // $36 claimed on the $120 crate would fit under the $50 cap.
        let req = request(&shop.token.clone(), &id(), "o_2", 36.0);
        assert_eq!(blocked_by(&mut shop, &req), Some(GateCheck::Order));
    }

    #[test]
    fn zero_or_negative_amounts_block() {
        let mut shop = shop();
        for amount in [0.0, -5.0] {
            let req = request(&shop.token.clone(), &id(), "o_1", amount);
            assert_eq!(blocked_by(&mut shop, &req), Some(GateCheck::Order));
        }
    }

    #[test]
    fn junk_tokens_cant_use_up_the_agents_rate_limit() {
        let mut shop = shop();
        for _ in 0..RATE_LIMIT + 2 {
            let junk = request("not-a-token", &id(), "o_1", 36.0);
            assert_eq!(blocked_by(&mut shop, &junk), Some(GateCheck::Session));
        }
        // The real agent still gets its full limit, then the next request blocks.
        for _ in 0..RATE_LIMIT {
            let req = request(&shop.token.clone(), &id(), "o_2", 120.0);
            assert_eq!(blocked_by(&mut shop, &req), Some(GateCheck::RefundCap));
        }
        let req = request(&shop.token.clone(), &id(), "o_2", 120.0);
        assert_eq!(blocked_by(&mut shop, &req), Some(GateCheck::RateLimit));
    }
}
