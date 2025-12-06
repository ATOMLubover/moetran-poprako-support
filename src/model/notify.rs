use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct UpdateNotifyReply {
    pub has_update: bool,
}
