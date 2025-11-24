use tracing::debug;
use tracing_subscriber::EnvFilter;

mod cache;
mod config;
mod http;
mod model;
mod repo;
mod service;
mod state;

use crate::{cache::Cache, config::AppConfig, repo::Repo, state::AppState};

async fn init_env() -> anyhow::Result<()> {
    dotenvy::dotenv().map_err(|err| anyhow::anyhow!("Error when loading env: {}", err))?;

    debug!("Environment variables loaded from .env file");

    Ok(())
}

async fn init_logger() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    debug!("Logger initialized");
}

async fn init_config() -> anyhow::Result<AppConfig> {
    let config = AppConfig::try_from_file(None)
        .map_err(|err| anyhow::anyhow!("Error when loading config: {}", err))?;

    debug!("Configuration loaded: {:?}", config);

    Ok(config)
}

async fn init_repo() -> anyhow::Result<Repo> {
    let database = Repo::new()
        .await
        .map_err(|err| anyhow::anyhow!("Error when initializing Database connection: {}", err))?;

    database
        .ping()
        .await
        .map_err(|err| anyhow::anyhow!("Error when PING Database: {}", err))?;

    debug!("Database connected");

    Ok(database)
}

async fn init_cache() -> anyhow::Result<Cache> {
    let cache =
        Cache::new().map_err(|err| anyhow::anyhow!("Error when initializing Cache: {}", err))?;

    // Test the Redis with an initial PING command to ensure connectivity
    cache
        .ping()
        .await
        .map_err(|err| anyhow::anyhow!("Error when PING Redis: {}", err))?;

    debug!("Redis connected");

    Ok(cache)
}

fn compose_app_state(config: AppConfig, cache: Cache, repo: Repo) -> AppState {
    AppState::new(config, repo, cache)
}

pub async fn run() -> anyhow::Result<()> {
    init_env().await?;

    init_logger().await;

    let config = init_config().await?;

    let cache = init_cache().await?;

    let repo = init_repo().await?;

    let app_state = compose_app_state(config, cache, repo);

    http::run_server(&app_state).await?;

    Ok(())
}
