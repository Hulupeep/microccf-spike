use std::time::Instant;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::config::Config;
use crate::error::Result;

#[derive(Clone)]
pub struct SeedClient {
    http: Client,
    endpoint: String,
    token: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SeedStatus {
    pub witness_chain_length: Option<u64>,
    pub total_vectors: Option<u64>,
    pub epoch: Option<u64>,
    pub paired: Option<bool>,
}

#[derive(Clone, Debug)]
pub struct Timed<T> {
    pub value: T,
    pub millis: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct WitnessEvent {
    pub tick: u64,
    pub context: String,
    pub count: u64,
}

impl SeedClient {
    pub fn new(config: &Config, token: String) -> Result<Self> {
        let http = Client::builder()
            .timeout(config.http_timeout())
            .danger_accept_invalid_certs(config.seed.allow_invalid_certs)
            .build()?;
        Ok(Self {
            http,
            endpoint: config.seed.endpoint.trim_end_matches('/').to_string(),
            token,
        })
    }

    pub async fn status(&self) -> Result<Timed<SeedStatus>> {
        let start = Instant::now();
        let value = self
            .http
            .get(format!("{}/api/v1/status", self.endpoint))
            .send()
            .await?
            .error_for_status()?
            .json::<SeedStatus>()
            .await?;
        Ok(Timed {
            value,
            millis: start.elapsed().as_secs_f64() * 1000.0,
        })
    }

    pub async fn query(&self, vector: &[f64; 8]) -> Result<Timed<Value>> {
        let start = Instant::now();
        let value = self
            .http
            .post(format!("{}/api/v1/store/query", self.endpoint))
            .bearer_auth(&self.token)
            .json(&json!({
                "vector": vector,
                "k": 1,
                "limit": 1
            }))
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;
        Ok(Timed {
            value,
            millis: start.elapsed().as_secs_f64() * 1000.0,
        })
    }

    pub async fn ingest_witness(&self, event: &WitnessEvent) -> Result<Timed<Value>> {
        let vector = vector_for(event.tick);
        let id = witness_vector_id(event.tick);
        let start = Instant::now();
        let ingest = self
            .http
            .post(format!("{}/api/v1/store/ingest", self.endpoint))
            .bearer_auth(&self.token)
            .json(&json!({
                "vectors": [[id, vector]]
            }))
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;
        let metadata = self
            .http
            .put(format!(
                "{}/api/v1/store/vectors/{id}/metadata",
                self.endpoint
            ))
            .bearer_auth(&self.token)
            .json(&json!({
                "metadata": [
                    {"field_id": 0, "value": {"String": "microccf"}},
                    {"field_id": 1, "value": {"String": "witness-test"}},
                    {"field_id": 2, "value": {"String": event.context}},
                    {"field_id": 3, "value": {"U64": event.tick}},
                    {"field_id": 4, "value": {"U64": event.count}}
                ]
            }))
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;
        Ok(Timed {
            value: json!({
                "id": id,
                "ingest": ingest,
                "metadata": metadata
            }),
            millis: start.elapsed().as_secs_f64() * 1000.0,
        })
    }
}

pub fn witness_vector_id(tick: u64) -> u64 {
    9_000_000_000_000u64 + tick
}

pub fn vector_for(tick: u64) -> [f64; 8] {
    let first = ((tick % 100) as f64) / 100.0;
    [first, 0.125, 0.25, 0.375, 0.5, 0.625, 0.75, 0.875]
}

pub fn extract_id(value: &Value) -> Option<String> {
    for key in ["id", "vector_id", "uuid"] {
        if let Some(id) = value.get(key).and_then(id_value_to_string) {
            return Some(id);
        }
    }
    value
        .pointer("/vector/id")
        .or_else(|| value.pointer("/metadata/id"))
        .and_then(id_value_to_string)
}

fn id_value_to_string(value: &Value) -> Option<String> {
    if let Some(id) = value.as_str() {
        return Some(id.to_string());
    }
    value.as_u64().map(|id| id.to_string())
}

#[cfg(test)]
mod tests {
    use super::{extract_id, vector_for, witness_vector_id};
    use serde_json::json;

    #[test]
    fn vector_first_dimension_is_tick_bucket_source() {
        assert_eq!(vector_for(0)[0], 0.0);
        assert_eq!(vector_for(42)[0], 0.42);
        assert_eq!(vector_for(142)[0], 0.42);
        assert_eq!(vector_for(99)[7], 0.875);
    }

    #[test]
    fn witness_ids_are_stable_and_nonzero() {
        assert_eq!(witness_vector_id(0), 9_000_000_000_000);
        assert_eq!(witness_vector_id(42), 9_000_000_000_042);
    }

    #[test]
    fn extracts_common_seed_ids() {
        assert_eq!(extract_id(&json!({"id": "a"})).as_deref(), Some("a"));
        assert_eq!(extract_id(&json!({"vector_id": "b"})).as_deref(), Some("b"));
        assert_eq!(
            extract_id(&json!({"vector": {"id": "c"}})).as_deref(),
            Some("c")
        );
        assert_eq!(extract_id(&json!({"id": 42})).as_deref(), Some("42"));
        assert_eq!(extract_id(&json!({"ok": true})), None);
    }
}
