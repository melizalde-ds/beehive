# General Plan

**Assumptions:** No rubric file exists, so the brief's three Targets stand in for it (Commerce, Technical Build, Overall). The personal agent script is TypeScript, run with Node, so no Python is needed. The stack is Vite + Rust. A Vite build is static files only, so the PAP front door lives in the Rust (Axum) service. Vite holds the dashboard and the consent screen.

**Demo path (one sentence):** The personal agent script discovers Cache Valley Honey Co. through `poppy.json`, gets a write session capped at $50, and files a $36 broken-candle return. The Rust gate passes it, Redo approves it and issues a label, and the shop dashboard updates live. The same agent is then blocked on the $120 crate (over the cap) and on a replay of the first request. Verify proof shows green, then breaks at the edited link after a one-byte tamper.

**Wow moment:** `brief/brief.md`, Demo script steps 3 and 4: the $120 over-cap block, the replay block, and verify + tamper.

**Never cut:** the $120 block, the replay block, verify + tamper.

## Phases

| # | Phase | Blocking? | Parallel with | Target window | Rubric line | Plan file | Status |
|---|-------|-----------|---------------|---------------|-------------|-----------|--------|
| 01 | Contract + seed data: lock the gate, evidence and verify shapes, seed the shop, products, 3 customers and orders, set up empty runnable Vite and Axum apps, ask the Redo table for sandbox keys | Yes | | 6:15-6:45 | All | `phase-01-contract.md` | |
| 02 | Rust gate: every agent check, the hash chain, `verify`, unit tests per check | No | 03, 04, 05 | 6:45-7:45 | Technical | `phase-02-gate.md` | |
| 03 | Redo mock behind the `RedoClient` trait, matching `docs/redo/openapi.yaml`: custom order sync, return create (with `Idempotency-Key`), Redo's return status, label URL | No | 02, 04, 05 | 6:45-7:30 | Commerce | `phase-03-redo-mock.md` | |
| 04 | PAP front door (Rust) + agent script: `poppy.json`, session, returns endpoint, the terminal agent. Separate module from 02 and 03 so the parallel work does not collide | No | 02, 03, 05 | 6:45-8:00 | Commerce | `phase-04-front-door.md` | |
| 05 | Shop dashboard + consent screen (Vite): agent-filed returns, block reasons, Verify proof button, the $50-cap approval step (built against seed examples first) | No | 02, 03, 04 | 6:45-8:15 | Overall | `phase-05-dashboard.md` | |
| 06 | Wire end to end: agent script → front door → gate → Redo mock → dashboard, live. **Checkpoint 8:30** | Yes (needs 02-05) | | 8:15-9:00 | All | `phase-06-wire.md` | |
| 07 | Wow moment hardened: $120 block, replay block, verify + tamper. Seeded, working cold, repeatable reset, readable on a phone | Yes | | 9:00-10:00 | Technical, Overall | `phase-07-wow.md` | |
| 08 | Last mile: README (identity, live link, demo steps, cites PAP, the Redo API, libraries, starter kit, links `docs/findings.md`), Utah styling polish, live URL (ask first), backup recording, audit, personas, rehearse 3x, submission fields. **Freeze 11:25** | Yes | | 10:00-11:25 | Overall | `phase-08-last-mile.md` | |

Optional, only if Redo gave keys **and** 07 is green: swap in the real Redo sandbox behind `RedoClient` (10:00-10:30, taken from 08).

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
1. **Real Redo sandbox.** The mock matches the real API shapes behind `RedoClient`. It goes in only if keys arrive and 07 is green.
2. **Rate limit.** Built in 02, but it is the first check to drop if 02 runs past 7:45. It is not on the demo path.
3. **Consent screen UI.** Fallback: a pre-approved $50-cap session, shown on screen.

Not in the brief, so cut (no rubric line, not on the demo path):
- **An LLM-driven personal agent.** The brief says a script plays the agent. A model would only add risk to the wow moment.
- **Dashboard login.** One seeded shop, no auth.
- **Multiple shops / Utah's Own onboarding.** That goes on the "Next" slide only.
- **More PAP channels.** That goes on the "Next" slide only.
- **Tests beyond the gate.** Unit tests per gate check only (the brief asks for them). Everything else is checked by walking the demo.
