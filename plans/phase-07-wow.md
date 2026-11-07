# Phase 07: Wow hardened

**Goal:** The wow moment ($120 cap block, replay block, verify + tamper) can be rehearsed again and again without a restart, and shows on a cold open of the dashboard.
**Brief requirements covered:** Demo script steps 3 and 4. "Never cut: the $120 block, the replay block, verify + tamper."
**Depends on:** 02-06 (done)
**Touches only:** `server/src/` (new `demo.rs`, a route in `main.rs`), `web/src/` (reset button), `agent/agent.ts` (`--reset`), `brief/brief.md` (one line, approved), `AGENTS.md` (commands)

## Checklist

- [x] `POST /demo/reset`: empty the in-memory store (returns, sessions, evidence, replay and rate-limit memory), tell the dashboard, answer with the empty `ReturnList` (no schema change)
- [x] `BEEHIVE_DEMO_SEED=1`: at startup, file the $36 pass, the $120 block and the replay through the real gate, with the demo session (Redo mock or real)
- [x] Agent script: `--reset` clears the shop before the run
- [x] Dashboard: small "Reset demo" button
- [x] Brief: "Redo approves it (LOW risk), and a label is issued" → "Redo opens the return (approved)"
- [x] AGENTS.md: the new flag and env var
- [x] Walk: seed cold, tamper + verify, reset from the dashboard, agent `--reset` twice in a row, 375 px

## Questions asked

- Reset trigger? → Both: agent `--reset` and a dashboard button.
- Pre-run the demo at startup? → Yes, opt-in (`BEEHIVE_DEMO_SEED=1`).
- Unit tests per gate check (decision 13)? → Skip.
- Demo mode (decision 1) and brief line (decision 2)? → Real Redo with mock as the fallback, update the brief.

## Assumptions made

- Reset always leaves the shop empty and doesn't re-seed. A re-seed would make the next agent run block its own $36 as "already returned". The seed runs at startup only.
- Reset is open to anyone, like the tamper route. It's a demo; anyone on the live URL can clear it, and a restart or an agent run fills it again.
- In real mode the seed files a real Redo return on every startup (a new id each time).

## Done when

- [x] This phase's step of the demo path works in the running app
- [x] Report written: what you clicked, what you saw
- [ ] Waiting for me to say commit
