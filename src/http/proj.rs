use std::collections::HashMap;

use axum::{
    Extension, Json,
    extract::{Path, Query, State},
};
use reqwest::StatusCode;

use crate::{
    http::result::HttpResult,
    model::{
        auth::Claims,
        proj::{
            MarkProjStatusPayload, ProjCreatePayload, ProjCreateReply, ProjInfoReply,
            SearchProjPayload,
        },
    },
    service,
    state::AppState,
};

pub async fn create_proj(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<ProjCreatePayload>,
) -> HttpResult<ProjCreateReply> {
    service::create_proj(
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

/// POST /projs
pub async fn get_projs_by_id(
    State(state): State<AppState>,
    Json(payload): Json<SearchProjPayload>,
) -> HttpResult<Vec<ProjInfoReply>> {
    service::search_projs(payload, state.repo()).await.into()
}

/// GET /projs?team_id=&page=&limit=
pub async fn get_projs(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> HttpResult<Vec<ProjInfoReply>> {
    let team_id = match params.get("team_id") {
        Some(tid) => tid.clone(),
        None => {
            return HttpResult::new(
                StatusCode::BAD_REQUEST,
                Some("Missing team_id parameter.".to_string()),
                None,
            );
        }
    };

    let page = params
        .get("page")
        .and_then(|p| p.parse::<i64>().ok())
        .unwrap_or(1);
    let limit = params
        .get("limit")
        .and_then(|l| l.parse::<i64>().ok())
        .unwrap_or(10);

    service::get_projs(team_id, page, limit, state.repo())
        .await
        .into()
}

/// PUT /projs/:proj_id/status
pub async fn mark_proj_status(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(proj_id): Path<String>,
    Json(mut payload): Json<MarkProjStatusPayload>,
) -> HttpResult<()> {
    // Ensure proj_id from path is authoritative.
    payload.proj_id = proj_id;

    service::mark_proj_status(claims.sub, payload, state.repo())
        .await
        .into()
}

/// PUT /projs/:proj_id/publish
pub async fn mark_proj_published(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(proj_id): Path<String>,
) -> HttpResult<()> {
    service::mark_proj_published(claims.sub, proj_id, state.repo())
        .await
        .into()
}
