# Stretch S2: hardening

**Goal:** The live demo survives a Redo hiccup, a curious judge and a restart.
**Depends on:** phase 08 deploy (live URL up). Ranked by risk to the demo; stop wherever the clock says.
**Never cut for this:** the $120 block, the replay block, verify + tamper.
**Touches only:** `server/src/` (`redo.rs`, `gate.rs`, `demo.rs`), `agent/agent.ts`, `web/src/components/`

## Items (ranked)

1. **Agent keeps going after an error** (about 15 min). Today, if real Redo errors on the $36, the gate answers 502 (`server/src/gate.rs:130`), the agent's `call()` throws, and the $120 and replay blocks never run. Fix: `show()` the error and move on. The two blocks don't touch Redo, so they still show. Found by reading the code, not forced live.
2. **Retry transient Redo errors** (about 20 min). One retry with a short backoff on timeouts and 5xx in the real client (`server/src/redo.rs`). The `reqwest` client already has a 10 s timeout. Safe because the return import sends `idempotencyKey` = `request_id`, so a retry can't file twice. Check that against real Redo before relying on it (log in `docs/findings.md` either way). No 4xx retries.
3. **Shared state on the live URL** (about 30-45 min). "Reset demo" and "Edit one byte" are open to anyone. One click empties the wow for every viewer until a restart (`server/src/demo.rs:14`), and a tamper leaves the proof broken for the next viewer. See question 1.
4. **Fall back to the mock for one return** (about 20 min, only if 1 and 2 aren't enough). When real Redo still fails after the retry, file that one return with the mock and mark it "Redo offline, mock" on the card. It must never pass silently as a real Redo return. See question 2.
5. **Restarts don't spam Redo** (about 15 min). In real mode every restart and every "Reset demo" files the cold-open $36 into the real store. Option: cold-open seed uses mock results even in real mode, labeled as seeded. See question 3.
6. **Muse-friendly agent split** (about 10 min, S1 only). If Muse files the $36, the script's own $36 blocks as "order already has a return", and its replay replays a blocked request. Add `--skip-candle` so the script only does the $120 block, and replays it.

## Questions to ask (at stretch start, recommended first)

1. **Shared state:**
   (a) Dashboard "Reset demo" re-seeds the cold-open state; agent `--reset` keeps emptying the shop, so its own $36 still passes. Any viewer can restore a tampered or emptied demo. *(recommended)*
   (b) Hide "Reset demo" and "Edit one byte" unless the URL has `?demo=1`.
   (c) Both.
2. **Redo still down after a retry:**
   (a) Show the error on the card and keep going (items 1 + 2 only). *(recommended, smallest)*
   (b) Also fall back to the mock for that return (item 4).
3. **Cold-open seed in real mode:**
   (a) Keep filing into real Redo; clean the store after judging. *(recommended, no code)*
   (b) Seed with mock results, labeled.

## Done when

- [ ] Forced Redo failure (bad `REDO_API_SECRET` after startup, or `REDO_BASE` pointed at a dead port) still shows the $120 and replay blocks
- [ ] A judge clicking "Reset demo" or "Edit one byte" can't leave the next viewer an empty or broken demo
- [ ] Any Redo retry or idempotency surprise logged in `docs/findings.md`
