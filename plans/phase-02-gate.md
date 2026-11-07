# Phase 02: Rust gate

**Goal:** The gate decides whether an agent may ask for a return, records every step in a SHA-256 hash chain, and can prove the chain is intact.
**Brief requirements covered:** MVP piece 2 (agent checks, `RedoClient` handoff, hash chain + `verify`), and the endpoints `POST /gate/returns`, `GET /evidence/:return_id`, `GET /verify/:return_id`, `POST /evidence/:return_id/tamper`, `GET /returns`.
**Depends on:** phase 01 contract (`schema/beehive.schema.json`)
**Touches only:** `server/`, `plans/phase-02-gate.md`, `docs/findings.md`, `.env.example`

## Checklist

- [x] Session tokens: issue + verify (phase 04's `POST /poppy/session` calls issue)
- [x] Gate checks, in order: rate limit per agent, valid session, agent identity, expiry, replay (token id bound to agent + session), scope, idempotency (request id), order (belongs to the customer, amount within the order, not already returned), refund cap (in cents)
- [x] Redo handoff through phase 03's client (an enum: mock or real), passing `request_id` as Redo's `idempotencyKey`. Until 03 lands, a stand-in that answers like `redo-fixtures/return-import.json` + `return-status.json`
- [x] Hash chain per attempt: discovery, session, consent, request, gate decision (every check's result in its data), Redo result. A blocked attempt ends at the gate decision with no Redo link. `verify` names the first broken link
- [x] Endpoints: `/gate/returns`, `/evidence/:id`, `/verify/:id`, `/evidence/:id/tamper`, `/returns`. JSON errors on every failure
- [x] CORS via tower-http
- [x] Dev-only seeded session: `BEEHIVE_DEMO_SESSION=1` starts the service with Kaylee's approved $50 session for `muse-demo` and logs its token
- [x] Walk with curl: $36 passes, $120 is blocked by the cap, a replay of the first request is blocked, verify is green, tamper turns it red

## Status (handoff)

Built and walked (not committed). New files: `server/src/{error,session,gate,evidence,redo}.rs`; `main.rs` now holds `AppState`, the in-memory `Store`, the routes and CORS. `cargo build`, `cargo clippy`, `cargo fmt` clean; the 2 phase-01 contract tests pass.

For phase 03: replace the body of `redo::import_return(&GateRequest)`; `gate::run` calls it with no lock held and undoes the reservations on `Err`.
For phase 04: `gate::run(&state, req)` is the shared entry point for `/poppy/returns`. Sessions: `Session::pending(..)`, push the discovery and session `Event`s, `session.activate(keys, scopes, cap_cents, ttl)` returns the token, push the consent `Event`, insert into `store.sessions`. See `session::demo_session` for the full sequence.

## Assumptions made

- Only a **passed** request reserves its `request_id`. Re-sending a blocked request is checked again (and blocked again) rather than called a duplicate, so the idempotency reason "already filed as bh_0001" is always true.
- Return ids (`bh_0001`…) are given to every attempt, pass or block, so a block has its own evidence and verify.
- `tamper` returns the changed `Evidence`, so the dashboard can redraw without a second call.
- Request link data has the four fields in the plan (no `agent_id`; it's in the discovery link and the record).

## Demo-path risk

- Rate limit is 5 per minute per agent. The demo uses 3 requests, but two rehearsals within a minute without a restart will hit it. Restart the service between runs (it's in memory anyway), or phase 07 can add a reset.

## Design (agreed, build to this)

**Files in `server/src/`:** `main.rs` (config, state, router, CORS), `error.rs` (an error type that renders `ApiError` JSON), `session.rs` (claims, issue/verify, session table, dev session), `gate.rs` (checks + `/gate/returns` + `/returns`), `evidence.rs` (chain, verify, tamper + their routes), `redo.rs` (stand-in, replaced by phase 03). `types.rs` stays generated.

**State:** `AppState { seed: Arc<Seed>, keys: Arc<Keys>, store: Arc<std::sync::Mutex<Store>> }`. `Store` holds sessions by id, attempts (record + evidence links) newest first, `request_id → return_id`, `order_id → return_id` for passed returns, per-agent request times for the rate limit, and a counter for `bh_0001`-style ids. Never hold the lock across the Redo `await`: reserve `request_id` and `order_id` under the lock, release it, call Redo, then re-lock to record. If Redo fails, remove both reservations so the agent can retry.

**Session JWT (HS256):** claims `sid` (session id), `jti` (token id, `tok_…`), `sub` (customer id), `agent_id`, `scopes`, `cap_cents`, `iat`, `exp`. Secret from `BEEHIVE_JWT_SECRET`, or a fixed dev secret plus a `tracing::warn!`. Decode with `validate_exp = false`, so expiry is its own check with its own reason. A session record keeps its status, current `token_id`, scopes, cap, expiry, and its first three evidence events (discovery, session, consent). Phase 04 creates sessions through this module.

**Checks, in order.** The first failure blocks. `blocked_by` uses the `GateCheck` values:
1. `rate_limit`: more than 5 requests from this `agent_id` in the last 60 s. Only requests that get past this check count.
2. `session`: bad signature, or the `sid` is unknown.
3. `agent_identity`: `agent_id` isn't in the seed's `agents`.
4. `expiry`: `exp` has passed.
5. `replay`: the token's `agent_id` ≠ the request's, the `jti` ≠ the session's current token, or the session isn't `active`.
6. `scope`: no `poppy:write`.
7. `idempotency`: `request_id` already seen. Reason: "Duplicate of a request already filed as bh_0001. No second refund."
8. `order`: unknown order, not this customer's order, amount over the order total, or the order already has a passed return.
9. `refund_cap`: `round(amount × 100)` > `cap_cents`. Reason: "$120.00 is over the $50 cap the customer approved".
Pass reason: "Within $50 cap approved by customer".

**HTTP:** a gate block is still a 200 with a `GateResponse` body, because the contract says every non-2xx body is an `ApiError`. A malformed body gets a 400 `ApiError` (catch `JsonRejection`). An unknown `return_id` gets a 404 `ApiError`.

**Evidence chain per attempt:** the session's discovery/session/consent events, then `request` (request_id, order_id, reason, amount), then `gate_decision` (gate, reason, `checks: [{check, ok}]` up to the failing one), then `redo_result` (return_id, status, label_url) only on a pass. A block before the session is known gets just `request` + `gate_decision`. `hash = hex(sha256(prev_hash | index | step | at | data_json))`, where `prev_hash` is 64 zeros for link 0 and `data_json = serde_json::to_string(&data)` (serde_json has no `preserve_order` here, so keys are sorted and stable). `verify` walks the links and reports the first link whose `prev_hash` or recomputed hash doesn't match.

**Tamper (demo only):** in the chosen link's `data` JSON, add 1 to the first digit (9 becomes 0); if there's no digit, change the first letter. Store it without rehashing. On the request link, the amount 36 becomes 46, so it's visible on screen.

**Redo stand-in (`redo.rs`):** `import_return(request_id, …) -> Result<RedoResult, String>`. It returns `return_id` = the first 24 hex chars of sha256(request_id), so the same key gives the same id like real Redo, plus `status: Open` and `label_url: None`. Phase 03 replaces it with the mock/real enum from `docs/redo-integration.md`.

**`GET /returns`:** all attempts newest first, as `ReturnRecord`. Customer and product names come from the seed. If no session was known, the customer is shown as "Unknown", with empty scopes and a cap of 0.

**CORS:** `tower_http::cors::CorsLayer::permissive()`. There are no cookies or credentials on this API.

**Dev session:** with `BEEHIVE_DEMO_SESSION=1`, startup creates an active session for `c_1` / `muse-demo` (scopes read+write, $50 cap, expires in 12 h) with the three events, and logs the session id and token at `info`.

**Walk:**
```bash
cd server && BEEHIVE_DEMO_SESSION=1 cargo run      # copy the token from the log
T=<token>
curl -s localhost:3000/gate/returns -H 'content-type: application/json' -d '{"request_id":"7c1e2a90-4b6f-4d1e-9a3b-2f8d5c0e6a11","session_token":"'$T'","agent_id":"muse-demo","order_id":"o_1","reason":"damaged","amount":36}'          # pass
curl -s localhost:3000/gate/returns -H 'content-type: application/json' -d '{"request_id":"b2d4f6a8-1c3e-4a5b-8d7f-9e0a1b2c3d4e","session_token":"'$T'","agent_id":"muse-demo","order_id":"o_2","reason":"changed_mind","amount":120}'  # block: refund_cap
# re-send the first request: block: idempotency
curl -s localhost:3000/verify/bh_0001                                                        # ok: true
curl -s localhost:3000/evidence/bh_0001/tamper -H 'content-type: application/json' -d '{"link_index":3}'
curl -s localhost:3000/verify/bh_0001                                                        # broken_at: 3, request
curl -s localhost:3000/returns
```
Generated type names come from typify (for example `GateCheck::RefundCap`, `EvidenceStep::GateDecision`, `Scope::PoppyWrite`, `RedoStatus::Open`). Constrained strings are newtypes, so use `.as_str()` or deref.

## Questions asked

- New crates? **Approved:** jsonwebtoken, sha2, hex, chrono, uuid, tower-http.
- Duplicates and replay? **Request id + token binding:** the same `request_id` again is blocked by idempotency, pointing to the original. A token id is bound to its agent and session, so it's blocked as replay if another agent uses it or its session has ended. An order that already passed can't pass again.
- Storage? **In memory.** A restart gives a clean demo.
- Rate limit? **5 per minute per agent.**
- JWT secret? **`BEEHIVE_JWT_SECRET` from `.env`.** If it's missing, use a fixed dev secret and log a warning.
- `redo-fixtures/`? **Phase 03's mock replays them.** Commit them once you've checked they hold no secrets or real customer data.
- Session for the curl walk? **A dev-only seeded session behind `BEEHIVE_DEMO_SESSION=1`.**
- Unit tests? **Skipped.** The two contract tests from phase 01 stay; the curl walk is the check.

## Done when

- [x] This phase's step of the demo path works in the running app
- [x] Report written: what you clicked, what you saw
- [ ] Waiting for me to say commit
