pub struct NewProjSet {
    f_projset_id: String,
    f_projset_name: String,
    f_description: Option<String>,

    f_projset_serial: i32,

    f_team_id: String,
}

pub struct ProjSetBasic {
    pub f_projset_id: String,
    pub f_projset_name: String,
    pub f_description: Option<String>,

    pub f_projset_serial: i32,

    pub f_team_id: String,
}
