use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct HealthCheck {
    pub healthy: bool,
    pub status: String,
    pub comment: String,
}
