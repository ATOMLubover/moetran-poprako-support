use crate::{
    model::health::HealthCheck,
    service::result::{ServiceResult, succeed},
};

pub async fn health_check() -> ServiceResult<HealthCheck> {
    let heath_check = HealthCheck {
        healthy: true,
        status: "OK".to_string(),
        comment: "Service is running smoothly.".to_string(),
    };

    Ok(succeed()
        .with_message("Pong from server.")
        .with_code(200)
        .with_data(heath_check))
}
