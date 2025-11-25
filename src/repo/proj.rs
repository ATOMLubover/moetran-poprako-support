pub struct ProjBasic {
    pub f_proj_id: String,
    pub f_proj_name: String,

    pub f_projset_id: String,
    pub f_projset_serial: i32,
    pub f_projset_index: i32,

    pub f_translating_status: i32,
    pub f_proofreading_status: i32,
    pub f_typesetting_status: i32,
    pub f_reviewing_status: i32,
    pub f_is_published: bool,
}
