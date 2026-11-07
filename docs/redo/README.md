# Redo API reference

**Source of truth for how Beehive calls Redo: `docs/redo-integration.md`.** It was tested against a real Redo store, and it wins over this file, the spec, and the brief. Real responses are in `redo-fixtures/`.

`openapi.yaml` is Redo's REST spec, v2.2.1, saved from `https://developers.redo.com/api-schema/openapi.yaml` on 2026-11-06. Don't edit it; download it again if Redo updates it. Full docs index: `https://developers.redo.com/llms.txt`.

Base URL `https://api.getredo.com/v2.2`. Auth: `Authorization: Bearer $REDO_API_SECRET`. Store id: `$REDO_STORE_ID`.

## The calls Beehive makes (from `redo-integration.md`)

| Call | Path | When |
|---|---|---|
| Product upload | `POST /stores/{storeId}/products/bulk-upload` | Seed. Gives the sku → `prd_` id map. |
| Custom order sync | `POST /stores/{storeId}/custom/orders` | Seed, one order per call. |
| Return import | `POST /stores/{storeId}/returns/bulk` | After the gate passes. `idempotencyKey` = gate `request_id`. |
| Return status | `GET /returns/{returnId}/status` | Right after import. |
| Return (label) | `GET /returns/{returnId}` | Best effort only: returns 500 on imported returns. |

Don't call `/shipments/buy` (it doesn't exist) or `GET /stores/{storeId}/returns` for the dashboard (it doesn't list imported returns).
