use axum::{
    Extension, Json,
    extract::{Path, State},
};

use crate::{
    http::result::HttpResult,
    model::{assign::ProjAssignPayload, auth::Claims},
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

    service::assign_member(claims.sub, payload, state.repo())
        .await
        .into()
}
