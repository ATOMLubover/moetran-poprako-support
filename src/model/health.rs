use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct HealthCheckReply {
    pub healthy: bool,
    pub status: String,
    pub comment: String,
}
