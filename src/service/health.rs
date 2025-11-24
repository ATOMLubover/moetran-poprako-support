use crate::{
    model::health::HealthCheckReply,
    service::result::{ServiceResult, pass},
};

pub async fn health_check() -> ServiceResult<HealthCheckReply> {
    let heath_check = HealthCheckReply {
        healthy: true,
        status: "OK".to_string(),
        comment: "Service is running smoothly.".to_string(),
    };

    Ok(pass()
        .with_message("Pong from server.")
        .with_code(200)
        .with_data(heath_check))
}
