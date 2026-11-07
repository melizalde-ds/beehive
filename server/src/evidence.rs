//! SHA-256 hash chain per attempt, plus `verify` and the demo-only tamper.

use axum::{
    Json,
    extract::{Path, State, rejection::JsonRejection},
};
use chrono::{SecondsFormat, Utc};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

use crate::{
    AppState,
    error::AppError,
    types::{Evidence, EvidenceLink, EvidenceStep, TamperRequest, VerifyResult},
};

const GENESIS: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// One step before it is chained.
#[derive(Clone)]
pub struct Event {
    pub step: EvidenceStep,
    pub at: String,
    pub data: Map<String, Value>,
}

impl Event {
    /// `data` must be a JSON object; anything else is stored as an empty object.
    pub fn now(step: EvidenceStep, data: Value) -> Self {
        let data = match data {
            Value::Object(map) => map,
            _ => Map::new(),
        };
        Self {
            step,
            at: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
            data,
        }
    }
}

pub fn chain(events: Vec<Event>) -> Vec<EvidenceLink> {
    let mut prev = GENESIS.to_string();
    let mut links = Vec::with_capacity(events.len());
    for (index, event) in events.into_iter().enumerate() {
        let index = index as u64;
        let hash = link_hash(&prev, index, event.step, &event.at, &event.data);
        links.push(EvidenceLink {
            index,
            step: event.step,
            at: event.at,
            data: event.data,
            prev_hash: std::mem::replace(&mut prev, hash.clone()),
            hash,
        });
    }
    links
}

pub fn head(links: &[EvidenceLink]) -> String {
    links
        .last()
        .map_or_else(|| GENESIS.to_string(), |l| l.hash.clone())
}

/// serde_json maps are sorted (no `preserve_order`), so `data` hashes the same every time.
fn link_hash(
    prev: &str,
    index: u64,
    step: EvidenceStep,
    at: &str,
    data: &Map<String, Value>,
) -> String {
    let step = serde_json::to_string(&step).unwrap_or_default();
    let data = serde_json::to_string(data).unwrap_or_default();
    hex::encode(Sha256::digest(format!("{prev}|{index}|{step}|{at}|{data}")))
}

pub fn verify(return_id: &str, links: &[EvidenceLink]) -> VerifyResult {
    let mut prev = GENESIS;
    for link in links {
        let recomputed = link_hash(&link.prev_hash, link.index, link.step, &link.at, &link.data);
        if link.prev_hash != prev || recomputed != link.hash {
            let step = serde_json::to_value(link.step).ok();
            let step_name = step.as_ref().and_then(Value::as_str).unwrap_or("?");
            return VerifyResult {
                return_id: return_id.to_string(),
                ok: false,
                links_checked: link.index + 1,
                broken_at: Some(link.index),
                broken_step: Some(link.step),
                message: format!(
                    "Link {} ({step_name}) was changed after it was recorded",
                    link.index
                ),
            };
        }
        prev = &link.hash;
    }
    VerifyResult {
        return_id: return_id.to_string(),
        ok: true,
        links_checked: links.len() as u64,
        broken_at: None,
        broken_step: None,
        message: format!("All {} links check out", links.len()),
    }
}

/// Demo only: bump the first digit in the link's data (9 wraps to 0), or the first
/// letter if there is no digit. Stored without rehashing, so `verify` catches it.
fn tamper(link: &mut EvidenceLink) -> Result<(), AppError> {
    let mut bytes = serde_json::to_vec(&link.data).unwrap_or_default();
    let spot = bytes
        .iter()
        .position(u8::is_ascii_digit)
        .or_else(|| bytes.iter().position(u8::is_ascii_alphabetic));
    let Some(i) = spot else {
        return Err(AppError::new(
            axum::http::StatusCode::UNPROCESSABLE_ENTITY,
            "nothing_to_tamper",
            "That link has no data to change.",
        ));
    };
    bytes[i] = match bytes[i] {
        b'9' => b'0',
        b'z' => b'a',
        b'Z' => b'A',
        b => b + 1,
    };
    link.data = serde_json::from_slice(&bytes).map_err(|_| {
        AppError::new(
            axum::http::StatusCode::UNPROCESSABLE_ENTITY,
            "nothing_to_tamper",
            "That link's data can't be changed by one byte.",
        )
    })?;
    Ok(())
}

pub async fn get_evidence(
    State(state): State<AppState>,
    Path(return_id): Path<String>,
) -> Result<Json<Evidence>, AppError> {
    let store = state.store();
    let attempt = store.attempt(&return_id)?;
    Ok(Json(Evidence {
        return_id,
        chain_head: head(&attempt.links),
        links: attempt.links.clone(),
    }))
}

pub async fn verify_chain(
    State(state): State<AppState>,
    Path(return_id): Path<String>,
) -> Result<Json<VerifyResult>, AppError> {
    let store = state.store();
    let attempt = store.attempt(&return_id)?;
    Ok(Json(verify(&return_id, &attempt.links)))
}

pub async fn tamper_link(
    State(state): State<AppState>,
    Path(return_id): Path<String>,
    body: Result<Json<TamperRequest>, JsonRejection>,
) -> Result<Json<Evidence>, AppError> {
    let Json(body) = body?;
    let mut store = state.store();
    let attempt = store.attempt_mut(&return_id)?;
    let count = attempt.links.len();
    let link = attempt
        .links
        .get_mut(body.link_index as usize)
        .ok_or_else(|| {
            AppError::not_found(format!(
                "{return_id} has {count} links, so there's no link {}.",
                body.link_index
            ))
        })?;
    tamper(link)?;
    tracing::warn!(
        return_id,
        link_index = body.link_index,
        "evidence tampered (demo)"
    );
    Ok(Json(Evidence {
        return_id,
        chain_head: head(&attempt.links),
        links: attempt.links.clone(),
    }))
}
