# Phase 08: Last mile

**Goal:** A live URL a judge can touch, a wow moment that survives a shared URL and a Redo hiccup, then README, polish, rehearsal and submission. **Freeze 11:25.**
**Brief requirements covered:** Targets (live, clear in 3 min), Rules check (README cites PAP, Redo API, libraries, starter kit), "Never cut: the $120 block, the replay block, verify + tamper."
**Depends on:** 07 (done)
**Touches only:** deploy config, `server/src/` (`demo.rs`, `gate.rs`), `agent/agent.ts`, `web/src/components/`, `web/public/`, `README.md`

## Wow check (2026-11-06, before deploy)

Run with `/wow` on a cold mock instance, fresh Chromium profile, 375×667. Full report in that session.

| Check | Result |
|---|---|
| Cold start, fresh browser | Pass: seed shows $36 pass, $120 cap block, duplicate block at once |
| Under 90 s | Pass: scripted run 2.9 s; a human run should be about 40-60 s |
| Phone 375 px | Partial: "Passed" card below the fold; "Proof is broken" scrolls off after tamper |
| Failure modes | Partial: Redo error mid-run kills the run; rerun without `--reset` is messy |
| Live URL | Fail: none yet |
| Obvious without narration | Partial: newest-first order reads backwards; raw JSON in the proof |

## Checklist

### 1. Deploy (in progress)

- [ ] Live URL for the API and the dashboard
- [ ] Set `BEEHIVE_WEB_URL` to the live dashboard, or the agent's consent link points at localhost
- [ ] Build the dashboard with `VITE_API_URL` set to the live API
- [ ] Rerun `/wow` against the live URL, on the production build (only the dev server was tested)

### 2. Harden (ranked by risk to the demo)

- [ ] **Shared state on the live URL** (about 30-45 min). "Reset demo" and "Edit one byte" are open to anyone. One click empties the wow for every viewer until a restart (`server/src/demo.rs:14`); a tamper leaves the proof broken for the next viewer. See question 1.
- [ ] **Redo error mid-run** (about 20 min). If real Redo errors on the $36, the gate answers 502 (`server/src/gate.rs:130`), the agent's `call()` throws, and the $120 and replay blocks never run. Startup failure is already covered (mock fallback). Found by reading the code, not forced live. See question 2.
- [ ] **Rehearse with `--reset` only** (no code). Without it the $36 blocks as "Order o_1 already has a return", then the rate limit blocks everything.

### 3. Clean

- [ ] Redo store: 3 "Ana Test" smoke returns judges would see (general plan, decision 15)
- [ ] Decide the $0.00 store-credit line for the pitch (decision 15)

### 4. Beautify (phone first, 375 px)

- [ ] **Card order** (about 15 min). Newest first puts "Blocked: Duplicate" on top and the "Passed" $36 below the fold (`web/src/components/Dashboard.tsx`). See question 3.
- [ ] **Proof panel** (about 20 min). Show the gate checks as ✓ rows, not raw JSON, and scroll "Proof is broken" into view after the tamper (`web/src/components/ProofPanel.tsx`).
- [ ] **Jargon** (about 10 min). The card says `poppy:read, poppy:write`; use the consent screen's words ("See your orders", "File returns") (`web/src/components/ReturnCard.tsx`).
- [ ] **Brand picks** (about 20 min). Favicon, header mark, monospace for ids and hashes. List in `00-general.md`, "Next up" item 4.

### 5. Last mile

- [ ] README: one-line identity, live link, demo steps, cites PAP, the Redo API, libraries, starter kit, links `docs/findings.md`; tagline and Hash-Chain Comb
- [ ] Backup recording of the full demo path
- [ ] `/audit` in a fresh session, then `/persona` (architect, UX lead, Redo)
- [ ] Rehearse 3 times, each with `--reset`
- [ ] Fill every submission field. **Freeze 11:25**

## Questions to ask (at phase start, recommended first)

1. **Shared state on the live URL:**
   (a) Dashboard "Reset demo" re-seeds the cold-open state; agent `--reset` keeps emptying the shop, so its own $36 still passes. Any viewer can restore a tampered or emptied demo. *(recommended)*
   (b) Hide "Reset demo" and "Edit one byte" unless the URL has `?demo=1`.
   (c) Both.
2. **Redo error mid-run:**
   (a) The agent logs the error and keeps going, so the $120 and replay blocks still show. *(recommended, smallest)*
   (b) The gate falls back to the mock for that one return.
3. **Card order:**
   (a) Oldest first, so the story reads pass → cap block → duplicate. *(recommended)*
   (b) Keep newest first, pin the $120 block to the top.

## Done when

- [ ] `/wow` passes all 6 checks on the live URL
- [ ] Report written: what you clicked, what you saw
- [ ] Waiting for me to say commit
