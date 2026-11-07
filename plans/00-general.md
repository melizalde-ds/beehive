# General Plan

**Assumptions:** No rubric file exists, so the brief's three Targets stand in for it (Commerce, Technical Build, Overall). The personal agent script is TypeScript, run with Node, so no Python is needed. The stack is Vite + Rust. A Vite build is static files only, so the PAP front door lives in the Rust (Axum) service. Vite holds the dashboard and the consent screen.

**Demo path (one sentence):** The personal agent script discovers Cache Valley Honey Co. through `poppy.json`, gets a write session capped at $50, and files a $36 broken-candle return. The Rust gate passes it, Redo approves it and issues a label, and the shop dashboard updates live. The same agent is then blocked on the $120 crate (over the cap) and on a replay of the first request. Verify proof shows green, then breaks at the edited link after a one-byte tamper.

**Wow moment:** `brief/brief.md`, Demo script steps 3 and 4: the $120 over-cap block, the replay block, and verify + tamper.

**Never cut:** the $120 block, the replay block, verify + tamper.

## Phases

| # | Phase | Blocking? | Parallel with | Target window | Rubric line | Plan file | Status |
|---|-------|-----------|---------------|---------------|-------------|-----------|--------|
| 01 | Contract + seed data: lock the gate, evidence and verify shapes, seed the shop, products, 3 customers and orders, set up empty runnable Vite and Axum apps, ask the Redo table for sandbox keys | Yes | | 6:15-6:45 | All | `phase-01-contract.md` | Done (`cb3ab83`, `a8696e9`) |
| 02 | Rust gate: every agent check, the hash chain, `verify`, tamper, dashboard list, dev session (no unit tests, by choice) | No | 03, 04, 05 | 6:45-7:45 | Technical | `phase-02-gate.md` | Built + curl-walked, not committed |
| 03 | Redo client, built to the behavior in `docs/redo-integration.md`: a mock shaped like `redo-fixtures/` and a real HTTP client, switched by `REDO_MODE`. A seed command (product upload, then order sync). Return import with `idempotencyKey` = `request_id`, then a status read, then a best-effort label | No | 02, 04, 05 | 6:45-7:30 | Commerce | `phase-03-redo.md` | |
| 04 | PAP front door (Rust) + agent script: `poppy.json`, session, returns endpoint, the terminal agent. Separate module from 02 and 03 so the parallel work does not collide | No | 02, 03, 05 | 6:45-8:00 | Commerce | `phase-04-front-door.md` | |
| 05 | Shop dashboard + consent screen (Vite): agent-filed returns, block reasons, Verify proof button, the $50-cap approval step (built against seed examples first) | No | 02, 03, 04 | 6:45-8:15 | Overall | `phase-05-dashboard.md` | |
| 06 | Wire end to end: agent script → front door → gate → Redo mock → dashboard, live. **Checkpoint 8:30** | Yes (needs 02-05) | | 8:15-9:00 | All | `phase-06-wire.md` | |
| 07 | Wow moment hardened: $120 block, replay block, verify + tamper. Seeded, working cold, repeatable reset, readable on a phone | Yes | | 9:00-10:00 | Technical, Overall | `phase-07-wow.md` | |
| 08 | Last mile: README (identity, live link, demo steps, cites PAP, the Redo API, libraries, starter kit, links `docs/findings.md`), Utah styling polish, live URL (ask first), backup recording, audit, personas, rehearse 3x, submission fields. **Freeze 11:25** | Yes | | 10:00-11:25 | Overall | `phase-08-last-mile.md` | |
| S1 | Stretch: validate discovery with a third-party PAP client (datalayer SDK), then try real Meta Muse. Timeboxed, **hard stop 10:30** | No (after 06) | 07 | 9:30-10:30 | Commerce, Overall | `stretch-muse.md` | Planned, open decisions 8-12 first |

Real Redo keys work and the call order is tested (`docs/redo-integration.md`). Phase 03 builds both modes. Which mode the live demo runs is still open.

## Requirements

<!-- Copied verbatim from brief §6. -->

## MVP: exactly 3 pieces
1. **PAP front door** (Rust, Axum)
   - `/.well-known/poppy.json`, `POST /poppy/session` (guest or signed-in, scopes `poppy:read` / `poppy:write`, short expiry, unique token id)
   - `POST /poppy/returns` for agents to file a return or claim
   - A script acts as the personal agent: discover → session → file a return
2. **Rust gate** (the core)
   - Agent checks: valid session, agent identity, customer consent (scope + refund cap), expiry, replay (token id), idempotency (request id), rate limit per agent
   - On pass: hands off to Redo through a `RedoClient` trait (mock now, real sandbox if keys)
   - SHA-256 hash chain of each step: discovery, session, consent, request, gate decision, Redo result. `verify` reports the first broken link.
3. **Shop dashboard** (Vite + React)
   - Agent-filed returns: which agent, on whose behalf, what was allowed, Redo's result, gate blocks with reasons, a "Verify proof" button

Not in MVP: human chat (Redo has it), our own return decisions (Redo does them), checkout, Stripe, DPoP, real OAuth, photo grading.

## Cut on purpose

Out of scope in the brief:
- **Human chat.** Redo already has it.
- **Our own return decisions.** Redo's review and fraud agents make the call. This also keeps any model out of the decision path.
- **Checkout, Stripe.** Not on the demo path.
- **DPoP.** Short expiry + unique token id cover replay for the demo. The Q&A answer is "DPoP is next."
- **Real OAuth.** The session endpoint issues its own short-lived token. Guest and signed-in are simulated.
- **Photo grading.** Redo's job.

Cut by default, from the brief's cut list (in order):
1. ~~Real Redo sandbox~~. No longer cut: the keys work and phase 03 builds real mode. The mock stays as the fallback.
2. **Rate limit.** Built in 02, but it is the first check to drop if 02 runs past 7:45. It is not on the demo path.
3. **Consent screen UI.** Fallback: a pre-approved $50-cap session, shown on screen.

Cut after testing real Redo (`docs/redo-integration.md`):
- **Risk score (LOW/MEDIUM/HIGH).** Redo has none, and a mock-only score would look like a Redo feature. We show Redo's own return status instead (`open` = approved).
- **Return label on the demo path.** Reading an imported return gives a 500, so there's no label to show. `label_url` stays in the contract, filled on a best-effort basis.
- **One evidence link per gate check or per Redo call.** One `gate_decision` link records every check's result and one `redo_result` link records the Redo calls, so the chain stays readable on screen. A block chain ends at `gate_decision` with no Redo link.
- **`async-trait`.** The Redo client is an enum (mock or real) instead of a trait object, so one less dependency.

Not in the brief, so cut (no rubric line, not on the demo path):
- **An LLM-driven personal agent.** The brief says a script plays the agent. A model would only add risk to the wow moment.
- **Dashboard login.** One seeded shop, no auth.
- **Multiple shops / Utah's Own onboarding.** That goes on the "Next" slide only.
- **More PAP channels.** That goes on the "Next" slide only.
- **Tests beyond the gate.** Unit tests per gate check only (the brief asks for them). Everything else is checked by walking the demo.

## Open decisions (ask before the phase that needs them)

1. **Demo mode** (phase 07/08): run the live demo against real Redo with `REDO_MODE=mock` as the fallback (recommended), or mock only.
2. **Brief demo line** (editing the brief is ask-first): change "Redo approves it (LOW risk), and a label is issued" to "Redo opens the return (approved)".
3. **Seed fields for real Redo** (phase 03, schema change): product sku + external id, customer first/last name + shipping address, order line ids without underscores. Recommended: add them to the seed schema.
4. **`reqwest`** (phase 03, new crate) for the real Redo client.
5. **`~/redo_smoke.sh`**: copy into the repo (for example `scripts/redo_smoke.sh`) after checking it has no hardcoded secret?
6. **`dotenvy`** (new crate) so `cargo run` reads `.env` (rule RS-09), needed once real mode uses `REDO_API_SECRET`.
7. **`*:Zone.Identifier`** in `.gitignore` (Windows download leftovers in `brief/` and `docs/`).
8. **Discovery shape for S1** (schema change, ask first): the SDK targets PAP draft 0.3 (`protocol_version`, `organization`, `agent.protocols`, `auth.issuer`). Our `PoppyDiscovery` is our own `0.1-draft` shape (`pap_version`, `merchant`, `endpoints`), so the SDK check fails until the schema changes. Phase 04 is building against the current shape.
9. **Deploy target** (ask first): S1 assumes AWS (Lightsail/EC2 + Caddy TLS) by 9:30. That moves the live URL out of phase 08 and overlaps phase 07 (wow hardening).
10. **Python for S1**: the general plan says no Python. The SDK script is the one exception, outside the demo path.
11. **Muse's `agent_id`** (seed change, ask first): the gate only knows `muse-demo`. Real Muse sends its own id and would be blocked by `agent_identity` until it's in the seed.
12. **OAuth issuer metadata**: S1 needs a static `/.well-known/oauth-authorization-server`. The brief cuts real OAuth, so if Muse follows it to sign in, S1 stops (see `stretch-muse.md`).
