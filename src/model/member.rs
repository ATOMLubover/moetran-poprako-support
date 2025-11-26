use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct MemberInfoReply {
    pub member_id: String,
    pub username: String,
    pub is_admin: bool,
    pub is_translator: bool,
    pub is_proofreader: bool,
    pub is_typesetter: bool,
    pub is_principal: bool,
}

#[derive(Debug, Serialize)]
pub struct MemberAbstract {
    pub member_id: String,
    pub username: String,
}

/// Payload used for member search requests.
#[derive(Debug, Deserialize)]
pub struct SearchMemberPayload {
    pub team_id: String,
    pub position: Option<String>,
    pub fuzzy_name: Option<String>,

    pub page: Option<i64>,
    pub limit: Option<i64>,
}
