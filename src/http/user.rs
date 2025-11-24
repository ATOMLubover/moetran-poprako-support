use axum::{Extension, Json, extract::State};

use crate::{
    http::result::HttpResult,
    model::{
        auth::Claims,
        user::{SyncTokenReply, SyncUserPayload, UserInfoReply},
    },
    service,
    state::AppState,
};

pub async fn sync_user(
    State(state): State<AppState>,
    Json(payload): Json<SyncUserPayload>,
) -> HttpResult<SyncTokenReply> {
    service::sync_user(payload, state.config(), state.jwt_codec(), state.repo())
        .await
        .into()
}

pub async fn get_user_info(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> HttpResult<UserInfoReply> {
    service::get_user_info(claims.sub, state.repo())
        .await
        .into()
}
