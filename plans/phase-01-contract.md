# Phase 01: Contract + seed data

**Goal:** Lock every shape that crosses Rust ↔ TypeScript, seed Cache Valley Honey Co., and stand up empty runnable Axum and Vite apps that both use the generated types. This unblocks phases 02-05.
**Brief requirements covered:** API contract (lock first), Seed data, Redo mock result shape (`redo` object only), scaffold step at 6:15.
**Depends on:** nothing
**Touches only:** `schema/`, `seed/`, `server/`, `web/`, `.env.example`, `plans/phase-01-contract.md`

## Checklist

- [x] `schema/beehive.schema.json`: gate request/response, Redo result, evidence, verify, tamper, PAP discovery, session, consent, dashboard return list, seed, API error
- [x] `schema/examples/`: one example per shape, `schema/README.md`: endpoint table with the owning phase
- [x] `seed/cache-valley-honey.json`: shop, 4 products, 3 customers, orders, the demo agent
- [x] `server/`: Axum binary, types generated from the schema (typify), serves `GET /health` and `GET /shop`, test that the seed and every example parse
- [x] `web/`: Vite + React + TS, types generated from the schema (json-schema-to-typescript), one page that loads `/shop` through the dev proxy
- [x] Walk: `cargo run`, `pnpm dev`, open the page, see the shop and its products

## Assumptions made

Confirmed by you after the phase: `RedoResult` aligned with Redo's real API (Redo's return `status`, no risk score, label from `postageLabelUrl`; see `docs/redo/`). Keep all contract additions (return_id, blocked_by, session/consent flow, GET /returns, tamper), and build the real consent screen (not pre-approved). You also kept the remaining choices below.

- One schema file (`beehive.schema.json`) with a `definitions` entry per shape, so typify and json2ts share types instead of duplicating them per file.
- `GateResponse` gets a top-level `return_id` (Beehive's id for every attempt, pass or block) plus `blocked_by` (which check fired). The brief's shape had no id for blocked attempts, and the dashboard needs one for evidence and verify.
- Added shapes the brief implies but didn't spell out: session request/response with a `pending_consent` → `active` flow, consent decision, the dashboard list (`GET /returns`), and a demo-only tamper endpoint. If the consent screen gets cut, the session can come back `active` right away with the same shape.
- Money is a JSON number in USD. Phase 02 converts to integer cents before comparing (confirmed).
- Example hashes are made up. Phase 02 sets the hashing rule.
- The Redo API's own request/response shapes stay inside Rust (phase 03). Only the `redo` result crosses to TypeScript.
- The dashboard calls `/api/*`. In dev, Vite proxies that to `localhost:3000`, so the consent page route `/consent/:id` can't collide with the API route. Prod sets `VITE_API_URL`. CORS: phase 02 adds tower-http (confirmed).
- Tailwind, shadcn, zod and React Router are left to phase 05.
- Customer emails use `example.com`.

## Done when

- [x] This phase's step of the demo path works in the running app
- [x] Report written: what you clicked, what you saw
- [x] Waiting for me to say commit
