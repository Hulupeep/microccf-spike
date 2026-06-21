mod config;
mod error;
mod loop_runner;
mod seed_client;
mod server;
mod state;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use clap::Parser;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::config::Config;
use crate::error::Result;
use crate::seed_client::SeedClient;
use crate::state::AppState;

#[derive(Debug, Parser)]
#[command(version, about = "microCCF: Spike build, NOT v1.0")]
struct Args {
    #[arg(long, default_value = "/etc/microccf/config.toml")]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let config = Config::load(&args.config)?;
    init_logging(&config.logging.level);

    info!("microccf starting: Spike build, NOT v1.0");
    let token = config.load_token()?;
    let seed = SeedClient::new(&config, token)?;
    let state = Arc::new(RwLock::new(AppState::new()));

    let loop_state = Arc::clone(&state);
    let loop_config = config.clone();
    let loop_seed = seed.clone();
    tokio::spawn(async move {
        loop_runner::run_loop(loop_config, loop_seed, loop_state).await;
    });

    serve(config.server.bind, state).await
}

async fn serve(bind: SocketAddr, state: Arc<RwLock<AppState>>) -> Result<()> {
    let listener = TcpListener::bind(bind).await?;
    info!("microccf listening on {bind}");
    axum::serve(listener, server::router(state)).await?;
    Ok(())
}

fn init_logging(level: &str) {
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(level))
        .unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}
