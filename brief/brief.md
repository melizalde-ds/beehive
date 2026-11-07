# Brief: Beehive

Crimson Hacks 2026 · solo with agents · Vite + Rust · keyboards down 11:30 PM · judges at your table 11:35 PM

## One-liner
Beehive is a plugin for Redo that lets a customer's own AI agent handle returns and claims through the Personal Agent Protocol (PAP). It checks who the agent is and what the customer allowed, then hands the work to Redo.

## Positioning (plugin, not competitor)
"Redo's MCP server lets the merchant's AI in. Beehive opens the other door: the customer's own AI agent. Redo's review and fraud agents still make the call."

## Opening line
"I had to dig for AllTrails' support chat to get my refund. Soon my own agent will do that for me, but only if the shop speaks PAP. Beehive makes every Redo shop, down to a Cache Valley honey farm, ready for that."

## How this makes Utah better
Small Utah shops (Utah's Own producers, local makers) can't build agent integrations or hire support. Beehive gives them the same agent-ready service as big brands, so Utahns don't default to Amazon for easy returns, and the money stays local.

## Targets
1. Best Commerce Hack: helps Redo merchants serve customers' agents
2. Best Technical Build: Rust gate on the money path, provable evidence chain
3. Best Overall: works live, Utah story, clear in 3 min

## MVP: exactly 3 pieces
1. **PAP front door** (Rust, Axum)
   - `/.well-known/poppy.json`, `POST /poppy/session` (guest or signed-in, scopes `poppy:read` / `poppy:write`, short expiry, unique token id)
   - `POST /poppy/returns` for agents to file a return or claim
   - A script acts as the personal agent: discover → session → file a return
2. **Rust gate** (the core)
   - Agent checks: valid session, agent identity, customer consent (scope + refund cap), expiry, replay (token id), idempotency (request id), rate limit per agent
   - On pass: hands off to Redo through a `RedoClient` trait (mock now, real sandbox if keys)
   - SHA-256 hash chain of each step: discovery, session, consent, request, gate decision, Redo result. `verify` reports the first broken link.
3. **Shop dashboard** (Vite + React)
   - Agent-filed returns: which agent, on whose behalf, what was allowed, Redo's result, gate blocks with reasons, a "Verify proof" button

Not in MVP: human chat (Redo has it), our own return decisions (Redo does them), checkout, Stripe, DPoP, real OAuth, photo grading.

## Redo mock (match the real API shapes)
- `POST /stores/{storeId}/custom/orders`: sync seeded orders for the tiny shop
- Create a return and get a review result: LOW / MEDIUM / HIGH risk → approved / manual review / denied
- `POST /stores/{storeId}/shipments/buy`: return label for approved returns

## Seed data (invented shop)
Utah Cache Valley Honey Co. (fictional, modeled on Utah's Own producers)
- Honey Subscription Box $28/mo · Raw Wildflower Honey $14 · Beeswax Candle Set $36 · Honeycomb Gift Crate $120
- 3 customers with seeded orders

## API contract (lock first)
`POST /gate/returns` (Rust)
```json
{ "request_id": "uuid", "session_token": "jwt", "agent_id": "muse-demo",
  "order_id": "o_1", "reason": "damaged|wrong_item|changed_mind|forgot_cancel", "amount": 36.00 }
```
Response
```json
{ "gate": "pass|block", "reason": "Within $50 cap approved by customer",
  "redo": { "return_id": "mock_ret_123", "risk": "LOW", "status": "approved", "label_url": "..." },
  "chain_head": "sha256..." }
```
`GET /evidence/:return_id` · `GET /verify/:return_id`

## Demo script (3 min)
- **0:00-0:25** Opening line. Small Utah shops can't build for agents.
- **0:25-1:45**
  1. Terminal: the personal agent reads `poppy.json`, starts a session, and the customer approves write access with a $50 refund cap.
  2. The agent files "candle set arrived broken" ($36). The gate passes it, Redo opens the return (approved). The dashboard updates live.
  3. WOW: the same agent tries the $120 crate. The gate blocks it: over the customer's $50 cap. Then it replays the first request: blocked as a duplicate, no double refund.
  4. Verify proof: green. Edit one byte: it breaks at that link.
- **1:45-2:30** How: PAP front door, Rust gate, Redo does the returns, hash-chained evidence.
- **2:30-3:00** Next: real Redo sandbox, Utah's Own onboarding through Redo's custom orders endpoint, more PAP channels.

## Build plan (from 6:15)
- **6:15-6:30** Redo table (see asks). Scaffold Vite + Axum. Paste brief.
- **6:30-7:45** Rust gate: session JWT verify, consent/cap, replay, idempotency, rate limit, hash chain, verify. Unit tests per check.
- **7:45-8:30** Redo mock + handoff. **CHECKPOINT 8:30: agent script files a return end to end.**
- **8:30-9:15** poppy.json, session endpoint, consent screen.
- **9:15-10:00** Shop dashboard + verify/tamper.
- **10:00-10:30** Real Redo sandbox if keys. Otherwise polish the mock.
- **10:30-11:00** Polish, Utah styling, backup recording.
- **11:00-11:25** Rehearse 3 times. Freeze.
- **11:30** Keyboards down, slip. **11:35** at your table.

## Cut list (in order)
1. Real Redo (keep the mock)
2. Rate limit
3. Consent screen UI (pre-approved session, shown on screen)
Never cut: the $120 block, the replay block, verify + tamper.

## Judge Q&A
- **"Doesn't Redo already do AI returns?"** "Yes, and Beehive uses it. Redo decides the return. Beehive decides whether the agent asking is allowed to ask."
- **"Why not just use Redo's MCP server?"** "That's for the merchant's own tools, behind an allowlist. A customer's agent needs its own door, with the customer's consent."
- **"Prompt injection?"** "The agent can only request. Rust checks consent, caps and replay before anything reaches Redo."
- **"Bearer tokens?"** "Short expiry + unique token id block replay. DPoP is next."
- **"Is PAP real?"** "Draft from Sierra and Meta, Oct 2026, with Shopify, Stripe, Walmart, Visa, Mastercard and OpenAI as partners. I built a subset."
- **"Why Rust?"** "It's on the money path. It has to be correct and testable."

## Rules check
- Confirm your starter kit is OK under "start from zero." Cite it.
- README cites PAP, the Redo API and libraries.
