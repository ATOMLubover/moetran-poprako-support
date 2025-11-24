use axum::{
    Extension,
    extract::{Path, State},
};

use crate::{
    http::result::HttpResult,
    model::{auth::Claims, member::MemberInfoReply},
    service::{self},
    state::AppState,
};

pub async fn get_member_info(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(member_id): Path<String>,
) -> HttpResult<MemberInfoReply> {
    service::get_member_info(claims.sub, member_id, state.repo())
        .await
        .into()
}
