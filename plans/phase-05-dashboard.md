# Phase 05: Shop dashboard + consent screen

**Goal:** The shop sees every agent-filed return (pass or block, with reasons and Redo's status) update live, and can prove any one with "Verify proof" and break it with a one-byte tamper. The customer approves an agent's session with a refund cap.
**Brief requirements covered:** MVP piece 3 (agent-filed returns: which agent, on whose behalf, what was allowed, Redo's result, gate blocks with reasons, "Verify proof"). Demo steps 1 (consent, $50 cap), 2 (dashboard updates live), 4 (verify + tamper).
**Depends on:** phase 01 contract. Phase 02 endpoints (`/returns`, `/evidence`, `/verify`, `/tamper`) for the walk. Phase 04 for `GET /poppy/session/:id` + `POST /consent/:id`.
**Touches only:** `web/`, `plans/phase-05-dashboard.md`, and for SSE (asked): `server/src/gate.rs` (`file_return` + `stream_returns`), `server/src/main.rs` (one state field + one route), `server/Cargo.toml`

## Checklist

- [x] Tailwind + shadcn/ui (RX-03), honey theme as CSS variables (RX-04), zod (RX-06)
- [x] `src/api/`: zod-checked `getJson` / `postJson`, validators built from `schema/beehive.schema.json` with `z.fromJSONSchema` (nothing hand-copied); one hook per endpoint
- [x] Live: `EventSource` on `/returns/stream` invalidates `useReturns`; polls every 3 s while the stream is down
- [x] Server: `GET /returns/stream` (SSE), one message after every gate decision, via a tokio broadcast channel
- [x] Dashboard: one card per attempt (agent → customer, product, amount, reason, pass/block + reason, Redo status, allowed scopes + cap), newest first
- [x] Verify proof: green/red result, the chain with the broken link marked, "Edit one byte (demo)" on the request link
- [x] Consent screen at `/consent/:session_id` or `?consent=<id>`: requested scopes in plain words, refund cap (lower only), Approve / Deny
- [x] Loading, empty and friendly error state on every screen (HACK-13)
- [x] Walk at 375 px and desktop, with phase 04's agent script end to end

## Questions asked

- Live updates? **SSE.** The server route comes after 03/04 land; correct course then.
- Consent screen URL? **Query param, no router.** The contract already says `/consent/:session_id`, so both work (no router needed for either).
- Run where? **This session**, worktree `../crimson-hacks-project-phase-05`.
- Build the SSE route here once phase 04 landed? **Yes.** Crate: **`tokio-stream`** (plus tokio's `sync` feature).

## Assumptions made

- **Branch base:** fast-forwarded to `phase-04-front-door` (`3f1fad6`, includes phase 03) before wiring consent, as phase 04's report asked.
- **SSE payload is ignored**: any message on `/returns/stream` means "refetch `/returns`". No schema change.
- **The signal fires in `file_return`** (after `gate::run` returns), not inside `finish`, so the gate logic is untouched. A Redo 502 sends nothing (nothing was recorded).
- **Poll fallback** (3 s) while the stream isn't open, so a dropped stream never freezes the screen.
- **Consent screen can't name the agent**: `SessionResponse` has no `agent_id` / customer. It names the shop and what's asked. Adding them is a schema change (ask first).
- "Edit one byte" tampers the `request` link (36 becomes 46, visible on screen).
- **shadcn's `cn` package** (`github.com/shadcn-ui/cn`) replaces clsx + tailwind-merge; it's what `shadcn add` now imports. Dependency versions pinned exact (RX-02).

## Walk (what I clicked, what I saw)

Server on :3005 (no demo session), dashboard on :5175, headless Chromium at 375 px, `agent/agent.ts` as the agent.

1. Opened the dashboard: empty state, status dot "Live" (SSE open).
2. Ran the agent. It printed the consent link; opened it: "Your AI agent is asking to act for you", two scopes, cap 50.
3. Typed 80 in the cap: Approve disabled. Typed 50, clicked Approve: "Approved … refunds up to $50.00 until 8:08 PM".
4. Without a reload, three cards arrived on the dashboard: $36 Passed (Redo Approved, id `ee1f…`), $120 "Blocked: Refund cap", replay "Blocked: Duplicate".
5. Verify proof on the pass: green, "All 6 links check out". Edit one byte: red, "Link 3 (request) was changed after it was recorded", link 3 outlined and shows `amount: 46`.
6. Verify proof on the $120 block: green, 5 links (no Redo link).
7. New session via `?consent=<id>`, clicked Deny: "Denied". Unknown session id: "No session ses_nope at this shop. Ask your agent to start again."
8. Page width at 375 px: 375 (no side scroll). No console errors.

## Polish left for phase 08

- The gate decision link shows its `checks` array as raw JSON. Readable, not pretty.

## Done when

- [ ] This phase's step of the demo path works in the running app
- [ ] Report written: what you clicked, what you saw
- [ ] Waiting for me to say commit
