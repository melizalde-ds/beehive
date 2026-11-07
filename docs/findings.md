# Findings for sponsors

Things we found while building Beehive that a sponsor would want to hear: gaps in their API or docs, bugs, surprising results, and what we did about them. Every claim links to evidence in this repo.

Add an entry as soon as you hit one. Newest first.

## Template

### [Sponsor]: [one-line finding]

- **When:** phase ##, HH:MM
- **What we found:**
- **Evidence:** file path, command, or response
- **What we did:**
- **Pitch line:** one sentence to say at the table

## Entries

### Meta Muse: it ran the PAP flow end to end, and approved its own consent

- **When:** phase 08 track E, 2026-11-06 21:58 MST, live URL
- **What we found:** Through an in-browser API client (Hoppscotch), Muse sent `POST /poppy/session` as `muse-demo`, then opened the `consent_url` meant for the customer and clicked Approve on the $50 cap itself, 18 seconds later. The user confirmed they didn't click it. Muse then polled the session for its token and sent `POST /poppy/returns` (o_1, damaged, $36). The gate blocked it at Order only because the cold-open demo had already returned o_1. Every check before that passed, including the session and scope that Muse had granted itself.
- **Evidence:** `GET /api/evidence/bh_0004` (discovery, session, consent `approved: true` at 04:58:19Z, request, block at `order`); `journalctl -u caddy`: `POST /poppy/session`, `GET /consent/ses_c1e9…`, `POST /api/consent/ses_c1e9…`, all from 104.29.192.79
- **Then:** after emptying the demo shop, Muse did it again at 22:05 MST (session, self-approved consent, return). The gate passed it as `bh_0001` and Redo created return `6aeeb19bebc57cda8dab553f`; `verify` says all 6 links check out. So a third-party agent filed a real Redo return through PAP, with the proof chain intact.
- **What we did:** Nothing in code. The brief cuts customer sign-in, so the consent screen trusts whoever opens it. This is the exact hole that PAP's sign-in (OAuth through `auth.issuer`) closes, and why it's first in "Next".
- **Pitch line:** "We pointed Meta's Muse at our shop. It found PAP, opened a session, then approved its own refund cap. The consent step must be bound to the customer's sign-in, not to whoever holds the link."

### Meta Muse: it finds PAP discovery but only sends GETs, so it can't open a session

- **When:** phase 08 track E, 2026-11-06 ~22:50, live URL with Caddy access logging
- **What we found:** Asked to return a candle set at Cache Valley Honey Co., Muse (IP 104.29.192.79, desktop Chrome user agent) loaded the dashboard, `/api/shop`, `/api/returns`, the proof endpoints and `/.well-known/poppy.json`. Then it sent `GET /poppy/session` three times. PAP opens a session with a `POST`, so each got a 405. Its browsing tool reads pages; we saw no POST, no `DPoP` header and no OAuth calls.
- **Evidence:** `journalctl -u caddy` on the box: `GET /poppy/session 405` ×3 from 104.29.192.79, right after `GET /.well-known/poppy.json 200`
- **What we did:** Nothing yet. Rung 1 of the Muse ladder (`plans/stretch-muse.md`).
- **Pitch line:** "Muse found our PAP door on its own, then knocked with a GET. The step from reading a shop to acting for a customer is the gap."

### PAP: the SDK's discovery checks differ from the draft 0.3 notes we built from

- **When:** phase 08 track A, 2026-11-06 ~22:00, read from the `personal-agent-protocol` 0.30.0 wheel source (draft 0.3)
- **What we found:**
  - The SDK finds the issuer through `GET /.well-known/oauth-protected-resource` (RFC 9728), not only `auth.issuer`. If `auth.issuer` is set and that URL is 404, discovery fails with `issuer_not_found`.
  - The OAuth metadata must list `protected_resources` containing `https://{domain}` (`models.py`, `protected_resources: list[str] = Field(min_length=1)`). Our notes said `poppy_domains`.
  - It also wants `token_endpoint`, `revocation_endpoint` and `grant_types_supported` including `refresh_token`. So a shop that only offers guest sessions has to advertise sign-in endpoints to pass, or leave `auth.issuer` out and look like it has no sign-in at all.
  - `agent.protocols[].endpoint` must be an absolute HTTPS URL, while our own `endpoints` are relative.
- **Evidence:** `personal_agent_protocol/models.py` (`protected_resources`, `REFRESH_TOKEN_GRANT`), `discovery.py`; branch `phase-08-a-muse`, `server/src/poppy.rs` (`protected_resource`, `oauth_metadata`)
- **What we did:** Served all three `/.well-known` documents. Token and revoke URLs are placeholders (404), since the brief cuts real OAuth. Kept `poppy_domains` too.
- **Pitch line:** "To pass PAP discovery a guest-only shop still has to publish OAuth token endpoints, so the spec could use a clear 'guest only' mode."

### Redo: an imported return is worth $0.00 and always store credit

- **When:** phase 06, 2026-11-06, real-mode walk in Redo's dashboard
- **What we found:** Every return imported through `POST /stores/{storeId}/returns/bulk` shows "Retail price $0.00, Return value $0.00, Store credit", even when the order was synced first with the $36.00 line price (the smoke test does this). The import body has no field for a price, a refund type (refund vs store credit) or the Redo order it belongs to. `externalReferenceId` is documented as the merchant's own RMA id, not an order link. So Redo doesn't link the return to the synced order: it also shows "No address on original order". The `notes` do show up, as a message in the return's Activity feed, but the feed attributes it to "Redo".
- **Evidence:** `docs/redo/openapi.yaml`, `bulk-returns-request.schema` (product fields: `productId`, `quantity`, `sku`, `barcode`, `productTitle`, `variantTitle`, `reason`, `condition`). The 3 "Ana Test" returns from `~/redo_smoke.sh` in the Crimson Hack store: $0.00, store credit, note "Filed by Beehive smoke test" in Activity.
- **What we did:** Nothing in code yet. Beehive's dashboard shows the $36 amount from its own record.
- **Pitch line:** "An agent can file a return through Redo's API, but the API can't say what it's worth or how it should be refunded, so the shop sees $0 store credit."

### Redo: a return can't say that a customer's agent filed it

- **When:** phase 02, while building the gate's handoff to Redo (read from the spec). Tested live in phase 06 (2026-11-06): the note shows in the return's Activity feed with the same gate hash that `GET /verify/bh_0001` checks (Redo return `6aee99198a41ad54330e8cfe`).
- **What we found:** The word "agent" appears nowhere in Redo's OpenAPI spec. The return import body (`bulk-returns-request.schema`) has `customer`, `notes`, `externalReferenceId` and `idempotencyKey`, but no field for who filed the return, which agent it was, or what the customer allowed. On a return, `source` holds only the customer's email and name, and return tags come only from `merchant` or `redo`. So once agent-filed returns reach Redo, the shop can't tell them apart from ones the customer filed, and Redo's review can't weigh them differently.
- **Evidence:** `docs/redo/openapi.yaml`: `bulk-returns-request.schema`, the return `source` object, the return tag `source` enum (`merchant`, `redo`)
- **What we did:** Every return Beehive imports carries the proof in Redo's free-text `notes`: "Filed by agent muse-demo for Kaylee Jensen through Beehive bh_0001, within a $50.00 cap the customer approved. Proof: gate decision hash a7e3…21aa". The hash matches the gate decision link that `GET /verify/bh_0001` checks. `externalReferenceId` stays the order id (`cvh-o-1`). Beehive's dashboard and `verify` hold the full record.
- **Pitch line:** "Redo can't tell yet that an agent filed a return. Beehive records which agent, on whose behalf and under what limit, and hands Redo a proof id it can check."

### Redo: tested against a real store, and what the docs don't tell you

- **When:** 2026-11-06/07, smoke test against a real Redo store (see `docs/redo-integration.md`)
- **What we found:**
  - Good: Redo publishes `llms.txt`, `llms-full.txt` and an OpenAPI 3.1 spec (`docs/redo/openapi.yaml`), so agents can plan an integration straight from the docs. Custom order sync makes a shop on any platform returnable.
  - Good: return import takes an `idempotencyKey`. A replay with the same key returned the same `returnId` (`redo-fixtures/return-import.json` and `return-replay.json`), so there's no double return.
  - Gap: `GET /returns/{returnId}` returns 500 for imported returns (`redo-fixtures/return-get.json`), so there's no way to read the label for an imported return.
  - Gap: `GET /stores/{storeId}/returns` doesn't list imported returns.
  - Gap: a `productId` that isn't Redo's `prd_` id gets a 400 with `"detail":"[object Object]"`, which doesn't say what's wrong.
  - Gap: a bulk import can answer 200 and still contain per-item errors in `results[].error`.
  - No risk score on returns, and no `/shipments/buy` in REST v2.2.
- **Evidence:** `docs/redo-findings.md` (bugs with steps, return ids and questions for Redo), `docs/redo-integration.md`, `redo-fixtures/*.json`, `docs/redo/openapi.yaml`
- **What we did:** Built to the tested call order. The Beehive gate blocks replays before Redo; Redo's `idempotencyKey` is the backup. The dashboard reads our own evidence store, not Redo's list. The label is best effort.
- **Pitch line:** "We smoke-tested Redo's API for real. Idempotency held up, and we found three rough edges your team can fix in an afternoon."
