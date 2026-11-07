//! Redo client, switched by `REDO_MODE`: `mock` (default) answers like `redo-fixtures/`,
//! `real` calls the Redo API. Call order and quirks: `docs/redo-integration.md`.

use std::{collections::HashMap, fmt, time::Duration};

use chrono::{DateTime, SecondsFormat};
use reqwest::Method;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::types::{GateRequest, Order, RedoResult, RedoStatus, Seed};

pub enum Redo {
    Mock,
    Real(Real),
}

pub struct Real {
    http: reqwest::Client,
    base: String,
    store_id: String,
    secret: String,
    /// Seed product id → Redo `prd_` id. Returns must use Redo's id, not ours.
    products: HashMap<String, String>,
}

#[derive(Debug)]
pub struct RedoError {
    /// None when Redo never answered (timeout, network).
    status: Option<u16>,
    message: String,
}

impl RedoError {
    fn new(status: Option<u16>, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }
}

impl fmt::Display for RedoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.status {
            Some(status) => write!(f, "Redo HTTP {status}: {}", self.message),
            None => write!(f, "Redo unreachable: {}", self.message),
        }
    }
}

impl Redo {
    /// Real mode seeds the Redo store first. If that fails, the app falls back to the mock
    /// so the demo stays runnable; the evidence's `redo_mode` shows which one answered.
    pub async fn from_env(seed: &Seed) -> Redo {
        if std::env::var("REDO_MODE").as_deref() != Ok("real") {
            tracing::info!("Redo: mock (set REDO_MODE=real to call the Redo API)");
            return Redo::Mock;
        }
        match Real::connect(seed).await {
            Ok(real) => {
                tracing::info!(
                    store_id = real.store_id,
                    "Redo: real, store seeded with {} products and {} orders",
                    real.products.len(),
                    seed.orders.len()
                );
                Redo::Real(real)
            }
            Err(error) => {
                tracing::warn!(%error, "Redo real mode couldn't start, using the mock");
                Redo::Mock
            }
        }
    }

    pub fn mode(&self) -> &'static str {
        match self {
            Redo::Mock => "mock",
            Redo::Real(_) => "real",
        }
    }

    /// `request_id` is Redo's `idempotencyKey`, so a replay gets the same return id.
    pub async fn import_return(
        &self,
        seed: &Seed,
        req: &GateRequest,
    ) -> Result<RedoResult, RedoError> {
        let (return_id, status, label_url) = match self {
            Redo::Mock => {
                // 24 hex chars, like a Redo id; same key, same id.
                let hash = hex::encode(Sha256::digest(req.request_id.as_bytes()));
                (hash[..24].to_string(), RedoStatus::Open, None)
            }
            Redo::Real(real) => real.import_return(seed, req).await?,
        };
        Ok(RedoResult {
            return_id: return_id
                .try_into()
                .map_err(|e| RedoError::new(None, format!("empty return id: {e}")))?,
            status,
            label_url,
        })
    }
}

impl Real {
    async fn connect(seed: &Seed) -> Result<Real, RedoError> {
        let env = |key: &str| {
            std::env::var(key)
                .ok()
                .filter(|v| !v.is_empty())
                .ok_or_else(|| RedoError::new(None, format!("{key} isn't set")))
        };
        let mut real = Real {
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .map_err(|e| RedoError::new(None, e.to_string()))?,
            base: std::env::var("REDO_BASE")
                .unwrap_or_else(|_| "https://api.getredo.com/v2.2".to_string()),
            store_id: env("REDO_STORE_ID")?,
            secret: env("REDO_API_SECRET")?,
            products: HashMap::new(),
        };
        real.products = real.upload_products(seed).await?;
        for order in &seed.orders {
            real.sync_order(seed, order).await?;
        }
        Ok(real)
    }

    async fn call(
        &self,
        method: Method,
        path: &str,
        body: Option<&Value>,
    ) -> Result<Value, RedoError> {
        let mut request = self
            .http
            .request(method, format!("{}{path}", self.base))
            .bearer_auth(&self.secret)
            .header(reqwest::header::ACCEPT, "application/json");
        if let Some(body) = body {
            request = request.json(body);
        }
        let response = request
            .send()
            .await
            .map_err(|e| RedoError::new(None, e.without_url().to_string()))?;
        let status = response.status();
        let body: Value = response.json().await.unwrap_or(Value::Null);
        if status.is_success() {
            return Ok(body);
        }
        let text = |key: &str| body[key].as_str().unwrap_or_default().to_string();
        Err(RedoError::new(
            Some(status.as_u16()),
            format!("{} {}", text("title"), text("detail")).trim(),
        ))
    }

    /// All products in one upsert. Returns seed product id → `prd_` id.
    async fn upload_products(&self, seed: &Seed) -> Result<HashMap<String, String>, RedoError> {
        let body = json!({ "products": seed.products.iter().map(|p| json!({
            "title": p.name,
            "external_id": external_id(&p.id),
            "vendor": seed.shop.name,
            "variants": [{ "sku": sku(&p.id), "price": cents(p.price), "currency": "USD" }],
        })).collect::<Vec<_>>() });
        let path = format!("/stores/{}/products/bulk-upload", self.store_id);
        let mut tries = 0;
        let response = loop {
            tries += 1;
            match self.call(Method::POST, &path, Some(&body)).await {
                // 409: another upload is still running.
                Err(e) if e.status == Some(409) && tries < 4 => {
                    tokio::time::sleep(Duration::from_secs(3)).await;
                }
                other => break other?,
            }
        };
        let by_sku = parse_products(&response)?;
        seed.products
            .iter()
            .map(|p| {
                by_sku
                    .get(&sku(&p.id))
                    .map(|prd| (p.id.clone(), prd.clone()))
                    .ok_or_else(|| RedoError::new(None, format!("no Redo id for {}", sku(&p.id))))
            })
            .collect()
    }

    async fn sync_order(&self, seed: &Seed, order: &Order) -> Result<(), RedoError> {
        let customer = seed
            .customers
            .iter()
            .find(|c| c.id == order.customer_id)
            .ok_or_else(|| RedoError::new(None, format!("no customer for {}", order.id)))?;
        let (first, last) = names(&customer.name);
        let (city, state) = customer
            .city
            .split_once(", ")
            .unwrap_or((&customer.city, "UT"));
        let at = utc(&order.placed_at);
        let id = order_id(&order.id);
        let line_items = order.items.iter().enumerate().map(|(i, item)| {
            let money =
                json!({ "amount": format!("{:.2}", item.unit_price), "currencyCode": "USD" });
            json!({
                "id": format!("{id}-{}", i + 1),
                "title": product_name(seed, &item.product_id),
                "productId": external_id(&item.product_id),
                "sku": sku(&item.product_id),
                "quantity": item.qty,
                "priceSet": { "shopMoney": money, "presentmentMoney": money },
            })
        });
        let body = json!({
            "id": id, "type": "order", "createdAt": at, "updatedAt": at,
            "presentmentCurrency": "USD", "shopCurrency": "USD",
            "customer": { "email": customer.email, "firstName": first, "lastName": last },
            "lineItems": line_items.collect::<Vec<_>>(),
            "orderDiscounts": [], "shippingLines": [], "fulfillments": [],
            "shippingAddress": { "address1": "100 N Main St", "city": city, "provinceCode": state,
                                 "zip": zip(city), "countryCode": "US" },
        });
        let path = format!("/stores/{}/custom/orders", self.store_id);
        self.call(Method::POST, &path, Some(&body)).await.map(drop)
    }

    async fn import_return(
        &self,
        seed: &Seed,
        req: &GateRequest,
    ) -> Result<(String, RedoStatus, Option<String>), RedoError> {
        let missing = |what: &str| RedoError::new(None, format!("{what} not in the seed"));
        let order = seed
            .orders
            .iter()
            .find(|o| o.id == req.order_id.as_str())
            .ok_or_else(|| missing("order"))?;
        let customer = seed
            .customers
            .iter()
            .find(|c| c.id == order.customer_id)
            .ok_or_else(|| missing("customer"))?;
        let (first, last) = names(&customer.name);
        let products = order
            .items
            .iter()
            .map(|item| {
                let prd = self
                    .products
                    .get(&item.product_id)
                    .ok_or_else(|| missing("product"))?;
                Ok(json!({
                    "productId": prd, "sku": sku(&item.product_id),
                    "productTitle": product_name(seed, &item.product_id),
                    "quantity": item.qty, "reason": req.reason,
                }))
            })
            .collect::<Result<Vec<_>, RedoError>>()?;
        let body = json!({ "returns": [{
            "idempotencyKey": req.request_id.as_str(),
            "externalReferenceId": order_id(&order.id),
            "currency": "USD",
            "customer": { "firstName": first, "lastName": last, "email": customer.email },
            "notes": format!("Filed by agent {} for {}", req.agent_id.as_str(), customer.name),
            "products": products,
        }]});
        let path = format!("/stores/{}/returns/bulk", self.store_id);
        let return_id = parse_import(&self.call(Method::POST, &path, Some(&body)).await?)?;

        let status = self
            .call(Method::GET, &format!("/returns/{return_id}/status"), None)
            .await?;
        let status: RedoStatus = serde_json::from_value(status["status"].clone())
            .map_err(|e| RedoError::new(None, format!("unknown return status: {e}")))?;

        // Best effort: Redo answers 500 here for imported returns (docs/redo-findings.md, bug 2).
        let label_url = match self
            .call(Method::GET, &format!("/returns/{return_id}"), None)
            .await
        {
            Ok(body) => body
                .pointer("/return/shipments/0/postageLabel")
                .and_then(Value::as_str)
                .map(String::from),
            Err(error) => {
                tracing::warn!(%error, return_id, "no label from Redo");
                None
            }
        };
        Ok((return_id, status, label_url))
    }
}

/// `products/bulk-upload` response → sku → `prd_` id.
fn parse_products(body: &Value) -> Result<HashMap<String, String>, RedoError> {
    let mut by_sku = HashMap::new();
    for result in body["results"].as_array().into_iter().flatten() {
        if result["status"] == "error" {
            return Err(RedoError::new(
                None,
                format!("product rejected: {}", result["error"]),
            ));
        }
        for p in result["products"].as_array().into_iter().flatten() {
            if let (Some(sku), Some(id)) = (p["sku"].as_str(), p["id"].as_str()) {
                by_sku.insert(sku.to_string(), id.to_string());
            }
        }
    }
    Ok(by_sku)
}

/// `returns/bulk` answers 200 even when the item failed, so read `results[0].status`.
fn parse_import(body: &Value) -> Result<String, RedoError> {
    let result = &body["results"][0];
    match (result["status"].as_str(), result["returnId"].as_str()) {
        (Some("created"), Some(id)) => Ok(id.to_string()),
        _ => Err(RedoError::new(
            None,
            format!("return not created: {}", result["error"]),
        )),
    }
}

// Redo needs a few fields the seed doesn't carry, so they're derived here.
fn sku(product_id: &str) -> String {
    format!("CVH-{}", product_id.trim_start_matches("p_").to_uppercase())
}

fn external_id(product_id: &str) -> String {
    format!(
        "cvh-{}",
        product_id.trim_start_matches("p_").replace('_', "-")
    )
}

/// Redo rejects underscores in line item ids, which are built from this.
fn order_id(seed_order_id: &str) -> String {
    format!("cvh-{}", seed_order_id.replace('_', "-"))
}

fn names(full: &str) -> (&str, &str) {
    full.split_once(' ').unwrap_or((full, ""))
}

fn zip(city: &str) -> &'static str {
    match city {
        "Provo" => "84601",
        "Brigham City" => "84302",
        _ => "84321", // Logan
    }
}

fn product_name<'a>(seed: &'a Seed, product_id: &'a str) -> &'a str {
    seed.products
        .iter()
        .find(|p| p.id == product_id)
        .map_or(product_id, |p| p.name.as_str())
}

/// Redo product prices are integer cents.
fn cents(usd: f64) -> i64 {
    (usd * 100.0).round() as i64
}

fn utc(rfc3339: &str) -> String {
    DateTime::parse_from_rfc3339(rfc3339).map_or_else(
        |_| rfc3339.to_string(),
        |at| at.to_utc().to_rfc3339_opts(SecondsFormat::Secs, true),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> Value {
        let path = format!(
            "{}/../redo-fixtures/{name}.json",
            env!("CARGO_MANIFEST_DIR")
        );
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    #[test]
    fn parsers_read_the_real_redo_responses() {
        let products = parse_products(&fixture("product-upload")).unwrap();
        assert_eq!(products["CVH-CANDLE-SET"], "prd_697G3S1625KAG");
        assert_eq!(
            parse_import(&fixture("return-import")).unwrap(),
            "6aee859a5934fa9229d5a325"
        );
        let status: RedoStatus =
            serde_json::from_value(fixture("return-status")["status"].clone()).unwrap();
        assert_eq!(status, RedoStatus::Open);
        let failed =
            json!({ "results": [{ "status": "error", "index": 0, "error": "product not found" }] });
        assert!(parse_import(&failed).is_err());
    }
}
