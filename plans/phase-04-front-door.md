# Phase 04: PAP front door + agent script

**Goal:** A personal agent finds the shop through `poppy.json`, asks for a session, waits for the customer's consent, and files returns through `/poppy/returns`, which runs the phase 02 gate.
**Brief requirements covered:** MVP piece 1 (`/.well-known/poppy.json`, `POST /poppy/session` guest or signed-in with scopes, short expiry and unique token id, `POST /poppy/returns`, the agent script), demo script steps 1-3 from the terminal.
**Depends on:** phase 01 contract, phase 02 gate (`gate::run` and `gate::file_return`, `Session`, `Keys`)
**Touches only:** `server/src/poppy.rs` (new), `server/src/main.rs` (`mod` line + routes), `server/src/session.rs` (one field), `agent/` (new), `plans/phase-04-front-door.md`, `.env.example`

## Checklist

- [x] `GET /.well-known/poppy.json`: `PoppyDiscovery` built from the seed (`0.1-draft`, platform Redo, 900 s max session)
- [x] `POST /poppy/session`: find the customer (by `customer_id` when signed in, by email for a guest), store a pending session with its discovery + session events, reply with `consent_url`
- [x] `GET /poppy/session/:id`: the agent polls it. Token fields stay null until the customer approves
- [x] `POST /consent/:id`: approve (the customer may lower the cap or drop scopes, never raise them) → token with a 900 s expiry and a fresh `jti`, plus the consent event. Deny → `denied`. A second decision → 409
- [x] `POST /poppy/returns`: same handler as `/gate/returns`
- [x] `agent/agent.ts` (Node 24 runs it directly, no deps): discover → session → wait for consent → $36 candle return → $120 crate → replay the first request
- [x] `.env.example`: `BEEHIVE_WEB_URL` (consent link base), `BEEHIVE_URL` (agent → service)
- [x] `cargo build`, `clippy`, `fmt` clean, contract tests pass
- [x] Walk: run the agent, approve with curl (consent screen is phase 05), see pass / cap block / replay block

## Assumptions made

- **Worktree.** Phase 03 is running in the main checkout, so this phase lives in `../crimson-hacks-project-phase-04` on `phase-04-front-door`, branched from `1ca93cc`.
- **Discovery shape stays `0.1-draft`** (open decision 8 is still open; changing it is a schema change).
- **No `dotenvy`.** You said config comes from the shell env.
- **Session creation doesn't check the agent.** An unknown agent still gets a session, and the gate blocks it at `agent_identity` with evidence, so the block shows on the dashboard.
- **Unknown customer → 400.** A signed-in `customer_id` whose email doesn't match → 400.
- **The session id is the polling secret.** Anyone holding `ses_…` can read the token from `GET /poppy/session/:id`. Fine for the demo; real PAP uses OAuth here (cut).
- **Expiry is 900 s** (the `max_session_seconds` in discovery). The dev session from `BEEHIVE_DEMO_SESSION=1` keeps its 12 h.
- **No pauses in the agent script.** It runs the three requests straight through. Phase 07 can add an Enter-to-continue if the pitch needs it.

## Questions asked

- Which phase? **04**, in parallel with 03 (another agent).
- `dotenvy`? **No.** Vars come from the env.

## Done when

- [x] This phase's step of the demo path works in the running app
- [x] Report written: what you clicked, what you saw
- [x] Waiting for me to say commit
