# Beehive: submission draft

## Project name + tagline

**Beehive**: The Personal Agent Gate for Redo Merchants.

## Elevator pitch (under 200 characters)

Beehive lets a customer's own AI agent file returns at a Redo shop through PAP. A Rust gate checks consent, refund cap and replays first, and every step is hash-chained.

## Inspiration

I had to dig for AllTrails' support chat to get my refund. Soon my own agent will do that for me, but only if the shop speaks the Personal Agent Protocol (PAP). Big brands will build for that. Small Utah shops, like Utah's Own producers and local makers, can't build agent integrations or hire support. I wanted every Redo shop, down to a Cache Valley honey farm, to be ready for a customer's agent, so Utahns don't default to Amazon for easy returns.

## What it does

Beehive is a plugin for Redo. A customer's AI agent finds the shop at `/.well-known/poppy.json`, opens a PAP session, and the customer approves write access with a refund cap on a consent screen. The agent then files a return. A Rust gate checks who the agent is and what the customer allowed. Only a request that passes reaches Redo, which opens a real return. The shop's dashboard shows each request live: which agent, on whose behalf, what was allowed, the gate's reason, Redo's result, and a "Verify proof" button.

The demo shop is Utah Cache Valley Honey Co., a made-up honey farm:
- **Passed:** a $36 Beeswax Candle Set, arrived broken. It's under Kaylee's $50 cap, so Redo opened the return.
- **Blocked:** a $120 Honeycomb Gift Crate. Over the cap, so it never reached Redo.
- **Blocked:** the same $36 request sent again. A duplicate, so no second refund.
- **Tamper:** "Edit one byte" changes one digit in the stored request, and the proof check points at the broken link.

## How we built it

- **PAP front door (Rust, Axum):** discovery, `POST /poppy/session`, a consent screen, `GET /poppy/orders` (the customer's own orders, needs `poppy:read`), and `POST /poppy/returns`. The session token is a short-lived JWT with a unique token id.
- **Rust gate:** every request runs the same checks in order, and the first failure blocks it: session, rate limit per agent (counted only once the token checks out), agent identity, expiry, replay (token id), scope, idempotency (request id), order (the price comes from the order, not the agent), refund cap.
- **Redo handoff:** a passed request becomes a return through Redo's real return import, with the request id as Redo's `idempotencyKey`. Beehive makes no return decisions; Redo does.
- **Proof:** each step (discovery, session, consent, request, gate decision, Redo result) is one link in a SHA-256 hash chain. `GET /verify/{return_id}` recomputes it and reports the first broken link.
- **Tests:** `cargo test` runs 11 tests, 7 of them on the gate (pass, over the cap, duplicate, expired token, a price that doesn't match the order, $0 or less, junk tokens against the rate limit), plus contract checks that every schema example and the seed parse.
- **Dashboard:** Vite + React, live over server-sent events.
- **One contract:** `schema/beehive.schema.json` generates both the Rust types (typify) and the TypeScript types.
- **Hosting:** a Docker build on AWS Lightsail behind Caddy, using sslip.io for the HTTPS hostname. The live URL runs against a real Redo store, with a mock shaped like Redo's real responses as a fallback.
- **No model in the decision path.** The AI is the customer's own agent, outside the shop. Every gate decision is plain Rust. The demo agent is a script, so the demo never depends on a model call.

## Challenges we ran into

- **Redo has no way to say an agent filed a return.** The import body has no field for who filed it, which agent, or what the customer allowed. I put the proof in Redo's free-text `notes`: the agent, the customer, the cap, and the gate decision hash that `verify` checks.
- **An imported return shows as $0.00 store credit in Redo.** The import body has no field for price, refund type or the order it belongs to. So Redo shows "Return value $0.00, Store credit", even when the order was synced with the $36 price. The dashboard shows the $36 from Beehive's own record.
- **Rough edges in Redo's API:** `GET /returns/{returnId}` returns 500 for imported returns, so I can't read the label. The store's return list doesn't include imported returns. A bad `productId` gets a 400 with `"detail":"[object Object]"`. A bulk import can answer 200 and still have per-item errors.
- **PAP discovery was stricter than the draft notes I built from.** The PAP SDK also wants `/.well-known/oauth-protected-resource`, `protected_resources`, and token and revocation endpoints. A guest-only shop has to publish placeholder sign-in URLs to pass.
- **Muse first knocked with a GET.** Meta's Muse found the PAP discovery file, then sent `GET /poppy/session` three times and got 405s before it worked out it needed a POST. Afterwards I made that GET answer with how to POST a session, so the next agent doesn't have to guess.
- **Muse approved its own consent.** On the run, Muse opened the consent link meant for the customer and approved the $50 cap itself. The brief left out customer sign-in, so the consent screen trusts whoever opens it.

## Accomplishments that we're proud of

- A third-party agent used it. At 22:05 MST, Meta's Muse found the shop, opened a PAP session and filed the $36 return with no help from me. The gate passed it, Redo opened a real return, and the proof verified 6 of 6 links.
- It runs live against a real Redo store, not only a mock.
- Redo's `idempotencyKey` held up in my tests: a replay with the same key returned the same return id. Beehive blocks the replay before it reaches Redo, and Redo's key is the backup.
- Change one byte of the stored record and the proof check names the exact link that broke.
- One JSON Schema drives both the Rust and the TypeScript types.

## What we learned

- Consent has to be tied to the customer's sign-in, not to whoever holds the link. Muse showed me that in one click.
- Agents that browse can find a shop on their own, but going from reading a shop to acting for a customer is still hard for them. A wrong-method request should get an answer that says what to send.
- Keeping the model out of the money path keeps the gate simple to test. The agent can only ask; plain code decides.
- Redo's published `llms.txt` and OpenAPI spec let me plan the integration straight from the docs. Testing against a real store showed what the docs don't say.

## What's next for Beehive

- **Customer sign-in on the consent screen**, through PAP's OAuth issuer. This closes the hole Muse found, and it's first.
- **DPoP**, so a stolen session token can't be reused.
- **More PAP channels** beyond filing returns.
- **Onboarding Utah's Own producers** through Redo's custom orders endpoint.
- Asks for Redo: fields for who filed a return (agent, customer, cap) and for its price and refund type.

## Built with

Rust, Axum, Tokio, jsonwebtoken, sha2 (SHA-256), reqwest, serde, typify, tower-http, tracing, TypeScript, React, Vite, TanStack Query, Tailwind CSS, shadcn/ui, Radix, zod, Lucide, Node.js, JSON Schema, Redo API, Personal Agent Protocol (PAP), Docker, AWS Lightsail, Caddy, sslip.io

## Links

- **Live demo:** https://32-189-214-147.sslip.io
- **Repo:** https://github.com/melizalde-ds/crimson-hacks-project

## Prize categories

- **Best Commerce Hack:** it lets any Redo merchant, even a tiny Utah honey farm, serve customers' own AI agents, with Redo still making the return decision.
- **Best Technical Build:** a Rust gate sits on the money path with nine ordered checks, and a SHA-256 hash chain proves every step from discovery to Redo's result.
- **Best Overall:** it works live against a real Redo store, a real third-party agent (Meta's Muse) used it, and the Utah story fits in three minutes.
