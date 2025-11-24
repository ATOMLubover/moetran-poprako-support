use axum::{
    Extension,
    extract::{Query, State},
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
    Query(team_id): Query<String>,
) -> HttpResult<MemberInfoReply> {
    service::get_member_info(claims.sub, team_id, state.repo())
        .await
        .into()
}
