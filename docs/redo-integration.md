# Redo integration spec (for Claude Code)

The call order and shapes below were tested against a real Redo store on 2026-11-07. Follow them exactly. Where this file and `brief.md` disagree about Redo, this file wins.

Real responses are saved in `redo-fixtures/*.json`. The mock must return those exact shapes.

## Config
```
REDO_MODE=mock|real          # default mock
REDO_BASE=https://api.getredo.com/v2.2
REDO_API_SECRET=...          # Bearer token, never log it
REDO_STORE_ID=6aee7f00ff342f00a07492e4
```
Every call sends `Authorization: Bearer $REDO_API_SECRET`, plus `Content-Type: application/json` when there's a body.

## Call order

### A. Seed (once at startup, or `cargo run --bin seed`)
Run for each seeded product and order. Both calls are upserts, so re-running is safe.

1. **Upload product:** `POST /stores/{storeId}/products/bulk-upload`
   ```json
   {"products":[{"title":"Beeswax Candle Set","external_id":"cvh-candle-set","vendor":"Cache Valley Honey Co.",
     "variants":[{"sku":"CVH-CANDLE-SET","price":3600,"currency":"USD"}]}]}
   ```
   - `price` is an **integer in cents**.
   - Response: `results[].products[] = {id:"prd_...", sku}`. Store a **sku → prd_ id** map. You need it for returns.
   - `status` comes back `created` on the first run and `updated` after that. On 409 (an upload is already running), sleep 3s and retry.
   - Up to 1000 products per call, so send all 4 products at once.

2. **Sync order:** `POST /stores/{storeId}/custom/orders`, one order per call
   - Required: `id, type:"order", createdAt, updatedAt, presentmentCurrency, shopCurrency, customer{email,firstName,lastName}, lineItems[], orderDiscounts:[], shippingLines:[], fulfillments:[], shippingAddress{address1,city,provinceCode,zip,countryCode}`
   - Each line item: `{id, title, productId (our external id), sku, quantity, priceSet:{shopMoney:{amount:"36.00",currencyCode:"USD"}, presentmentMoney:{...same}}}`
   - **Amounts are strings** here. Line item `id` can't contain underscores.
   - Response: `{message, orderId, redoId}`

   Optional check: `GET /stores/{storeId}/orders/{orderId}`. **Prices come back as numbers** (`"price": 36`).

### B. Per agent return (only after every gate check passes)
3. **Import return:** `POST /stores/{storeId}/returns/bulk`
   ```json
   {"returns":[{
     "idempotencyKey":"<gate request_id>",
     "externalReferenceId":"<order_id>",
     "currency":"USD",
     "customer":{"firstName":"Ana","lastName":"Test","email":"ana.test@example.com"},
     "notes":"Filed by agent <agent_id> for <customer>",
     "products":[{"productId":"<prd_ id from sku map>","sku":"CVH-CANDLE-SET","productTitle":"Beeswax Candle Set","quantity":1,"reason":"damaged"}]}]}
   ```
   - `productId` **must be the Redo `prd_` id**, not our external id. If it isn't, Redo returns 400 `"detail":"[object Object]"`.
   - **Use the gate `request_id` as `idempotencyKey`.** We tested this: Redo returns the same `returnId` on a replay. Our gate still blocks replays first; this is the backup.
   - **A 200 can still contain failures.** Check `results[0].status`: `created` → `results[0].returnId`; `error` → `results[0].error`.

4. **Read status:** `GET /returns/{returnId}/status` → `{"status":"open"}`

   Map it to the brief's `redo.status`:
   | Redo | Beehive |
   |---|---|
   | `open` | `approved` |
   | `needs_review`, `in_review`, `pre_shipment` | `manual_review` |
   | `rejected` | `denied` |
   | `in_transit`, `delivered`, `complete` | `approved` (show the raw value too) |
   | `flagged` | `manual_review` |

### C. Not available from real Redo, so the mock supplies these
- **`risk` (LOW/MEDIUM/HIGH):** no Redo endpoint we can call returns it. In real mode, set `risk: null`. In mock mode, derive it: `damaged`/`wrong_item` ≤ $50 → LOW, other reasons ≤ $50 → MEDIUM, > $100 → HIGH.
- **`label_url`:** comes from `GET /returns/{id}` → `shipments[].postageLabel`, but that endpoint returns 500 on imported returns (see `redo-findings.md`). In real mode, try it once and set `label_url: null` on error. Don't fail the request over it. The mock returns a fake label URL when the status is approved.

## Do NOT call
- `POST /stores/{storeId}/shipments/buy`: it doesn't exist in the current API (labels come on the return). Remove it from the mock.
- `GET /returns/{returnId}` as a required step: it returns 500 for imported returns.
- `GET /stores/{storeId}/returns` for the dashboard: it doesn't list imported returns. The dashboard reads **our** evidence store instead.

## Rust shape
```rust
#[async_trait]
pub trait RedoClient: Send + Sync {
    /// Upsert products, returns sku -> Redo product id (prd_...)
    async fn upsert_products(&self, products: &[ProductSeed]) -> Result<HashMap<String, String>, RedoError>;
    /// Returns Redo's redoId
    async fn sync_order(&self, order: &OrderSeed) -> Result<String, RedoError>;
    /// idempotency_key = gate request_id. Returns Redo returnId.
    async fn import_return(&self, req: &ReturnRequest, idempotency_key: &str) -> Result<String, RedoError>;
    async fn return_status(&self, return_id: &str) -> Result<RedoStatus, RedoError>;
    /// Best effort. Ok(None) on Redo 500.
    async fn label_url(&self, return_id: &str) -> Result<Option<String>, RedoError>;
}
// Implementations: MockRedo (loads redo-fixtures/), HttpRedo (reqwest).
```
`RedoError` should hold the HTTP status, Redo's `type`, `title` and `detail`, and the per-item `results[].error` for bulk calls.

## Gate handoff flow
```
agent POST /gate/returns
  -> gate checks (all must pass, each one appended to the hash chain)
  -> import_return(req, request_id)     chain: "redo.request" + "redo.return_id"
  -> return_status(return_id)           chain: "redo.status"
  -> label_url(return_id) (best effort) chain: "redo.label" (or null)
  -> response { gate:"pass", redo:{return_id, risk, status, label_url}, chain_head }
```
The $120 block and the replay block never reach Redo. Make sure the evidence chain shows a `gate.block` link with no `redo.*` links after it.

## Tests to write
- Mock: `import_return` with the same key twice → same `return_id`
- Mock: an unknown `productId` → `RedoError` with status 400
- Status mapping table (one test case per row)
- `label_url` returns `Ok(None)` when the mock is set to return 500
- Real mode (only when `REDO_API_SECRET` is set; skip otherwise): seed, then import, then status = `open`. This matches `redo_smoke.sh`.
