use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::Serialize;

const MAX_LATENCIES: usize = 512;

#[derive(Clone, Debug)]
pub struct AppState {
    started_at: Instant,
    counts: HashMap<String, u64>,
    total_seen: u64,
    tick_count: u64,
    error_count: u64,
    last_seed_timestamp_ms: Option<u128>,
    witness_chain_length: Option<u64>,
    total_vectors: Option<u64>,
    epoch: Option<u64>,
    paired: Option<bool>,
    last_ingest_id: Option<String>,
    latencies: VecDeque<LatencySample>,
}

#[derive(Clone, Debug, Serialize)]
pub struct LatencySample {
    pub operation: String,
    pub millis: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct StateSnapshot {
    pub service: &'static str,
    pub warning: &'static str,
    pub uptime_secs: u64,
    pub tick_count: u64,
    pub total_seen: u64,
    pub counts: HashMap<String, u64>,
    pub error_count: u64,
    pub last_seed_timestamp_ms: Option<u128>,
    pub witness_chain_length: Option<u64>,
    pub total_vectors: Option<u64>,
    pub epoch: Option<u64>,
    pub paired: Option<bool>,
    pub last_ingest_id: Option<String>,
    pub generated_at_ms: u128,
}

#[derive(Clone, Debug, Serialize)]
pub struct MetricsSnapshot {
    pub service: &'static str,
    pub tick_count: u64,
    pub total_seen: u64,
    pub error_count: u64,
    pub witness_chain_length: Option<u64>,
    pub total_vectors: Option<u64>,
    pub epoch: Option<u64>,
    pub paired: Option<bool>,
    pub latency_samples: Vec<LatencySample>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            started_at: Instant::now(),
            counts: HashMap::new(),
            total_seen: 0,
            tick_count: 0,
            error_count: 0,
            last_seed_timestamp_ms: None,
            witness_chain_length: None,
            total_vectors: None,
            epoch: None,
            paired: None,
            last_ingest_id: None,
            latencies: VecDeque::new(),
        }
    }

    pub fn record_tick(&mut self) {
        self.tick_count += 1;
    }

    pub fn record_vector(&mut self, vector: &[f64]) -> String {
        let context = context_for(vector.first().copied().unwrap_or(0.0));
        *self.counts.entry(context.clone()).or_insert(0) += 1;
        self.total_seen += 1;
        context
    }

    pub fn record_error(&mut self) {
        self.error_count += 1;
    }

    pub fn record_status(
        &mut self,
        witness_chain_length: Option<u64>,
        total_vectors: Option<u64>,
        epoch: Option<u64>,
        paired: Option<bool>,
    ) {
        self.last_seed_timestamp_ms = Some(now_ms());
        self.witness_chain_length = witness_chain_length;
        self.total_vectors = total_vectors;
        self.epoch = epoch;
        self.paired = paired;
    }

    pub fn record_ingest_id(&mut self, id: Option<String>) {
        if let Some(id) = id {
            self.last_ingest_id = Some(id);
        }
    }

    pub fn record_latency(&mut self, operation: &str, duration: Duration) {
        if self.latencies.len() == MAX_LATENCIES {
            self.latencies.pop_front();
        }
        self.latencies.push_back(LatencySample {
            operation: operation.to_string(),
            millis: duration.as_secs_f64() * 1000.0,
        });
    }

    pub fn state_snapshot(&self) -> StateSnapshot {
        StateSnapshot {
            service: "microccf",
            warning: "Spike build, NOT v1.0",
            uptime_secs: self.started_at.elapsed().as_secs(),
            tick_count: self.tick_count,
            total_seen: self.total_seen,
            counts: self.counts.clone(),
            error_count: self.error_count,
            last_seed_timestamp_ms: self.last_seed_timestamp_ms,
            witness_chain_length: self.witness_chain_length,
            total_vectors: self.total_vectors,
            epoch: self.epoch,
            paired: self.paired,
            last_ingest_id: self.last_ingest_id.clone(),
            generated_at_ms: now_ms(),
        }
    }

    pub fn metrics_snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            service: "microccf",
            tick_count: self.tick_count,
            total_seen: self.total_seen,
            error_count: self.error_count,
            witness_chain_length: self.witness_chain_length,
            total_vectors: self.total_vectors,
            epoch: self.epoch,
            paired: self.paired,
            latency_samples: self.latencies.iter().cloned().collect(),
        }
    }
}

pub fn context_for(value: f64) -> String {
    if value < 0.25 {
        "ctx_a"
    } else if value < 0.50 {
        "ctx_b"
    } else if value < 0.75 {
        "ctx_c"
    } else {
        "ctx_d"
    }
    .to_string()
}

pub fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

#[cfg(test)]
mod tests {
    use super::{context_for, AppState};
    use std::time::Duration;

    #[test]
    fn buckets_vectors_and_counts_contexts() {
        let mut state = AppState::new();

        assert_eq!(state.record_vector(&[0.00]), "ctx_a");
        assert_eq!(state.record_vector(&[0.49]), "ctx_b");
        assert_eq!(state.record_vector(&[0.50]), "ctx_c");
        assert_eq!(state.record_vector(&[0.99]), "ctx_d");

        let snapshot = state.state_snapshot();
        assert_eq!(snapshot.total_seen, 4);
        assert_eq!(snapshot.counts.get("ctx_a"), Some(&1));
        assert_eq!(snapshot.counts.get("ctx_b"), Some(&1));
        assert_eq!(snapshot.counts.get("ctx_c"), Some(&1));
        assert_eq!(snapshot.counts.get("ctx_d"), Some(&1));
    }

    #[test]
    fn records_seed_status_and_latency_without_silent_skip() {
        let mut state = AppState::new();
        state.record_tick();
        state.record_status(Some(12), Some(34), Some(2), Some(true));
        state.record_ingest_id(Some("witness-1".to_string()));
        state.record_latency("query", Duration::from_millis(7));

        let snapshot = state.state_snapshot();
        assert_eq!(snapshot.tick_count, 1);
        assert_eq!(snapshot.witness_chain_length, Some(12));
        assert_eq!(snapshot.total_vectors, Some(34));
        assert_eq!(snapshot.epoch, Some(2));
        assert_eq!(snapshot.paired, Some(true));
        assert_eq!(snapshot.last_ingest_id.as_deref(), Some("witness-1"));

        let metrics = state.metrics_snapshot();
        assert_eq!(metrics.latency_samples.len(), 1);
        assert_eq!(metrics.latency_samples[0].operation, "query");
        assert!(metrics.latency_samples[0].millis >= 7.0);
    }

    #[test]
    fn bucket_edges_are_stable() {
        assert_eq!(context_for(0.2499), "ctx_a");
        assert_eq!(context_for(0.25), "ctx_b");
        assert_eq!(context_for(0.50), "ctx_c");
        assert_eq!(context_for(0.75), "ctx_d");
    }
}
