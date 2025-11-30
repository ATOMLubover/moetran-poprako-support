use axum::{
    Extension, Json,
    extract::{Path, State},
};

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
