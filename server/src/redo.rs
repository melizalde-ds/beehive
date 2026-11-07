//! Stand-in for the Redo client until phase 03 lands the mock/real enum.
//! It answers like `redo-fixtures/return-import.json` + `return-status.json`.

use sha2::{Digest, Sha256};

use crate::types::{GateRequest, RedoResult, RedoStatus};

/// `request_id` is Redo's `idempotencyKey`, so the same key gives the same return id.
pub async fn import_return(req: &GateRequest) -> Result<RedoResult, String> {
    let hash = hex::encode(Sha256::digest(req.request_id.as_bytes()));
    Ok(RedoResult {
        return_id: hash[..24]
            .to_string()
            .try_into()
            .map_err(|e| format!("{e}"))?,
        status: RedoStatus::Open,
        label_url: None,
    })
}
