//! Shapes shared with the TypeScript clients. Generated at build time from
//! `schema/beehive.schema.json`; change the schema, never these types.

typify::import_types!(schema = "../schema/beehive.schema.json");

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn parse_as(shape: &str, json: &str) -> Result<(), serde_json::Error> {
        match shape {
            "ApiError" => serde_json::from_str::<ApiError>(json).map(drop),
            "ConsentDecision" => serde_json::from_str::<ConsentDecision>(json).map(drop),
            "Evidence" => serde_json::from_str::<Evidence>(json).map(drop),
            "GateRequest" => serde_json::from_str::<GateRequest>(json).map(drop),
            "GateResponse" => serde_json::from_str::<GateResponse>(json).map(drop),
            "OrderList" => serde_json::from_str::<OrderList>(json).map(drop),
            "PoppyDiscovery" => serde_json::from_str::<PoppyDiscovery>(json).map(drop),
            "ReturnList" => serde_json::from_str::<ReturnList>(json).map(drop),
            "SessionRequest" => serde_json::from_str::<SessionRequest>(json).map(drop),
            "SessionResponse" => serde_json::from_str::<SessionResponse>(json).map(drop),
            "TamperRequest" => serde_json::from_str::<TamperRequest>(json).map(drop),
            "VerifyResult" => serde_json::from_str::<VerifyResult>(json).map(drop),
            other => panic!("schema/examples/{other}.json has no matching type in parse_as"),
        }
    }

    #[test]
    fn every_example_matches_the_schema() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../schema/examples");
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            let shape = name.split('.').next().unwrap();
            let json = std::fs::read_to_string(&path).unwrap();
            parse_as(shape, &json).unwrap_or_else(|error| panic!("{name}: {error}"));
        }
    }

    #[test]
    fn seed_matches_the_schema_and_its_orders_add_up() {
        let seed: Seed =
            serde_json::from_str(include_str!("../../seed/cache-valley-honey.json")).unwrap();
        let customers: HashSet<_> = seed.customers.iter().map(|c| c.id.as_str()).collect();
        let products: HashSet<_> = seed.products.iter().map(|p| p.id.as_str()).collect();
        for order in &seed.orders {
            assert!(
                customers.contains(order.customer_id.as_str()),
                "{}: unknown customer",
                order.id
            );
            let mut sum = 0.0;
            for item in order.items.iter() {
                assert!(
                    products.contains(item.product_id.as_str()),
                    "{}: unknown product",
                    order.id
                );
                sum += item.unit_price * item.qty.get() as f64;
            }
            assert!(
                (sum - order.total).abs() < 0.005,
                "{}: total {} != items {sum}",
                order.id,
                order.total
            );
        }
        assert!(seed.agents.iter().any(|a| a.agent_id == "muse-demo"));
    }
}
