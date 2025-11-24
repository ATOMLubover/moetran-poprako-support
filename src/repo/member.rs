pub struct MemberBasic {
    pub f_member_id: String,
    pub f_user_id: String,
    pub f_team_id: String,

    pub f_is_admin: bool,
    pub f_is_translator: bool,
    pub f_is_proofreader: bool,
    pub f_is_typesetter: bool,
    pub f_is_principal: bool,
}

pub struct MemberPerm {
    pub f_user_id: String,
    pub f_team_id: String,
    pub f_is_admin: bool,
}
