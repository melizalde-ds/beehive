# Beehive

**The Personal Agent Gate for Redo Merchants.**

Beehive is a plugin for Redo that lets a customer's own AI agent file returns and claims through the Personal Agent Protocol (PAP). It checks who the agent is and what the customer allowed, then hands the work to Redo.

**Live:** https://32-189-214-147.sslip.io

Built solo for Crimson Hacks 2026. The demo shop is Cache Valley Honey Co., a made-up Utah honey farm.

## Try it (2 minutes)

1. **Open the dashboard:** https://32-189-214-147.sslip.io. No login. It opens with three requests from Kaylee Jensen's agent already filed:
   - **Passed:** a $36 Beeswax Candle Set, arrived broken. It's under the $50 cap Kaylee approved, so it went to Redo, and Redo opened the return.
   - **Blocked:** a $120 Honeycomb Gift Crate. It's over Kaylee's $50 cap, so it never reached Redo.
   - **Blocked:** the same $36 request sent again. It's a duplicate, so there's no second refund.
2. **Check the proof:** on the $36 card, click **Verify proof**. All 6 links check out.
3. **Tamper with it:** click **Edit one byte (demo)**. It changes one digit in the stored request without re-hashing it. The check now says **Proof is broken** and points at the link that changed.
4. **Optional, run the agent yourself** (Node 24, from a clone of this repo):
   ```bash
   BEEHIVE_URL=https://32-189-214-147.sslip.io/api node agent/agent.ts --reset
   ```
   `--reset` empties the demo shop first. The script prints a consent link. Open it, approve write access with a $50 cap, and watch the dashboard: the $36 passes, then the $120 and the replay are blocked.

The live demo shares one in-memory state, so you may see another visitor's run. Click **Reset demo** on the dashboard to bring back the three requests above.

## Tested with Meta's Muse

On 2026-11-06 we pointed Meta's Muse at the live shop and asked it to return a broken candle set. With no help from us, it found `/.well-known/poppy.json`, opened a PAP session, and filed the $36 return. The gate passed it, Redo opened a real return, and the proof checks out (all 6 links).

It also showed us a gap. Muse opened the consent link meant for the customer and approved its own $50 cap. The brief left out customer sign-in, so the consent screen trusts whoever opens it. Binding consent to the customer's sign-in (PAP's OAuth issuer) is the first thing next. Details in [`docs/findings.md`](docs/findings.md).

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

The Hash-Chain Comb: agent, gate, Redo.

1. **PAP front door** (Rust, Axum). The agent finds the shop at `/.well-known/poppy.json`, asks for a session at `POST /poppy/session`, and the customer approves the scopes and a refund cap on a consent screen. The session token is a short-lived JWT with a unique token id. The agent files at `POST /poppy/returns`.
2. **Rust gate.** Every request runs the same checks in order, and the first one that fails blocks it: rate limit per agent, session, agent identity, expiry, replay (token id), scope, idempotency (request id), order, refund cap. The agent can only ask. A blocked request never reaches Redo.
3. **Redo handoff.** A passed request becomes a Redo return through Redo's return import, with the request id as Redo's `idempotencyKey`. From there it's Redo's return: Beehive makes no return decisions. Redo has no field for "an agent filed this", so Beehive writes which agent, for whom, under what cap, and the gate's hash into the return's `notes`.
4. **Proof.** Each step (discovery, session, consent, request, gate decision, Redo result) is one link in a SHA-256 hash chain. `GET /verify/{return_id}` recomputes it and reports the first broken link.
5. **Dashboard** (Vite + React). Shows each agent request live: which agent, on whose behalf, what was allowed, the gate's reason, Redo's result, and the proof.

**Where the model sits:** nowhere in Beehive. The AI is the customer's own agent, outside the shop. Every gate decision is plain Rust code, and the return itself is Redo's. The demo agent in `agent/agent.ts` is a script, so the demo never depends on a model call.

The live URL runs against a real Redo store. If Redo can't be reached at startup, the server falls back to a mock shaped like Redo's real responses.

## Run it locally

Needs Rust (edition 2024), Node 24 and pnpm. Run each line from the repo root, in its own terminal.

```bash
cd server && BEEHIVE_DEMO_SEED=1 cargo run   # API on :3000, mock Redo, cold-open demo seeded
cd web && pnpm install && pnpm dev           # dashboard on http://localhost:5173
BEEHIVE_URL=http://localhost:3000 node agent/agent.ts --reset   # then approve the consent link it prints
```

| Task | Command |
|------|---------|
| Real Redo | `cd server && set -a; . ../.env; set +a; REDO_MODE=real cargo run` (keys in `.env`, see `.env.example`) |
| Contract check | `cd server && cargo test` |
| Build | `cd server && cargo build --release` · `cd web && pnpm build` |

## What we found for sponsors

[`docs/findings.md`](docs/findings.md): what building on Redo's API taught us, with evidence. The short version: Redo's idempotency held up, a return can't say that an agent filed it, and an imported return shows as $0.00 store credit. Bug reports with steps are in [`docs/redo-findings.md`](docs/redo-findings.md).

## Credits

- **PAP (Personal Agent Protocol, "Poppy"):** [Sierra's announcement](https://sierra.ai/blog/poppy), with [Descope's explainer](https://www.descope.com/learn/post/personal-agent-protocol). PAP is a draft. Beehive builds a subset: discovery, sessions with consent, and filing returns.
- **Redo API:** [docs for agents (`llms.txt`)](https://developers.redo.com/llms.txt) and the [OpenAPI spec](https://developers.redo.com/api-schema/openapi.yaml). How the real API behaved in our tests: [`docs/redo-integration.md`](docs/redo-integration.md).
- **Server:** [Axum](https://github.com/tokio-rs/axum), [Tokio](https://tokio.rs), [jsonwebtoken](https://github.com/Keats/jsonwebtoken), [sha2](https://github.com/RustCrypto/hashes), [reqwest](https://github.com/seanmonstar/reqwest), [serde](https://serde.rs), [typify](https://github.com/oxidecomputer/typify), tower-http, tracing, chrono, uuid.
- **Dashboard:** [React](https://react.dev), [Vite](https://vite.dev), [TanStack Query](https://tanstack.com/query), [Tailwind CSS](https://tailwindcss.com), [shadcn/ui](https://ui.shadcn.com) on [Radix](https://www.radix-ui.com), [zod](https://zod.dev), [Lucide](https://lucide.dev), [Geist](https://vercel.com/font), [json-schema-to-typescript](https://github.com/bcherny/json-schema-to-typescript).
- **One contract:** `schema/beehive.schema.json` generates both the Rust types (typify) and the TypeScript types.
- **Hosting:** AWS Lightsail, [Caddy](https://caddyserver.com), [sslip.io](https://sslip.io).
- **Starter kit:** my own [Hackathon Starter template](https://github.com/melizalde-ds/hackathon-starter), inspired by [Juan Lizarazo's hackathon-starter-kit](https://github.com/juanlizarazo/hackathon-starter-kit). It holds only agent rules, slash commands and plan templates (`.claude/`, `AGENTS.md`, `brief/`, `plans/`, `start/`). No app code: everything in `server/`, `web/`, `agent/` and `schema/` was written during the event.

## License

MIT, see [`LICENSE`](LICENSE).
