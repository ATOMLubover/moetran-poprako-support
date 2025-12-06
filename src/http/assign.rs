use axum::{
    Extension, Json,
    extract::{Path, Query, State},
};
use std::collections::HashMap;

use crate::{
    http::result::HttpResult,
    model::{
        assign::{ProjAssignInfoReply, ProjAssignPayload},
        auth::Claims,
    },
    service,
    state::AppState,
};

pub async fn assign_member(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(proj_id): Path<String>,
    Json(mut payload): Json<ProjAssignPayload>,
) -> HttpResult<()> {
    // Ensure proj_id from path is used as the target project.
    payload.proj_id = proj_id;

    service::assign_member(
        state.crawler(),
        state.config(),
        claims.sub,
        payload,
        state.repo(),
    )
    .await
    .into()
}

pub async fn get_assigns(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> HttpResult<Vec<ProjAssignInfoReply>> {
    let time_start = params
        .get("time_start")
        .and_then(|t| t.parse::<i64>().ok())
        .unwrap_or(0);

    let page = params
        .get("page")
        .and_then(|p| p.parse::<i64>().ok())
        .unwrap_or(1);
    let limit = params
        .get("limit")
        .and_then(|l| l.parse::<i64>().ok())
        .unwrap_or(10);

    service::get_assigns(time_start, page, limit, state.repo()).await.into()
}
