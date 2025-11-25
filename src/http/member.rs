use std::collections::HashMap;

use axum::{
    Extension,
    extract::{Query, State},
};
use reqwest::StatusCode;

use crate::{
    http::result::HttpResult,
    model::{
        auth::Claims,
        member::{MemberAbstract, MemberInfoReply},
    },
    service,
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

/// Supported postions: "translator", "proofreader", "typesetter", "principal".
pub async fn pick_members_by_position(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> HttpResult<Vec<MemberAbstract>> {
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

    let position = match params.get("position") {
        Some(pos) => pos.clone(),
        None => {
            return HttpResult::new(
                StatusCode::BAD_REQUEST,
                Some("Missing position parameter.".to_string()),
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

    service::pick_members_by_position(team_id, position, page, limit, state.repo())
        .await
        .into()
}
