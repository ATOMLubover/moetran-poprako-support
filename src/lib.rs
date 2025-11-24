use tracing::debug;
use tracing_subscriber::EnvFilter;

mod cache;
mod config;
mod http;
mod jwt;
mod model;
mod repo;
mod service;
mod state;

use crate::{cache::Cache, config::AppConfig, jwt::JwtCodec, repo::Repo, state::AppState};

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

    // Test the database with an initial PING command to ensure connectivity.
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

fn init_jwt_codec() -> anyhow::Result<JwtCodec> {
    let jwt_secret = std::env::var("JWT_SECRET").map_err(|err| {
        anyhow::anyhow!("Error when reading JWT_SECRET from environment: {}", err)
    })?;

    let jwt_codec = JwtCodec::new(jwt_secret);

    debug!("JWT Codec initialized");

    Ok(jwt_codec)
}

pub async fn run() -> anyhow::Result<()> {
    init_env().await?;

    init_logger().await;

    let config = init_config().await?;

    let cache = init_cache().await?;

    let repo = init_repo().await?;

    let jwt_codec = init_jwt_codec()?;

    let app_state = AppState::new(config, repo, cache, jwt_codec);

    http::run_server(&app_state).await?;

    Ok(())
}
