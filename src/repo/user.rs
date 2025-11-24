pub struct UserSecret {
    pub f_user_id: String,
    pub f_password_hash: String,
}

pub struct NewUser {
    pub f_user_id: String,
    pub f_username: String,
    pub f_email: String,
    pub f_password_hash: String,
}

pub struct UserBasic {
    pub f_user_id: String,
    pub f_username: String,
    pub f_email: String,
}
