use axum::{
    Extension, Json,
    extract::{Path, Query, State},
};

use crate::{
    http::result::HttpResult,
    model::{
        auth::Claims,
        projset::{ProjSetCreatePayload, ProjSetCreateReply, TeamProjSetReply},
    },
    service,
    state::AppState,
};

pub async fn create_projset(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<ProjSetCreatePayload>,
) -> HttpResult<ProjSetCreateReply> {
    service::create_projset(
        claims.sub,
        payload,
        state.config(),
        state.crawler(),
        state.cache(),
        state.repo(),
    )
    .await
    .into()
}

pub async fn get_projsets_by_team(
    State(state): State<AppState>,
    Query(team_id): Query<String>,
) -> HttpResult<TeamProjSetReply> {
    service::get_projsets_by_team(team_id, state.repo())
        .await
        .into()
}
