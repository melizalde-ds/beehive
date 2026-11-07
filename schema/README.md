# Contract

`beehive.schema.json` is the single source of truth for every shape that crosses Rust ↔ TypeScript. Each shape is a `definitions` entry. `examples/<Shape>.json` (or `<Shape>.<case>.json`) holds one seeded example of it.

- Rust types: generated at build time by `typify` in `server/src/types.rs`.
- TypeScript types: `pnpm gen:types` in `web/` writes `web/src/types/generated/beehive.ts`.
- Never edit generated types. Change the schema (ask first once phases 02-05 have started), then regenerate.
- `cargo test` in `server/` checks that the seed and every example parse into the generated types.
- The hashes in the examples are made up. Phase 02 sets the hashing rule.

## Endpoints (all served by the Rust service)

| Method + path | Body | Response | Phase |
|---|---|---|---|
| `GET /health` | | `ok` | 01 |
| `GET /shop` | | `Seed` | 01 |
| `GET /.well-known/poppy.json` | | `PoppyDiscovery` | 04 |
| `POST /poppy/session` | `SessionRequest` | `SessionResponse` | 04 |
| `GET /poppy/session/:session_id` | | `SessionResponse` | 04 |
| `POST /consent/:session_id` | `ConsentDecision` | `SessionResponse` | 04 (called by the consent screen, 05) |
| `POST /poppy/returns` | `GateRequest` | `GateResponse` | 04 (forwards to the gate) |
| `POST /gate/returns` | `GateRequest` | `GateResponse` | 02 |
| `GET /evidence/:return_id` | | `Evidence` | 02 |
| `GET /verify/:return_id` | | `VerifyResult` | 02 |
| `POST /evidence/:return_id/tamper` | `TamperRequest` | `Evidence` | 02 (demo only) |
| `GET /returns` | | `ReturnList` | 02 |

Every non-2xx response has an `ApiError` body.

`return_id` at the top of `GateResponse` is Beehive's id for one attempt (pass or block). `redo.return_id` is Redo's own id for the return.

The consent screen is the dashboard route `/consent/:session_id` (phase 05).
