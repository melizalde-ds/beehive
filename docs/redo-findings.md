# Redo API findings: Beehive (Crimson Hacks 2026)

**Store:** `6aee7f00ff342f00a07492e4` · **API:** `https://api.getredo.com/v2.2` (v2.2.1) · **Auth:** Bearer API secret · **Date:** 2026-11-07, 01:40 to 01:58 UTC

## What works
| Step | Call | Result |
|---|---|---|
| Token check | `GET /stores/{storeId}/returns` | 200 |
| Add product | `POST /stores/{storeId}/products/bulk-upload` | 200, `prd_697G3S1625KAG` (`created`, then `updated` on re-run) |
| Sync order | `POST /stores/{storeId}/custom/orders` | 200, redoId `6aee81ba2dfdaf6b203c2d87` |
| Read order | `GET /stores/{storeId}/orders/beehive-test-1` | 200 |
| Import return | `POST /stores/{storeId}/returns/bulk` | 200, `created` |
| Replay, same `idempotencyKey` | `POST /stores/{storeId}/returns/bulk` | 200, same `returnId`, no duplicate |
| Return status | `GET /returns/{returnId}/status` | 200, `{"status":"open"}` |

## Bug 1: a 400 error with no usable detail
**Call:** `POST /stores/{storeId}/returns/bulk`

**Steps:** On a store with no products in the catalog, import a return whose `productId` doesn't exist (for example `cvh-candle-set`).

**Actual:**
```json
{"type":"com.getredo.api:external-return/invalid-request","title":"Invalid request body","detail":"[object Object]","status":400}
```
**Expected:** a readable `detail`, or a per-item error in `results[]` such as "product not found."

**Impact:** The validation error object is converted to a string without being serialized, so the real cause is lost. Once we uploaded the product with `products/bulk-upload`, the same request returned 200.

## Bug 2: a 500 when reading an imported return
**Call:** `GET /returns/{returnId}`

**Steps:**
1. Upload a product (`prd_697G3S1625KAG`).
2. Sync an order (`beehive-test-1`).
3. Import a return for it with `returns/bulk`.
4. Call `GET /returns/{returnId}`.

**Return IDs where this happens:**
- `6aee84b6aacae33642c5ca2a` (about 01:54 UTC; also retried after a 5-second wait)
- `6aee859a5934fa9229d5a325` (about 01:58 UTC)

**Actual:**
```json
{"type":"com.getredo.api:return/server-error","detail":"An unexpected error occurred while processing the return","title":"Internal server error"}
```
**Expected:** 200 with the return object, including `shipments`.

**Notes:**
- `GET /returns/{returnId}/status` works for the same IDs and returns `open`, so the return exists.
- The import body included `idempotencyKey`, `externalReferenceId`, `currency`, `customer`, `notes`, and `products[]` (`productId`, `sku`, `productTitle`, `quantity`, `reason: "damaged"`). It had no `shippingAddress` or `merchantAddress`.
- Our guess: the full serializer expects something an imported return doesn't have, such as an order link or addresses.

## Finding 3: imported returns don't appear in the list
**Call:** `GET /stores/{storeId}/returns`

After two returns were imported successfully, it still returns `{"orders":[],"returns":[]}`. We can't tell if this is on purpose (imported returns filtered out) or a bug.

## Questions for Redo
1. Is the Bug 2 500 known? Would including `shippingAddress` or `merchantAddress` in the import avoid it?
2. Should imported returns show up in `GET /stores/{storeId}/returns`?
3. For a customer-initiated return, should we use **Create Return** instead of **Import Returns**? Does Create Return run Redo's review (risk level) and create a return label?
4. Is there a way to link an imported return to a synced custom order? `externalReferenceId` doesn't seem to do that.
5. What values do `reason` and `condition` accept?
