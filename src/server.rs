use std::sync::Arc;

use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use tokio::sync::RwLock;

use crate::state::AppState;

#[derive(Clone)]
pub struct ServerState {
    pub state: Arc<RwLock<AppState>>,
}

#[derive(Serialize)]
struct Health {
    service: &'static str,
    status: &'static str,
    warning: &'static str,
}

pub fn router(state: Arc<RwLock<AppState>>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/state", get(current_state))
        .route("/metrics", get(metrics))
        .route("/version", get(version))
        .with_state(ServerState { state })
}

async fn health() -> impl IntoResponse {
    Json(Health {
        service: "microccf",
        status: "ok",
        warning: "Spike build, NOT v1.0",
    })
}

async fn current_state(State(server): State<ServerState>) -> impl IntoResponse {
    Json(server.state.read().await.state_snapshot())
}

async fn metrics(State(server): State<ServerState>) -> impl IntoResponse {
    Json(server.state.read().await.metrics_snapshot())
}

async fn version() -> impl IntoResponse {
    format!(
        "microccf {} - Spike build, NOT v1.0\n",
        env!("CARGO_PKG_VERSION")
    )
}
