//! PAP front door: discovery, sessions the customer approves, and the returns endpoint.
//! `/poppy/returns` is the gate's own handler, so agents get the exact same checks.

use std::{num::NonZeroU64, sync::LazyLock};

use axum::{
    Json,
    extract::{Path, State, rejection::JsonRejection},
    http::{HeaderMap, StatusCode, Uri, header::AUTHORIZATION},
};
use chrono::{Duration, SecondsFormat};
use serde_json::{Value, json};

use crate::{
    AppState,
    error::AppError,
    evidence::Event,
    session::Session,
    types::{
        ConsentDecision, CustomerOrder, EvidenceStep, OrderList, PoppyDiscovery,
        PoppyDiscoveryAgent, PoppyDiscoveryAgentProtocolsItem, PoppyDiscoveryAuth,
        PoppyDiscoveryEndpoints, PoppyDiscoveryMerchant, PoppyDiscoveryOrganization,
        PoppyDiscoveryReturnRequest, Scope, SessionMode, SessionRequest, SessionResponse,
        SessionStatus,
    },
};

/// Short expiry stands in for DPoP on the demo (brief: "DPoP is next").
const MAX_SESSION_SECONDS: NonZeroU64 = NonZeroU64::new(900).unwrap();

/// What a return needs, read from the contract so it can't drift from `GateRequest` and `Reason`.
static RETURN_REQUEST: LazyLock<PoppyDiscoveryReturnRequest> = LazyLock::new(|| {
    let schema: Value = serde_json::from_str(include_str!("../../schema/beehive.schema.json"))
        .expect("the schema typify already parsed is JSON");
    let defs = &schema["definitions"];
    serde_json::from_value(json!({
        "fields": defs["GateRequest"]["required"],
        "reasons": defs["Reason"]["enum"],
    }))
    .expect("the schema has GateRequest.required and Reason.enum")
});

pub async fn discovery(State(state): State<AppState>) -> Json<PoppyDiscovery> {
    let shop = &state.seed.shop;
    let issuer = issuer();
    Json(PoppyDiscovery {
        pap_version: "0.1-draft".to_string(),
        protocol_version: "0.3".to_string(),
        organization: PoppyDiscoveryOrganization {
            name: shop.name.clone(),
            domain: domain(),
        },
        agent: PoppyDiscoveryAgent {
            protocols: vec![PoppyDiscoveryAgentProtocolsItem {
                type_: "poppy".to_string(),
                endpoint: format!("{issuer}/poppy/session"),
                access: "session".to_string(),
            }],
        },
        auth: PoppyDiscoveryAuth { issuer },
        return_request: RETURN_REQUEST.clone(),
        merchant: PoppyDiscoveryMerchant {
            id: shop.id.clone(),
            name: shop.name.clone(),
            platform: "Redo".to_string(),
        },
        endpoints: PoppyDiscoveryEndpoints {
            session: "/poppy/session".to_string(),
            returns: "/poppy/returns".to_string(),
            orders: "/poppy/orders".to_string(),
        },
        scopes_supported: vec![Scope::PoppyRead, Scope::PoppyWrite],
        session_modes: vec![SessionMode::Guest, SessionMode::SignedIn],
        max_session_seconds: MAX_SESSION_SECONDS,
    })
}

/// RFC 9728 metadata for the shop's domain. The PAP SDK reads it to find the issuer.
pub async fn protected_resource() -> Json<Value> {
    Json(json!({
        "resource": format!("https://{}", domain()),
        "authorization_servers": [issuer()],
    }))
}

/// RFC 8414 metadata with the fields the PAP SDK (draft 0.3) requires.
/// shortcut: static only, the brief cuts real OAuth; the token endpoints 404, so an agent
/// that tries to sign in stops there. Upgrade when sign-in is in scope.
pub async fn oauth_metadata() -> Json<Value> {
    let issuer = issuer();
    let domain = domain();
    Json(json!({
        "issuer": issuer,
        "token_endpoint": format!("{issuer}/oauth/token"),
        "revocation_endpoint": format!("{issuer}/oauth/revoke"),
        "grant_types_supported": ["refresh_token"],
        "scopes_supported": [Scope::PoppyRead, Scope::PoppyWrite],
        "protected_resources": [format!("https://{domain}")],
        "poppy_domains": [domain],
    }))
}

/// `GET /poppy/session`: browser-driven agents (Muse) land here with a GET, so say how to POST.
pub async fn session_help() -> Json<Value> {
    let parse = |s: &str| serde_json::from_str::<Value>(s).unwrap_or(Value::Null);
    Json(json!({
        "message": "Open a PAP session with POST /poppy/session and a JSON body like `example`. The customer approves it at the returned consent_url. Then GET /poppy/orders with `Authorization: Bearer <session_token>` to see the customer's orders, and POST each return to /poppy/returns with the session_token, like `return_example`.",
        "method": "POST",
        "content_type": "application/json",
        "example": parse(include_str!("../../schema/examples/SessionRequest.json")),
        "return_example": parse(include_str!("../../schema/examples/GateRequest.json")),
        "discovery": "/.well-known/poppy.json",
    }))
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

/// The token's customer's orders, so an agent can pick one to return. Needs poppy:read.
pub async fn list_orders(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<OrderList>, AppError> {
    let unauthorized = |message: &str| {
        AppError::new(
            StatusCode::UNAUTHORIZED,
            "unauthorized",
            message.to_string(),
        )
    };
    let token = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or_else(|| {
            unauthorized("Send the session token as `Authorization: Bearer <token>`.")
        })?;
    let claims = state
        .keys
        .verify(token)
        .ok_or_else(|| unauthorized("The session token isn't one this shop issued."))?;
    let store = state.store();
    let session = store
        .sessions
        .get(&claims.sid)
        .filter(|s| s.status == SessionStatus::Active)
        .filter(|s| s.token_id.as_deref() == Some(claims.jti.as_str()))
        .ok_or_else(|| unauthorized("This session isn't active, or the token was replaced."))?;
    if claims.exp <= chrono::Utc::now().timestamp() {
        return Err(unauthorized(
            "The session expired. The customer needs to approve a new one.",
        ));
    }
    if !claims.scopes.contains(&Scope::PoppyRead) {
        return Err(AppError::new(
            StatusCode::FORBIDDEN,
            "forbidden",
            "The customer didn't allow this agent to see their orders (no poppy:read).",
        ));
    }
    let seed = &state.seed;
    let mut orders: Vec<CustomerOrder> = seed
        .orders
        .iter()
        .filter(|order| order.customer_id == session.customer_id)
        .map(|order| CustomerOrder {
            id: order.id.clone(),
            product_name: order
                .items
                .first()
                .and_then(|item| seed.products.iter().find(|p| p.id == item.product_id))
                .map_or_else(|| "Unknown product".to_string(), |p| p.name.clone()),
            placed_at: order.placed_at.clone(),
            delivered_at: order.delivered_at.clone(),
            total: order.total,
            return_id: store.passed_orders.get(&order.id).cloned(),
        })
        .collect();
    // shortcut: sorts RFC 3339 as text, right while every seed date shares one offset; parse if that changes.
    orders.sort_by(|a, b| b.placed_at.cmp(&a.placed_at));
    Ok(Json(OrderList {
        customer_id: session.customer_id.clone(),
        orders,
    }))
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

/// One string for `auth.issuer` and both metadata documents: the SDK compares them exactly.
fn issuer() -> String {
    web_url().trim_end_matches('/').to_string()
}

/// The host part of `BEEHIVE_WEB_URL`, which is the host that serves `poppy.json`.
fn domain() -> String {
    web_url()
        .parse::<Uri>()
        .ok()
        .and_then(|u| u.host().map(str::to_string))
        .unwrap_or_default()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Reason;

    #[test]
    fn return_request_comes_from_the_contract() {
        assert!(RETURN_REQUEST.fields.iter().any(|f| f == "order_id"));
        assert!(RETURN_REQUEST.reasons.contains(&Reason::Damaged));
    }
}
