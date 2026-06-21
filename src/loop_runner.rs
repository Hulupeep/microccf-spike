use std::sync::Arc;

use tokio::sync::RwLock;
use tokio::time;
use tracing::{debug, warn};

use crate::config::Config;
use crate::seed_client::{extract_id, vector_for, SeedClient, WitnessEvent};
use crate::state::AppState;

pub async fn run_loop(config: Config, seed: SeedClient, state: Arc<RwLock<AppState>>) {
    let mut interval = time::interval(config.poll_interval());
    loop {
        interval.tick().await;
        if let Err(error) = tick(&seed, &state).await {
            state.write().await.record_error();
            warn!("microccf tick failed: {error}");
        }
    }
}

async fn tick(seed: &SeedClient, state: &Arc<RwLock<AppState>>) -> crate::error::Result<()> {
    let status = seed.status().await?;
    {
        let mut state = state.write().await;
        state.record_tick();
        state.record_status(
            status.value.witness_chain_length,
            status.value.total_vectors,
            status.value.epoch,
            status.value.paired,
        );
        state.record_latency(
            "status",
            std::time::Duration::from_secs_f64(status.millis / 1000.0),
        );
    }

    let tick = state.read().await.state_snapshot().tick_count;
    let vector = vector_for(tick);
    let context = {
        let mut state = state.write().await;
        state.record_vector(&vector)
    };
    let count = state
        .read()
        .await
        .state_snapshot()
        .counts
        .get(&context)
        .copied()
        .unwrap_or(0);

    let query = seed.query(&vector).await?;
    state.write().await.record_latency(
        "query",
        std::time::Duration::from_secs_f64(query.millis / 1000.0),
    );

    let ingest = seed
        .ingest_witness(&WitnessEvent {
            tick,
            context,
            count,
        })
        .await?;
    let id = extract_id(&ingest.value);
    {
        let mut state = state.write().await;
        state.record_ingest_id(id);
        state.record_latency(
            "ingest",
            std::time::Duration::from_secs_f64(ingest.millis / 1000.0),
        );
    }
    debug!("tick={tick} complete");
    Ok(())
}
