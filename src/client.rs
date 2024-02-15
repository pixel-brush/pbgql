use std::collections::HashMap;

use anyhow::Context;

#[derive(Debug)]
pub struct PBGql {
    client: reqwest::Client,
    url: String,
}

impl Default for PBGql {
    fn default() -> Self {
        Self {
            client: reqwest::ClientBuilder::default()
                .build()
                .expect("default reqwest client"),
            url: String::from("https://countries.trevorblades.com/graphql"),
        }
    }
}

impl PBGql {
    pub fn new<S: Into<String>>(url: S) -> Self {
        Self {
            client: reqwest::ClientBuilder::default()
                .build()
                .expect("default reqwest client"),
            url: url.into(),
        }
    }

    pub async fn send<T>(
        &self,
        query: &str,
        variables: Option<HashMap<String, serde_json::Value>>,
    ) -> anyhow::Result<T>
    where
        T: for<'a> serde::Deserialize<'a>,
    {
        let body = if let Some(variables) = variables {
            serde_json::json!({
               "query": query,
               "variables": variables,
            })
        } else {
            serde_json::json!({
               "query": query,
            })
        };

        let resp = self
            .client
            .post(&self.url)
            .json(&body)
            .send()
            .await?
            .text()
            .await?;

        let data: serde_json::Value = serde_json::from_str(&resp)?;

        match serde_json::from_str(data.get("data").context(resp)?.clone().to_string().as_str()) {
            Ok(d) => Ok(d),
            Err(e) => Err(anyhow::anyhow!(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn pbgql_should_resolve_query_from_example() -> anyhow::Result<()> {
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

    #[tokio::test]
    async fn pbgql_should_resolve_query_from_example_typed() -> anyhow::Result<()> {
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

        #[derive(serde::Deserialize)]
        struct Country {
            name: String,
        }

        #[derive(serde::Deserialize)]
        struct CountriesResponse {
            countries: Vec<Country>,
        }

        let data: CountriesResponse = client.send(query, Some(variables)).await?;
        assert_eq!("Australia", data.countries.first().unwrap().name);

        Ok(())
    }

    #[tokio::test]
    async fn pbgql_should_resolve_query() -> anyhow::Result<()> {
        let client = PBGql::new(String::from("http://localhost:8080/graphql"));

        let query = "
        query qj {
            queryJob {
                id
            }
        }
        ";

        let d: serde_json::Value = client.send(query, None).await?;
        assert!(d.get("queryJob").is_some());

        Ok(())
    }

    #[tokio::test]
    async fn pbgql_should_resolve_mutation() -> anyhow::Result<()> {
        let client = PBGql::new(String::from("http://localhost:8080/graphql"));

        let query = "
        mutation addJob($input: [AddJobInput!]!) {
            addJob(input: $input) {
                numUids
            }
        }
        ";

        let mut variables = HashMap::new();
        variables.insert(
            String::from("input"),
            serde_json::json!(vec![serde_json::json!({
                "title": "Software Engineer",
                "location": "Sydney",
                "requiredSkills": vec!["Rust", "Go"]
            })]),
        );

        let d: serde_json::Value = client.send(query, Some(variables)).await?;
        assert!(d.get("addJob").is_some());

        Ok(())
    }

    #[tokio::test]
    async fn pbgql_should_resolve_mutation_rn() -> anyhow::Result<()> {
        let client = PBGql::new(String::from("http://localhost:8080/graphql"));

        let query = "
        mutation addCandidate($input: [AddCandidateInput!]!) {
            addCandidate(input: $input) {
                numUids
            }
        }
        ";

        let mut variables = HashMap::new();

        variables.insert(
            String::from("input"),
            serde_json::json!(vec![serde_json::json!({
                "firstName": "First Name",
                "lastName": "Last Name",
            })]),
        );

        let d: serde_json::Value = client.send(query, Some(variables)).await?;
        assert!(d.get("addCandidate").is_some());

        Ok(())
    }

    #[tokio::test]
    async fn pbgql_should_show_error_in_context() -> anyhow::Result<()> {
        let client = PBGql::new(String::from("http://localhost:8080/graphql"));

        let query = "
        mutation addJob($input: [AddJobInputInvalid!]!) {
            addJob(input: $input) {
                numUids
            }
        }
        ";

        let mut variables = HashMap::new();
        variables.insert(
            String::from("input"),
            serde_json::json!(vec![serde_json::json!({
                "title": "Software Engineer",
                "location": "Sydney",
                "requiredSkills": vec!["Rust", "Go"]
            })]),
        );

        let d = client
            .send::<serde_json::Value>(query, Some(variables))
            .await;
        assert!(d.is_err());

        assert!(d.err().unwrap().to_string().contains("Variable type provided [AddJobInputInvalid!]! is incompatible with expected type [AddJobInput!]!"));

        Ok(())
    }
}
