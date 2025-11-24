use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct MemberInfoReply {
    pub member_id: String,
    pub is_admin: bool,
    pub is_translator: bool,
    pub is_proofreader: bool,
    pub is_typesetter: bool,
    pub is_principal: bool,
}

#[derive(Debug, Serialize)]
pub struct MemberAbstract {
    pub member_id: String,
    pub team_id: String,
    pub username: String,
}
