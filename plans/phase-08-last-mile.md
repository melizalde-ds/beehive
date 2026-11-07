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

Questions 1 (shared state) and 2 (Redo error mid-run) moved to `stretch-hardening.md`.

3. **Card order:**
   (a) Oldest first, so the story reads pass → cap block → duplicate. *(recommended)*
   (b) Keep newest first, pin the $120 block to the top.

## Done when

- [ ] `/wow` passes all 6 checks on the live URL
- [ ] Report written: what you clicked, what you saw
- [ ] Waiting for me to say commit
