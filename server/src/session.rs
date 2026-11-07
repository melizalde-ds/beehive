//! PAP session tokens: HS256 JWTs the gate checks on every request.
//! Phase 04's `POST /poppy/session` and consent screen build on `Session` and `Keys`.

use chrono::{DateTime, Duration, SecondsFormat, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    evidence::Event,
    types::{EvidenceStep, Scope, SessionStatus},
};

const DEV_SECRET: &str = "beehive-dev-secret-not-for-production";

pub struct Keys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl Keys {
    pub fn from_env() -> Self {
        let secret = std::env::var("BEEHIVE_JWT_SECRET")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| {
                tracing::warn!("BEEHIVE_JWT_SECRET is not set; using the fixed dev secret");
                DEV_SECRET.to_string()
            });
        Self {
            encoding: EncodingKey::from_secret(secret.as_bytes()),
            decoding: DecodingKey::from_secret(secret.as_bytes()),
        }
    }

    pub fn issue(&self, claims: &Claims) -> Result<String, jsonwebtoken::errors::Error> {
        jsonwebtoken::encode(&Header::new(Algorithm::HS256), claims, &self.encoding)
    }

    /// Signature only. Expiry is the gate's own check, with its own reason.
    pub fn verify(&self, token: &str) -> Option<Claims> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = false;
        jsonwebtoken::decode::<Claims>(token, &self.decoding, &validation)
            .ok()
            .map(|data| data.claims)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Session id.
    pub sid: String,
    /// Token id. Only the session's current one is accepted.
    pub jti: String,
    /// Customer id.
    pub sub: String,
    pub agent_id: String,
    pub scopes: Vec<Scope>,
    pub cap_cents: i64,
    pub iat: i64,
    pub exp: i64,
}

pub struct Session {
    pub id: String,
    pub customer_id: String,
    pub agent_id: String,
    pub status: SessionStatus,
    pub token_id: Option<String>,
    pub scopes: Vec<Scope>,
    pub cap_cents: i64,
    pub expires_at: Option<DateTime<Utc>>,
    /// Discovery, session and consent, copied to the front of every attempt's chain.
    pub events: Vec<Event>,
}

impl Session {
    pub fn pending(agent_id: &str, customer_id: &str, scopes: Vec<Scope>, cap_cents: i64) -> Self {
        Self {
            id: format!("ses_{}", uuid::Uuid::new_v4().simple()),
            customer_id: customer_id.to_string(),
            agent_id: agent_id.to_string(),
            status: SessionStatus::PendingConsent,
            token_id: None,
            scopes,
            cap_cents,
            expires_at: None,
            events: Vec::new(),
        }
    }

    /// The customer approved: mint a fresh token id and token. Any older token stops working.
    pub fn activate(
        &mut self,
        keys: &Keys,
        scopes: Vec<Scope>,
        cap_cents: i64,
        ttl: Duration,
    ) -> Result<String, jsonwebtoken::errors::Error> {
        let now = Utc::now();
        let exp = now + ttl;
        let claims = Claims {
            sid: self.id.clone(),
            jti: format!("tok_{}", uuid::Uuid::new_v4().simple()),
            sub: self.customer_id.clone(),
            agent_id: self.agent_id.clone(),
            scopes: scopes.clone(),
            cap_cents,
            iat: now.timestamp(),
            exp: exp.timestamp(),
        };
        let token = keys.issue(&claims)?;
        self.status = SessionStatus::Active;
        self.token_id = Some(claims.jti);
        self.scopes = scopes;
        self.cap_cents = cap_cents;
        self.expires_at = Some(exp);
        Ok(token)
    }
}

/// `BEEHIVE_DEMO_SESSION=1`: Kaylee has already approved muse-demo with a $50 cap.
pub fn demo_session(keys: &Keys) -> anyhow::Result<(Session, String)> {
    let scopes = vec![Scope::PoppyRead, Scope::PoppyWrite];
    let mut session = Session::pending("muse-demo", "c_1", scopes.clone(), 5000);
    session.events.push(Event::now(
        EvidenceStep::Discovery,
        json!({ "url": "/.well-known/poppy.json", "agent_id": "muse-demo" }),
    ));
    session.events.push(Event::now(
        EvidenceStep::Session,
        json!({ "session_id": session.id, "mode": "signed_in", "customer_id": "c_1",
                "requested_scopes": scopes, "requested_cap": 50.0 }),
    ));
    let token = session.activate(keys, scopes.clone(), 5000, Duration::hours(12))?;
    let expires_at = session
        .expires_at
        .map(|t| t.to_rfc3339_opts(SecondsFormat::Secs, true));
    session.events.push(Event::now(
        EvidenceStep::Consent,
        json!({ "approved": true, "scopes": scopes, "refund_cap": 50.0,
                "token_id": session.token_id, "expires_at": expires_at }),
    ));
    Ok((session, token))
}
