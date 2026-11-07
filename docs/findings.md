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
