use crate::repo::Repo;

pub struct UserSecret {
    pub user_id: String,
    pub password_hash: String,
}

pub struct UserRow {
    pub user_id: String,
    pub username: String,
    pub email: String,
    pub password_hash: String,
}
