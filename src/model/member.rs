use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct MemberInfoReply {
    pub user_id: String,
    pub member_id: String,
    pub username: String,
    pub is_admin: bool,
    pub is_translator: bool,
    pub is_proofreader: bool,
    pub is_typesetter: bool,
    pub is_redrawer: bool,
    pub is_principal: bool,
}

#[derive(Debug, Serialize)]
pub struct MemberAbstract {
    pub member_id: String,
    pub user_id: String,
    pub username: String,
}

/// Payload used for member search requests.
#[derive(Debug, Deserialize)]
pub struct SearchMemberPayload {
    #[serde(alias = "teamId")]
    pub team_id: String,
    pub position: Option<String>,
    #[serde(alias = "fuzzyName")]
    pub fuzzy_name: Option<String>,

    pub page: Option<i64>,
    pub limit: Option<i64>,
}
