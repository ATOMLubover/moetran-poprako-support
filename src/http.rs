use std::net::SocketAddrV4;

use axum::Router;
use axum::routing;
use tokio::net::TcpListener;
use tracing::debug;

mod health;
mod result;

use crate::state::AppState;

async fn init_router(app_state: &AppState) -> anyhow::Result<Router> {
    let health_router = Router::new()
        .route("/check", routing::get(health::health_check))
        .route("/app_state", routing::get(health::check_app_state));

    let router = Router::new()
        .nest("/health", health_router)
        .with_state(app_state.clone());

    Ok(router)
}

async fn bind_addr(host: &str, port: u16) -> anyhow::Result<TcpListener> {
    let addr: SocketAddrV4 = format!("{}:{}", host, port)
        .parse()
        .map_err(|err| anyhow::anyhow!("Error when parsing listening address: {}", err))?;

    let listener = TcpListener::bind(&addr)
        .await
        .map_err(|err| anyhow::anyhow!("Error when binding to address {}: {}", addr, err))?;

    debug!("Server now listening on {}", addr);

    Ok(listener)
}

async fn signal_term() {
    debug!("SIGNAL TERM receiver installed");

    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL-C signal handler");

    debug!("SIGNAL TERM received, shutting down gracefully...");
}

pub async fn run_server(app_state: &AppState) -> anyhow::Result<()> {
    // Initialize tokio TCP listener.
    let server_host = &app_state.config().server_host;
    let server_port = app_state.config().server_port;

    let listener = bind_addr(server_host, server_port).await?;

    // Initialize make service on router.
    let router = init_router(app_state).await?;

    axum::serve(listener, router)
        .with_graceful_shutdown(signal_term())
        .await
        .map_err(|err| anyhow::anyhow!("Error running server: {}", err))?;

    Ok(())
}
