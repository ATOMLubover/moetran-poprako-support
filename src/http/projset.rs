use axum::{Extension, Json, extract::State};

use crate::{
    http::result::HttpResult,
    model::{auth::Claims, projset::ProjSetCreatePayload, projset::ProjSetCreateReply},
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
