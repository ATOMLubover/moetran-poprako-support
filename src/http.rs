use std::net::SocketAddrV4;

use axum::Router;
use axum::middleware::from_fn_with_state;
use axum::routing::{get, post, put};
use tokio::net::TcpListener;
use tracing::debug;

mod assign;
mod health;
mod member;
mod middleware;
mod proj;
mod projset;
mod result;
mod user;

use crate::http::assign::assign_member;
use crate::http::middleware::auth_middleware;
use crate::http::proj::{
    create_proj as http_create_proj, get_projs_by_id as http_get_projs_by_id,
    mark_proj_published as http_mark_proj_published, mark_proj_status as http_mark_proj_status,
};
use crate::http::projset::{
    create_projset as http_create_projset, get_projsets_by_team as http_get_projset_by_team,
};
use crate::http::user::get_user_info;
use crate::http::user::sync_user;
use crate::state::AppState;

async fn init_router(app_state: &AppState) -> anyhow::Result<Router> {
    let health_router = Router::new()
        .route("/check", get(health::health_check))
        .route("/app_state", get(health::check_app_state));

    let user_router = Router::new().route("/{user_id}", get(get_user_info));

    let member_router = Router::new()
        .route("/info", get(member::get_member_info))
        .route("/", get(member::pick_members_by_position))
        .route("/search", post(member::search_members));

    let proj_router = Router::new()
        .route("/", post(http_create_proj))
        .route("/search", post(http_get_projs_by_id))
        .route("/{proj_id}/assign", post(assign_member))
        .route("/{proj_id}/status", put(http_mark_proj_status))
        .route("/{proj_id}/publish", put(http_mark_proj_published));

    let projset_router = Router::new()
        .route("/", get(http_get_projset_by_team))
        .route("/", post(http_create_projset));

    let api_router = Router::new()
        .nest("/health", health_router)
        .nest("/users", user_router)
        .nest("/members", member_router)
        .nest("/projs", proj_router)
        .nest("/projsets", projset_router)
        .route_layer(from_fn_with_state(app_state.clone(), auth_middleware));

    let router = Router::new()
        .route("/api/v1/sync", post(sync_user))
        .nest("/api/v1", api_router)
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
