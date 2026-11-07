# Redo API reference

`openapi.yaml` is Redo's REST spec, v2.2.1, saved from `https://developers.redo.com/api-schema/openapi.yaml` on 2026-11-06. Don't edit it; download it again if Redo updates it. Full docs index: `https://developers.redo.com/llms.txt`.

Base URL `https://api.getredo.com/v2.2`. Auth: `Authorization: Bearer $REDO_API_SECRET`. Store id: `$REDO_STORE_ID`.

## The operations Beehive uses

| Operation | Path | Scope | Used for |
|---|---|---|---|
| Custom order sync | `POST /stores/{storeId}/custom/orders` | | Sync the seeded Cache Valley orders. Same order id again = update. |
| Return create | `POST /stores/{storeId}/returns` | `returns_write` | Hand off a return that passed the gate. Send `Idempotency-Key: <request_id>`. |
| Return status | `GET /returns/{returnId}/status` | `returns_read` | Read Redo's decision (`return-status.schema`). |

## What Redo does not have (REST v2.2)

- No risk score on returns. Beehive shows Redo's `status`: `open` = approved, `needs_review` / `in_review` = a person decides, `rejected` = denied.
- No label purchase (`/shipments/buy`). Beehive shows the return's `postageLabelUrl` when Redo has one.

See `docs/findings.md` for the full finding.
