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

## Parallel tracks (from 21:20, freeze 11:25)

The deploy is done (`890fe10`), so four tracks run in separate sessions. Each one runs in its own worktree from `phase-08-last-mile`, on its own branch (`phase-08-<letter>-<name>`). None of them touch the same files. Each session reads the brief, `00-general.md` and this section, confirms its decision with me, builds, walks its piece and stops (no commit until I say). `.gitnexus/` isn't in worktrees: run GitNexus from the main checkout.

| # | Track | Files (only these) | Time | Decision to confirm (recommended) |
|---|---|---|---|---|
| A | Muse front door | `server/src/poppy.rs`, `server/src/main.rs`, `schema/`, `web/src/types/generated/` | ~35 min | D1 |
| B | Agent + reset | `agent/agent.ts`, `server/src/demo.rs` | ~45 min | D4 |
| C | Beautify | `web/src/components/`, `web/public/` | ~45 min | D5 |
| D | README | `README.md` | ~20 min | none |
| E | Deploy + Muse test (main session) | `deploy/`, `scripts/`, `seed/` (Muse's id only), `plans/` | ongoing | D2, D3 |

**Decisions** (recommended answers; each track confirms its own before starting):
- **D1 Discovery shape:** add the PAP draft 0.3 fields (`protocol_version`, `organization`, `agent.protocols`, `auth.issuer`) next to today's fields, don't replace them. The agent script and the dashboard keep working, and track B never has to touch discovery. Serve `/.well-known/oauth-authorization-server` as static JSON only (no real OAuth). Python allowed for `scripts/validate_pap.py` only (decision 10).
- **D2 Order id for Muse:** Kaylee puts it in her prompt ("return order o_1, the broken candle set"). No order-lookup endpoint (that would be about 20 min more).
- **D3 Muse's agent id:** add it to `seed/` once the probe shows it. That costs one redeploy (decision 11).
- **D4 Redo error mid-run:** the agent shows the error and keeps going, with no mock fallback.
- **D5 Card order:** oldest first, so the cards read pass, then cap block, then duplicate.

### Track A: Muse front door

- [ ] Discovery: add the D1 fields. Issuer and domain come from `BEEHIVE_WEB_URL`, which is already the live host (`https://32-189-214-147.sslip.io`). Keep `endpoints` relative: Caddy routes `/poppy/*` and `/.well-known/*` at the root.
- [ ] Advertise what a return needs in discovery: the `GateRequest` fields and the `reason` values. Muse has no other way to learn them.
- [ ] Static `GET /.well-known/oauth-authorization-server`: `issuer` matches exactly, and `poppy_domains` lists the host.
- [ ] Schema change as text in `schema/beehive.schema.json`, then `pnpm gen:types` and `cargo test`.
- [ ] Done when: both `/.well-known` URLs return the new JSON locally, `cargo test` passes, and the old agent script still runs end to end in mock mode.

### Track B: Agent + reset (S2 items 1, 3, 6)

- [ ] On a Redo error (502), the agent shows it and goes on to the $120 and the replay. `call()` must stop throwing on gate errors.
- [ ] `--skip-candle`: file only the $120 and its replay (for when Muse files the $36).
- [ ] `POST /demo/reset` with no query re-seeds the cold-open demo (the dashboard button sends `{}` today, so `web/` stays untouched). `POST /demo/reset?empty=1` empties the shop: `agent.ts --reset` sends that.
- [ ] Done when: a forced Redo error (mock mode, bad `REDO_BASE` in real mode) still shows both blocks; "Reset demo" after a tamper brings back a clean, verified cold open; `--reset` still lets the agent's own $36 pass.

### Track C: Beautify (section 4 below)

- [ ] Card order (D5), proof panel as ✓ rows with "Proof is broken" scrolled into view, consent-screen words instead of `poppy:read`. Brand picks only if time is left.
- [ ] Done when: the 375 px walk reads top to bottom without narration.

### Track D: README (section 5, first item)

- [ ] Identity, live link, demo steps against the live URL, credits (PAP, Redo API, libraries, starter kit), link to `docs/findings.md`, tagline, Hash-Chain Comb. Mention Muse only after it's reached a rung.

### Track E: Deploy + Muse test (main session)

- [ ] Muse probe now: add `log` to `deploy/Caddyfile` and redeploy. You ask Muse about Cache Valley Honey Co. while I watch the request log (`journalctl -u caddy -f`). What Muse fetches goes to track A.
- [ ] ~10:00: merge A, then B, into `phase-08-last-mile`, then one `scripts/deploy.sh`. Merge C and D whenever they're ready (static files and docs, so a redeploy takes about a minute).
- [ ] SDK check: `scripts/validate_pap.py 32-189-214-147.sslip.io`, version pinned.
- [ ] Muse ladder, 10:10–10:40, see `stretch-muse.md`. Record each rung as we reach it.
- [ ] 10:40–11:10: backup recording and rehearsals. 11:10: Redo cleanup (Ana Test plus every rehearsal $36). 11:15: submit.

## Checklist

### 1. Deploy (done 2026-11-06: https://32-189-214-147.sslip.io)

Decided 2026-11-06: one Lightsail box, Caddy in front, real Redo, free `sslip.io` hostname (buy a domain only if Muse rejects it).

**Shape**

```
https://<ip-dashed>.sslip.io  (Caddy, auto HTTPS)
  /api/*          → strip /api → Rust :3000   (dashboard, agent script)
  /.well-known/*  → Rust :3000                (PAP discovery, for Muse and the SDK)
  /poppy/*        → Rust :3000                (PAP session + returns at the root)
  everything else → web/dist, SPA fallback    (dashboard, /consent/<id>)
```

- Same origin, so the dashboard keeps its default `/api` and needs no `VITE_API_URL`. CORS doesn't come into play.
- The agent script uses `BEEHIVE_URL=https://<host>/api`. Discovery hands out relative paths, so they resolve under `/api`.
- Caddy flushes `text/event-stream` on its own, so the live dashboard stream works through it.

**Box:** Lightsail `nano_3_0` ($5/month: 0.5 GB, 2 vCPU, public IPv4), `ubuntu_24_04`, `us-west-2`, static IP (free while attached), ports 22/80/443. The binary and Caddy together use well under 100 MB.

**Build:** local glibc is 2.43, Ubuntu 24.04 has 2.39, so a binary built on this machine won't run there. Build in Docker (`rust:1-bookworm`, glibc 2.36) with the cargo cache mounted. The dashboard is `pnpm build` as is.

**Steps**

- [x] `scripts/deploy.sh <host>`: Docker release build, `pnpm build`, copy binary + `web/dist` + Caddyfile up, restart the service
- [x] Create the box (aws-mcp): instance, static IP, open 443, import the local SSH public key
- [x] Box setup, once: `apt install caddy`, `beehive` user, systemd unit (`Restart=always`, `EnvironmentFile=/etc/beehive.env`, chmod 600)
- [x] `/etc/beehive.env`: `REDO_MODE=real`, `REDO_STORE_ID`, `REDO_API_SECRET`, `REDO_BASE`, `BEEHIVE_JWT_SECRET` (fresh random), `BEEHIVE_WEB_URL=https://<host>`, `BEEHIVE_DEMO_SEED=1`, `PORT=3000`
- [x] HTTPS cert issues for `<host>` (Caddy falls back to ZeroSSL if Let's Encrypt refuses)
- [x] Smoke: `curl https://<host>/.well-known/poppy.json`, `curl https://<host>/api/health`, then `BEEHIVE_URL=https://<host>/api node agent/agent.ts --reset`, approve on the phone
- [ ] Rerun `/wow` against the live URL (walked by hand on 2026-11-06 at 375 px: consent, $36 pass into real Redo, $120 + replay blocks, Live stream, verify 6/6, tamper breaks link 3; restart restores the cold open), on the production build (only the dev server was tested)
- [x] Update AGENTS.md: Deploy and Live URL rows

**Known costs of real mode on a public URL**
- Every restart re-seeds the Redo store and files the cold-open $36 into real Redo (`BEEHIVE_DEMO_SEED=1`). So does every "Reset demo".
- If real Redo is down at startup, the server falls back to the mock with a warning. That's already built.
- Fallback without a redeploy: set `REDO_MODE=mock` in `/etc/beehive.env`, then `systemctl restart beehive`.

**After judging:** delete the instance and release the static IP. An unattached static IP is billed.

### 2. Harden

- [ ] **Rehearse with `--reset` only** (no code). Without it the $36 blocks as "Order o_1 already has a return", then the rate limit blocks everything.
- Code hardening (Redo errors, shared state on the live URL, retries) moved to `stretch-hardening.md` (S2).

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

All folded into D1–D5 under "Parallel tracks". Card order is D5.

## Done when

- [ ] `/wow` passes all 6 checks on the live URL
- [ ] Report written: what you clicked, what you saw
- [ ] Waiting for me to say commit
