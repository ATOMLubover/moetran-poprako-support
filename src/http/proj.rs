use axum::{Extension, Json, extract::State};

use crate::{
    http::result::HttpResult,
    model::{auth::Claims, project::{ProjCreatePayload, ProjCreateReply}},
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
