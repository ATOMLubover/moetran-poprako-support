use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct LoginUser {
    pub user_id: String,
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LoginToken {
    pub token: String,
}
