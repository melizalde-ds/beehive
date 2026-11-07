# Phase 03: Redo client

**Goal:** A passed gate request becomes a real Redo return (or a mock shaped like `redo-fixtures/`), switched by `REDO_MODE`.
**Brief requirements covered:** MVP piece 2, "On pass: hands off to Redo through a `RedoClient`" (mock now, real sandbox with keys). Seed orders through `custom/orders`.
**Depends on:** phase 02 (`gate::run` calls `redo::import_return`)
**Touches only:** `server/` (`redo.rs`, `main.rs`, the call site in `gate.rs`, `Cargo.toml`), `plans/phase-03-redo.md`, `docs/findings.md`

## Checklist

- [x] `reqwest` (json, default rustls)
- [x] `redo.rs`: `Redo` enum, `Mock` or `Real`, built from `REDO_MODE` / `REDO_BASE` / `REDO_API_SECRET` / `REDO_STORE_ID`
- [x] Real seed at startup: one `products/bulk-upload` for all 4 products (retry on 409), keep sku → `prd_` id; then `custom/orders` per order
- [x] Per return: `returns/bulk` with `idempotencyKey` = `request_id` (check `results[0].status`), then `GET /returns/{id}/status`, then a best-effort label from `GET /returns/{id}` (null on error)
- [x] Mock: same id for the same key (24 hex chars, like Redo), status `open`, label null
- [x] `RedoError` keeps HTTP status + Redo's `title` / `detail` / per-item `error`; the gate logs it and answers 502
- [x] `redo_result` evidence link records which mode answered (`redo_mode`)
- [x] One small test: the parsers read the real fixtures
- [x] Walk: mock, then real ($36 passes with a real Redo id, $120 and the replay never reach Redo)

## Questions asked

- Seed fields Redo needs (sku, first/last name, address, line ids)? **Derive in code**, no schema change.
- New crates? **`reqwest` only.** No `dotenvy`: export the `REDO_*` vars in the shell for real mode.
- Mock `label_url`? **null, same as real**, so mock and real look the same on screen.
- Walk real mode against the Redo store? **Yes.**

## Assumptions made

- **Seed at startup in real mode**, not a separate `seed` binary (RS-01: one binary). Both calls are upserts, and the gate needs the sku → `prd_` map anyway.
- **Real mode falls back to mock** (with a `warn!`) if config is missing or the seed fails, so the app stays runnable. The `redo_mode` field in the evidence shows which one answered.
- Derived Redo ids: sku `CVH-` + product id without `p_` (`CVH-CANDLES`), external id `cvh-candles`, order id `cvh-o-1`, line id `cvh-o-1-1`. Names split on the first space. Address `100 N Main St` in the customer's seeded city.
- No status mapping to approved/manual_review/denied: the contract already carries Redo's raw status (phase 01 fix).
- `~/redo_smoke.sh` stays outside the repo (open decision 5 not answered).

## Done when

- [x] This phase's step of the demo path works in the running app
- [x] Report written: what you clicked, what you saw
- [ ] Waiting for me to say commit

## Status (handoff)

Built and walked (not committed). `cargo clippy` and `cargo fmt` clean; 3 tests pass (2 contract + 1 fixture parser).

- `AppState.redo` holds the client. `gate::run` calls `state.redo.import_return(&state.seed, &req)`; on `Err` it logs the `RedoError` and answers 502, as before.
- Real mode: `set -a; . ../.env; set +a; REDO_MODE=real cargo run`. Startup seeds the Redo store (about 3.5 s) before it listens.
- Every real pass makes 3 Redo calls: import, status, label. The label call always gets a 500 today (known bug), logged as a warn.

## Demo-path risk

- Real mode was only walked with reason `damaged`. `changed_mind` / `wrong_item` / `forgot_cancel` haven't been sent to Redo yet.
- If Redo is down at startup, the app silently runs the mock (one warn line). Check the log line `Redo: real, store seeded…` before going on stage.
