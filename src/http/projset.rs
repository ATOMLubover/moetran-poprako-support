use axum::{
    Extension, Json,
    extract::{Query, State},
};
use reqwest::StatusCode;
use std::collections::HashMap;

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
    Query(params): Query<HashMap<String, String>>,
) -> HttpResult<TeamProjSetReply> {
    let team_id = match params.get("team_id") {
        Some(t) => t.clone(),
        None => {
            return HttpResult::new(
                StatusCode::BAD_REQUEST,
                Some("Missing team_id query parameter.".to_string()),
                None,
            );
        }
    };

    service::get_projsets_by_team(team_id, state.repo())
        .await
        .into()
}
