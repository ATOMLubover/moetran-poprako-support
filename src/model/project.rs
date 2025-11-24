use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct ProjCreatePayload {
    pub proj_name: String,
    pub proj_description: Option<String>,

    pub team_id: String,
    pub projset_id: String,

    pub mtr_auth: String,

    pub workset_index: u32,

    pub source_language: String,
    pub target_languages: Vec<String>,

    pub allow_apply_type: i32,
    pub application_check_type: i32,

    pub default_role: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjCreateReply {
    pub proj_serial: i32,
    pub projset_index: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjInfoReply {
    pub proj_id: String,
    pub proj_name: String,
    pub description: Option<String>,

    pub proj_serial: i32,
    pub projset_index: i32,

    pub projset_id: String,
}
