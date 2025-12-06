use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Deserialize)]
pub struct ProjAssignPayload {
    pub proj_id: String,
    pub member_id: String,

    pub mtr_auth: String,

    pub is_translator: bool,
    pub is_proofreader: bool,
    pub is_typesetter: bool,
}

#[derive(Debug, Serialize)]
pub struct ProjAssignInfoReply {
    pub proj_id: String,
    pub proj_name: String,
    pub projset_serial: i32,
    pub projset_index: i32,

    pub member_id: String,
    pub username: String,

    pub is_translator: bool,
    pub is_proofreader: bool,
    pub is_typesetter: bool,

    #[serde(with = "time::serde::timestamp")]
    pub updated_at: OffsetDateTime,
}
