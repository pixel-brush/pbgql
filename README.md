# PBGql

Super minimal no frills good for moving quick and then ditch for a more featured one later graphql client for Rust

# Examples
```rust
use std::collections::HashMap;
use pbgql::client::PBGql;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
        let client = PBGql::new(String::from("https://countries.trevorblades.com/graphql"));

        let query = "
        query getCountry($name: String!) {
            countries(filter: {name: {eq: $name}}) {
                name
            }
        }
        ";

        let mut variables = HashMap::new();
        variables.insert(
            String::from("name"),
            serde_json::Value::String(String::from("Australia")),
        );

        let data: serde_json::Value = client.send(query, Some(variables)).await?;
        let country = data.get("countries").unwrap().as_array().unwrap();
        assert_eq!(
            "Australia",
            country
                .first()
                .unwrap()
                .get("name")
                .unwrap()
                .as_str()
                .unwrap()
        );

        Ok(())
}
```
