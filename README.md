# Beehive

**A safe front door for a customer's own AI agent at any Redo shop.**

https://github.com/user-attachments/assets/189d6a7a-a5a9-4dd1-90c1-2d740ab8e4fb

## The problem

I once had to dig through AllTrails' support chat to get a refund. Soon my own AI agent will do that for me. A new draft standard, the Personal Agent Protocol (PAP, "Poppy"), gives those agents a way into a shop.

Big brands will build for it. A small Utah shop, like a Cache Valley honey farm, can't build agent integrations or hire support staff. So customers will default to Amazon, where returns are easy.

Letting an agent ask for money also raises hard questions. Who is this agent? What did the customer actually allow? What stops it from asking twice, or asking for more?

## What Beehive does

Beehive is a plugin for Redo, the returns platform. It gives every Redo shop a PAP front door:

1. A customer's agent finds the shop on its own.
2. The customer approves what the agent may do, like "returns up to $50".
3. The agent files the return.
4. A Rust gate checks every request. Only a request that passes reaches Redo, and Redo still makes the return decision.
5. Every step goes into a hash chain, so the shop can prove what the customer allowed and spot any edit.

The AI is the customer's own agent, outside the shop. There is no model inside Beehive. Every decision is plain Rust code, so a prompt can't talk its way past a refund cap.

## The demo

The video above walks through it. The demo shop is Utah Cache Valley Honey Co., a made-up honey farm. Kaylee Jensen's agent files three requests under a $50 cap she approved:

- **Passed:** a $36 Beeswax Candle Set that arrived broken. Under the cap, so Redo opened a real return.
- **Blocked:** a $120 Honeycomb Gift Crate. Over the cap, so it never reached Redo.
- **Blocked:** the same $36 request sent again. A duplicate, so no second refund.

Then **Verify proof** shows all 6 links of the hash chain check out. **Edit one byte** changes one digit in the stored request, and the check points at the exact link that broke.

> The live deployment from the event is being shut down. You can run the same demo locally in a few minutes (see [Run it locally](#run-it-locally)).

## Tested with Meta's Muse

During the event I pointed Meta's Muse at the live shop and asked it to return a broken candle set. With no help from me, it found `/.well-known/poppy.json`, opened a PAP session, and filed the $36 return. The gate passed it, Redo opened a real return, and the proof checked out (all 6 links).

It also showed me a gap. Muse opened the consent link meant for the customer and approved its own $50 cap. The build left out customer sign-in, so the consent screen trusts whoever opens it. Binding consent to the customer's sign-in (PAP's OAuth issuer) is the first thing next. Details in [`docs/findings.md`](docs/findings.md).

## How it works

```mermaid
flowchart LR
  A{{"Customer's agent"}} -->|PAP| G{{"Beehive gate (Rust)"}} -->|pass only| R{{"Redo"}}
  classDef agent fill:#0F172A,stroke:#FDE68A,color:#FDE68A,stroke-width:3px
  classDef gate fill:#1E293B,stroke:#F59E0B,color:#F59E0B,stroke-width:4px
  classDef redo fill:#0F172A,stroke:#EA580C,color:#EA580C,stroke-width:3px
  class A agent
  class G gate
  class R redo
```

1. **PAP front door** (Rust, Axum). The agent finds the shop at `/.well-known/poppy.json`, asks for a session at `POST /poppy/session`, and the customer approves the scopes and a refund cap on a consent screen. The session token is a short-lived JWT with a unique token id. The agent reads the customer's orders at `GET /poppy/orders` (needs `poppy:read`), then files at `POST /poppy/returns`.
2. **Rust gate.** Every request runs the same 9 checks in order, and the first one that fails blocks it: session, rate limit per agent (counted only once the token checks out), agent identity, expiry, replay (token id), scope, idempotency (request id), order (the price comes from the order, not the agent), refund cap.
3. **Redo handoff.** A passed request becomes a Redo return through Redo's return import, with the request id as Redo's `idempotencyKey`. Redo has no field for "an agent filed this", so Beehive writes which agent, for whom, under what cap, and the gate's hash into the return's `notes`.
4. **Proof.** Each step (discovery, session, consent, request, gate decision, Redo result) is one link in a SHA-256 hash chain. `GET /verify/{return_id}` recomputes it and reports the first broken link.
5. **Dashboard** (Vite + React). Shows each agent request live over server-sent events: which agent, on whose behalf, what was allowed, the gate's reason, Redo's result, and the proof.

One JSON Schema (`schema/beehive.schema.json`) generates both the Rust and the TypeScript types, so the two sides can't drift apart. `cargo test` runs 11 tests, 7 of them on the gate.

## Run it locally

Needs Rust (edition 2024), Node 24 and pnpm. Run each line from the repo root, in its own terminal.

```bash
cd server && BEEHIVE_DEMO_SEED=1 cargo run   # API on :3000, mock Redo, demo requests seeded
cd web && pnpm install && pnpm dev           # dashboard on http://localhost:5173
BEEHIVE_URL=http://localhost:3000 node agent/agent.ts --reset   # optional: run the agent yourself
```

Open http://localhost:5173. The three requests above are already there. Click **Verify proof** on the $36 card, then **Edit one byte (demo)**.

To run the agent yourself, use the last line. `--reset` empties the demo shop first. It prints a consent link. Open it, approve write access with a $50 cap, and watch the dashboard fill in.

| Task | Command |
|------|---------|
| Real Redo | `cd server && set -a; . ../.env; set +a; REDO_MODE=real cargo run` (keys in `.env`, see `.env.example`) |
| Tests | `cd server && cargo test` |
| Build | `cd server && cargo build --release` · `cd web && pnpm build` |

Without Redo keys the server uses a mock shaped like Redo's real responses.

## How it went at Crimson Hacks 2026

I built Beehive solo at Crimson Hacks 2026 (Nov 6, State of Utah theme), working with AI coding agents. Everything worked on the build side. It didn't place.

What went wrong was the pitch, not the code. I left almost no time to prepare, went straight into the demo without setting up the problem, and the live demo broke. The winning teams had simpler ideas, cleaner UI and smoother demos.

What went right: Redo's judges, including their VP of Engineering, called Beehive cutting-edge. After I explained it in the judging room, they invited me to come talk to their team.

My takeaway: explain the problem before showing the solution. It's why this README now starts with the problem.

## What's next

- **Customer sign-in on the consent screen**, through PAP's OAuth issuer. This closes the hole Muse found.
- **DPoP**, so a stolen session token can't be reused.
- **More PAP actions** beyond filing returns.
- **Asks for Redo:** fields for who filed a return (agent, customer, cap) and for its price and refund type.

## What I found for Redo and PAP

[`docs/findings.md`](docs/findings.md) covers what building on Redo's API and the PAP draft taught me, with evidence. The short version: Redo's idempotency held up, a return can't say that an agent filed it, and an imported return shows as $0.00 store credit. Bug reports with steps are in [`docs/redo-findings.md`](docs/redo-findings.md). The full project write-up is in [`docs/submission.md`](docs/submission.md).

## Repo layout

| Folder | What's in it |
|--------|--------------|
| `server/` | Rust (Axum) service: PAP front door, gate, Redo handoff, proof |
| `web/` | Vite + React dashboard and consent screen |
| `agent/` | Demo agent script (Node) |
| `schema/` | The JSON Schema contract and examples |
| `seed/`, `redo-fixtures/` | Demo shop data and recorded Redo responses |
| `deploy/`, `scripts/` | Lightsail + Caddy deploy |
| `docs/` | Findings, Redo integration notes, write-up, pitch notes |
| `brief/`, `plans/`, `start/`, `.claude/` | Hackathon brief, build plans and agent rules from my starter kit |

## Credits

- **PAP (Personal Agent Protocol, "Poppy"):** [Sierra's announcement](https://sierra.ai/blog/poppy), with [Descope's explainer](https://www.descope.com/learn/post/personal-agent-protocol). PAP is a draft. Beehive builds a subset: discovery, sessions with consent, and filing returns.
- **Redo API:** [docs for agents (`llms.txt`)](https://developers.redo.com/llms.txt) and the [OpenAPI spec](https://developers.redo.com/api-schema/openapi.yaml). How the real API behaved in my tests: [`docs/redo-integration.md`](docs/redo-integration.md).
- **Server:** [Axum](https://github.com/tokio-rs/axum), [Tokio](https://tokio.rs), [jsonwebtoken](https://github.com/Keats/jsonwebtoken), [sha2](https://github.com/RustCrypto/hashes), [reqwest](https://github.com/seanmonstar/reqwest), [serde](https://serde.rs), [typify](https://github.com/oxidecomputer/typify), tower-http, tracing, chrono, uuid.
- **Dashboard:** [React](https://react.dev), [Vite](https://vite.dev), [TanStack Query](https://tanstack.com/query), [Tailwind CSS](https://tailwindcss.com), [shadcn/ui](https://ui.shadcn.com) on [Radix](https://www.radix-ui.com), [zod](https://zod.dev), [Lucide](https://lucide.dev), [Geist](https://vercel.com/font), [json-schema-to-typescript](https://github.com/bcherny/json-schema-to-typescript).
- **Hosting:** AWS Lightsail, [Caddy](https://caddyserver.com), [sslip.io](https://sslip.io).
- **Starter kit:** my own [Hackathon Starter template](https://github.com/melizalde-ds/hackathon-starter), inspired by [Juan Lizarazo's hackathon-starter-kit](https://github.com/juanlizarazo/hackathon-starter-kit). It holds only agent rules, slash commands and plan templates. No app code: everything in `server/`, `web/`, `agent/` and `schema/` was written during the event.

## License

MIT, see [`LICENSE`](LICENSE).
