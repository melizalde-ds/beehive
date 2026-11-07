//! PAP front door: discovery, sessions the customer approves, and the returns endpoint.
//! `/poppy/returns` is the gate's own handler, so agents get the exact same checks.

use std::num::NonZeroU64;

use axum::{
    Json,
    extract::{Path, State, rejection::JsonRejection},
    http::StatusCode,
};
use chrono::{Duration, SecondsFormat};
use serde_json::json;

use crate::{
    AppState,
    error::AppError,
    evidence::Event,
    session::Session,
    types::{
        ConsentDecision, EvidenceStep, PoppyDiscovery, PoppyDiscoveryEndpoints,
        PoppyDiscoveryMerchant, Scope, SessionMode, SessionRequest, SessionResponse, SessionStatus,
    },
};

/// Short expiry stands in for DPoP on the demo (brief: "DPoP is next").
const MAX_SESSION_SECONDS: NonZeroU64 = NonZeroU64::new(900).unwrap();

pub async fn discovery(State(state): State<AppState>) -> Json<PoppyDiscovery> {
    let shop = &state.seed.shop;
    Json(PoppyDiscovery {
        pap_version: "0.1-draft".to_string(),
        merchant: PoppyDiscoveryMerchant {
            id: shop.id.clone(),
            name: shop.name.clone(),
            platform: "Redo".to_string(),
        },
        endpoints: PoppyDiscoveryEndpoints {
            session: "/poppy/session".to_string(),
            returns: "/poppy/returns".to_string(),
        },
        scopes_supported: vec![Scope::PoppyRead, Scope::PoppyWrite],
        session_modes: vec![SessionMode::Guest, SessionMode::SignedIn],
        max_session_seconds: MAX_SESSION_SECONDS,
    })
}

/// The agent asks; nothing is allowed until the customer approves on the consent screen.
pub async fn create_session(
    State(state): State<AppState>,
    body: Result<Json<SessionRequest>, JsonRejection>,
) -> Result<Json<SessionResponse>, AppError> {
    let Json(req) = body?;
    if req.refund_cap < 0.0 {
        return Err(bad_request("The refund cap can't be negative."));
    }
    let email = req.customer_email.trim();
    let customer = state
        .seed
        .customers
        .iter()
        .find(|c| c.email.eq_ignore_ascii_case(email))
        .ok_or_else(|| bad_request("No customer with that email at this shop."))?;
    let id_matches = req
        .customer_id
        .as_deref()
        .is_none_or(|id| id == customer.id.as_str());
    if !id_matches || (req.mode == SessionMode::SignedIn && req.customer_id.is_none()) {
        return Err(bad_request(
            "A signed-in session needs the customer id that goes with that email.",
        ));
    }

    let mut session = Session::pending(
        &req.agent_id,
        &customer.id,
        req.scopes.clone(),
        cents(req.refund_cap),
    );
    session.events.push(Event::now(
        EvidenceStep::Discovery,
        json!({ "url": "/.well-known/poppy.json", "agent_id": *req.agent_id }),
    ));
    session.events.push(Event::now(
        EvidenceStep::Session,
        json!({ "session_id": session.id, "mode": req.mode, "customer_id": *customer.id,
                "requested_scopes": req.scopes, "requested_cap": req.refund_cap }),
    ));
    tracing::info!(
        session_id = session.id,
        agent_id = *req.agent_id,
        "session waiting for consent"
    );

    let res = response(&session);
    state.store().sessions.insert(session.id.clone(), session);
    Ok(Json(res))
}

/// The agent polls this until the customer decides.
pub async fn get_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Json<SessionResponse>, AppError> {
    let store = state.store();
    let session = store
        .sessions
        .get(&session_id)
        .ok_or_else(|| no_session(&session_id))?;
    Ok(Json(response(session)))
}

/// From the consent screen. The customer may lower the cap or drop scopes, never raise them.
pub async fn decide(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    body: Result<Json<ConsentDecision>, JsonRejection>,
) -> Result<Json<SessionResponse>, AppError> {
    let Json(decision) = body?;
    let mut store = state.store();
    let session = store
        .sessions
        .get_mut(&session_id)
        .ok_or_else(|| no_session(&session_id))?;
    if session.status != SessionStatus::PendingConsent {
        return Err(AppError::new(
            StatusCode::CONFLICT,
            "already_decided",
            "This session was already approved or denied.",
        ));
    }

    if !decision.approve {
        session.status = SessionStatus::Denied;
        session.events.push(Event::now(
            EvidenceStep::Consent,
            json!({ "approved": false }),
        ));
        return Ok(Json(response(session)));
    }

    let cap_cents = cents(decision.refund_cap);
    if decision.refund_cap < 0.0 || cap_cents > session.cap_cents {
        return Err(bad_request(
            "The cap can only go down from what the agent asked for.",
        ));
    }
    if decision.scopes.iter().any(|s| !session.scopes.contains(s)) {
        return Err(bad_request(
            "You can drop scopes the agent asked for, but not add new ones.",
        ));
    }

    session
        .activate(
            &state.keys,
            decision.scopes.clone(),
            cap_cents,
            Duration::seconds(MAX_SESSION_SECONDS.get() as i64),
        )
        .map_err(|e| {
            tracing::error!("signing the session token failed: {e}");
            internal("token")
        })?;
    let res = response(session);
    session.events.push(Event::now(
        EvidenceStep::Consent,
        json!({ "approved": true, "scopes": res.scopes, "refund_cap": res.refund_cap,
                "token_id": res.token_id, "expires_at": res.expires_at }),
    ));
    tracing::info!(session_id = session.id, cap_cents, "customer approved");
    Ok(Json(res))
}

fn response(s: &Session) -> SessionResponse {
    let pending = s.status == SessionStatus::PendingConsent;
    SessionResponse {
        session_id: s.id.clone(),
        status: s.status,
        consent_url: pending.then(|| format!("{}/consent/{}", web_url(), s.id)),
        session_token: s.token.clone(),
        token_id: s.token_id.clone(),
        scopes: s.scopes.clone(),
        refund_cap: s.cap_cents as f64 / 100.0,
        expires_at: s
            .expires_at
            .map(|t| t.to_rfc3339_opts(SecondsFormat::Secs, true)),
    }
}

/// Where the consent screen lives (the dashboard).
fn web_url() -> String {
    std::env::var("BEEHIVE_WEB_URL")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "http://localhost:5173".to_string())
}

fn cents(usd: f64) -> i64 {
    (usd * 100.0).round() as i64
}

fn bad_request(message: &str) -> AppError {
    AppError::new(StatusCode::BAD_REQUEST, "bad_request", message)
}

fn no_session(session_id: &str) -> AppError {
    AppError::not_found(format!("No session {session_id} at this shop."))
}

fn internal(what: &str) -> AppError {
    AppError::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        "internal",
        format!("Something went wrong building the {what}. Try again."),
    )
}
