# General Plan

**Assumptions:** No rubric file exists, so the brief's three Targets stand in for it (Commerce, Technical Build, Overall). The personal agent script is TypeScript, run with Node, so no Python is needed. The stack is Vite + Rust. A Vite build is static files only, so the PAP front door lives in the Rust (Axum) service. Vite holds the dashboard and the consent screen.

**Demo path (one sentence):** The personal agent script discovers Cache Valley Honey Co. through `poppy.json`, gets a write session capped at $50, and files a $36 broken-candle return. The Rust gate passes it, Redo approves it and issues a label, and the shop dashboard updates live. The same agent is then blocked on the $120 crate (over the cap) and on a replay of the first request. Verify proof shows green, then breaks at the edited link after a one-byte tamper.

**Wow moment:** `brief/brief.md`, Demo script steps 3 and 4: the $120 over-cap block, the replay block, and verify + tamper.

**Never cut:** the $120 block, the replay block, verify + tamper.

## Phases

| # | Phase | Blocking? | Parallel with | Target window | Rubric line | Plan file | Status |
|---|-------|-----------|---------------|---------------|-------------|-----------|--------|
| 01 | Contract + seed data: lock the gate, evidence and verify shapes, seed the shop, products, 3 customers and orders, set up empty runnable Vite and Axum apps, ask the Redo table for sandbox keys | Yes | | 6:15-6:45 | All | `phase-01-contract.md` | Done (`cb3ab83`, `a8696e9`) |
| 02 | Rust gate: every agent check, the hash chain, `verify`, tamper, dashboard list, dev session (no unit tests, by choice) | No | 03, 04, 05 | 6:45-7:45 | Technical | `phase-02-gate.md` | Done (`1ca93cc`) |
| 03 | Redo client, built to the behavior in `docs/redo-integration.md`: a mock shaped like `redo-fixtures/` and a real HTTP client, switched by `REDO_MODE`. A seed command (product upload, then order sync). Return import with `idempotencyKey` = `request_id`, then a status read, then a best-effort label | No | 02, 04, 05 | 6:45-7:30 | Commerce | `phase-03-redo.md` | Done (`4a48e76`), proof in Redo's `notes` (`05df15d`) |
| 04 | PAP front door (Rust) + agent script: `poppy.json`, session, returns endpoint, the terminal agent. Separate module from 02 and 03 so the parallel work does not collide | No | 02, 03, 05 | 6:45-8:00 | Commerce | `phase-04-front-door.md` | Done (`3f1fad6`) |
| 05 | Shop dashboard + consent screen (Vite): agent-filed returns, block reasons, Verify proof button, the $50-cap approval step (built against seed examples first) | No | 02, 03, 04 | 6:45-8:15 | Overall | `phase-05-dashboard.md` | Done (`04536f0`), incl. the SSE route `/returns/stream` |
| 06 | Wire end to end: agent script → front door → gate → Redo mock → dashboard, live. **Checkpoint 8:30** | Yes (needs 02-05) | | 8:15-9:00 | All | `phase-06-wire.md` | Checkpoint met in phase 05's walk (mock, at 375 px). Real mode: the return shows in Redo's dashboard (2026-11-06), but as $0.00 store credit (see decision 15). Walked in real mode at 375 px on 2026-11-06 20:20 (agent script + Playwright consent): $36 passed, Redo return `6aee99198a41ad54330e8cfe` open; $120 and the replay blocked; `verify` 6/6. Done: the proof note in Redo's Activity feed carries the same gate hash (`7752d692…`) |
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
3. ~~Seed fields for real Redo~~. Answered in phase 03: derived in code, no schema change.
4. ~~`reqwest`~~. Answered: added in phase 03.
5. **`~/redo_smoke.sh`**: copy into the repo (for example `scripts/redo_smoke.sh`) after checking it has no hardcoded secret?
6. ~~`dotenvy`~~. Answered: no. Export the vars in the shell (`set -a; . ../.env; set +a`).
7. ~~`*:Zone.Identifier`~~. Moot: no such files are left.
8. **Discovery shape for S1** (schema change, ask first): the SDK targets PAP draft 0.3 (`protocol_version`, `organization`, `agent.protocols`, `auth.issuer`). Our `PoppyDiscovery` is our own `0.1-draft` shape (`pap_version`, `merchant`, `endpoints`), so the SDK check fails until the schema changes. Phase 04 is building against the current shape.
9. **Deploy target** (ask first): S1 assumes AWS (Lightsail/EC2 + Caddy TLS) by 9:30. That moves the live URL out of phase 08 and overlaps phase 07 (wow hardening).
10. **Python for S1**: the general plan says no Python. The SDK script is the one exception, outside the demo path.
11. **Muse's `agent_id`** (seed change, ask first): the gate only knows `muse-demo`. Real Muse sends its own id and would be blocked by `agent_identity` until it's in the seed.
12. **OAuth issuer metadata**: S1 needs a static `/.well-known/oauth-authorization-server`. The brief cuts real OAuth, so if Muse follows it to sign in, S1 stops (see `stretch-muse.md`).
13. **Unit tests per gate check** (phase 07): the brief asks for them and "Cut on purpose" keeps them, but phase 02 skipped them by choice. Today there are 3 tests (contract + Redo fixture parser), none per check.
14. **Agent name on the consent screen** (schema change, ask first): `SessionResponse` has no `agent_id`, so the screen says "Your AI agent" instead of "Muse".
15. **$0.00 store credit in real Redo**: the return import (`server/src/redo.rs`, `import_return`) sends no price or refund type per product, so Redo shows the $36 candle return as "$0.00, Store credit". Checked: the import has no field for either, nor an order link (logged in `docs/findings.md`). Options: live with it and say so in the pitch, or try sending `shippingAddress` (untested, would only fix "No address"). The Redo store also has 3 "Ana Test" returns from the smoke runs, which judges would see if they open it.

## Next session (handoff, 2026-11-06 evening)

**Where it is:** `main` (`212efc3`) has phases 01-05 and the sponsor docs. The demo path works end to end in mock mode: agent script → consent screen → gate → Redo mock → dashboard over SSE → verify + tamper. Branch `phase-03-redo-proof` (uncommitted) puts Beehive's proof in Redo's `notes`.

**Run it:**
```bash
cd server && cargo run                        # :3000, mock Redo
cd web && pnpm dev                            # :5173
BEEHIVE_URL=http://localhost:3000 node agent/agent.ts   # prints the consent link; approve it in the browser
```
Real Redo: `cd server && set -a; . ../.env; set +a; REDO_MODE=real cargo run` (seeds the store at startup, about 3.5 s).

**Next up, in order:**
1. ~~Commit `phase-03-redo-proof`~~ (`05df15d`). Real-mode walk done, proof note verified in Redo. Left: decision 15 ($0.00 store credit).
2. Phase 07 (wow hardening). The main gap: a rehearsal reset. Everything is in memory, the rate limit is 5 per minute per agent, and a second agent run blocks the $36 at `order` (already returned). Today the only reset is a server restart.
3. Open decisions 1, 2, 13, 14, then 5, 7, 8-12 for the stretch.
4. Phase 08 (README, deploy, polish). Polish note from phase 05: the gate decision link shows its `checks` array as raw JSON.

**Leftovers:** none. Worktrees, merged branches and the stale GitNexus index were cleaned up on 2026-11-06.
