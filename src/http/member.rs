use std::collections::HashMap;

use axum::{
    Extension,
    extract::{Json, Query, State},
};
use reqwest::StatusCode;

use crate::{
    http::result::HttpResult,
    model::{
        auth::Claims,
        member::{MemberAbstract, MemberInfoReply, SearchMemberPayload},
    },
    service,
    state::AppState,
};

pub async fn get_member_info(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<HashMap<String, String>>,
) -> HttpResult<MemberInfoReply> {
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

    let position = params
        .get("position")
        .clone()
        .and_then(|p| Some(p.to_owned()));

    let fuzzy_name = params
        .get("fuzzy_name")
        .clone()
        .and_then(|n| Some(n.to_owned()));

    let page = params
        .get("page")
        .and_then(|p| p.parse::<i64>().ok())
        .unwrap_or(1);
    let limit = params
        .get("limit")
        .and_then(|l| l.parse::<i64>().ok())
        .unwrap_or(10);

    let payload = SearchMemberPayload {
        team_id,
        position,
        fuzzy_name,
        page: Some(page),
        limit: Some(limit),
    };

    service::search_members(payload, state.repo()).await.into()
}

/// POST /members/search - accept `PickMemberPayload` in request body (JSON)
pub async fn search_members(
    State(state): State<AppState>,
    Json(payload): Json<SearchMemberPayload>,
) -> HttpResult<Vec<MemberAbstract>> {
    service::search_members(payload, state.repo()).await.into()
}
