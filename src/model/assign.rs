use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ProjAssignPayload {
    pub proj_id: String,
    pub member_id: String,
    
    pub is_translator: bool,
    pub is_proofreader: bool,
    pub is_typesetter: bool,
    pub is_principal: bool,
}
