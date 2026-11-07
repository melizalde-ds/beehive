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

### Redo: docs are agent-ready, but REST v2.2 has no risk score or label purchase

- **When:** before phase 02, 2026-11-06 evening
- **What we found:**
  - Redo publishes `https://developers.redo.com/llms.txt`, `llms-full.txt`, and an OpenAPI 3.1 spec at `https://developers.redo.com/api-schema/openapi.yaml` (v2.2.1). We could plan the integration from the spec instead of hand-writing notes.
  - The two endpoints Beehive needs are there: `POST /stores/{storeId}/custom/orders` (sync orders, so a shop on any platform becomes returnable) and `POST /stores/{storeId}/returns` (create a return, `returns_write` scope).
  - The spec mentions "risk" 0 times. A return has a `status` instead: `open` (approved, awaiting shipment), `needs_review`, `in_review`, `rejected`, and others.
  - There is no `POST /stores/{storeId}/shipments/buy` in REST v2.2. Return create accepts an optional `label` with `postageLabelUrl`. Unverified lead: the v3 GraphQL changelog mentions a `purchaseCarrierShipment` mutation.
- **Evidence:** `openapi.yaml` paths list (`/stores/{storeId}/custom/orders`, `/stores/{storeId}/returns`, `/returns/{returnId}/status`), and the `return-status.schema` enum in it.
- **What we did:** Aligned our contract with Redo: `RedoResult` now carries Redo's own return `status` (no risk score), and the label comes from the return's `postageLabelUrl`. Saved the spec as `docs/redo/openapi.yaml`. Bonus: `POST /stores/{storeId}/returns` takes an `Idempotency-Key` header, so the gate's `request_id` goes straight through and Redo also refuses a double refund.
- **Pitch line:** "Redo's docs ship llms.txt and OpenAPI, so an agent-built plugin could read them directly. That's the same idea Beehive brings to shoppers' agents."
