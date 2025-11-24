use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct TeamInfoReply {
    pub team_id: String,
    pub team_name: String,
}
