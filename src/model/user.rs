use serde::{Deserialize, Serialize};

use crate::model::team::TeamInfoReply;

#[derive(Debug, Clone, Deserialize)]
pub struct SyncUserPayload {
    #[serde(alias = "userId")]
    pub user_id: String,
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SyncTokenReply {
    pub token: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserInfoReply {
    pub user_id: String,
    pub username: String,
    pub email: String,
    pub teams: Vec<TeamInfoReply>,
}
