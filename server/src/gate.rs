//! The gate: decides whether an agent may ask Redo for a return, in a fixed order of checks.
//! The first failed check blocks. A block is still a 200 with a `GateResponse` body.

use std::time::{Duration, Instant};

use axum::{
    Json,
    extract::{State, rejection::JsonRejection},
    http::StatusCode,
};
use serde_json::json;

use crate::{
    AppState, Attempt, Store,
    error::AppError,
    evidence::{self, Event},
    redo,
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
    Ok(Json(run(&state, req).await?))
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

    let redo = redo::import_return(&req).await;
    let mut store = state.store();
    match redo {
        Ok(redo) => {
            events.push(Event::now(
                EvidenceStep::RedoResult,
                json!({ "return_id": *redo.return_id, "status": redo.status, "label_url": redo.label_url }),
            ));
            record.redo = Some(redo.clone());
            finish(&mut store, record, events, Some(redo), reason)
        }
        Err(error) => {
            tracing::error!(return_id, error, "Redo return import failed");
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

    // Only requests that get past this check count toward the limit.
    let now = Instant::now();
    let hits = store.agent_hits.entry(agent.to_string()).or_default();
    hits.retain(|t| now.duration_since(*t) < RATE_WINDOW);
    if hits.len() >= RATE_LIMIT {
        block!(
            GateCheck::RateLimit,
            "{agent} sent more than {RATE_LIMIT} requests in a minute. Try again shortly."
        );
    }
    hits.push_back(now);
    v.passed.push(GateCheck::RateLimit);

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
    if amount > cents(order.total) {
        block!(
            GateCheck::Order,
            "{} is more than order {order_id}'s {} total.",
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
