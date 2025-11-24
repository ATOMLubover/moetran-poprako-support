use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct ProjSetCreatePayload {
    pub projset_name: String,
    pub projset_description: Option<String>,
    pub team_id: String,
    pub mtr_token: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjSetCreateReply {
    pub projset_serial: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjSetInfoReply {
    pub projset_id: String,
    pub projset_name: String,
    pub description: Option<String>,
    pub projset_serial: i32,
    pub team_id: String,
}
