use axum::extract::State;
use axum::http::StatusCode;

use crate::http::result::HttpResult;
use crate::model::health::HealthCheck;
use crate::service;
use crate::state::AppState;

pub async fn health_check() -> HttpResult<HealthCheck> {
    service::health_check().await.into()
}

pub async fn check_app_state(State(state): State<AppState>) -> HttpResult<String> {
    if std::env::var("APP_ENV")
        .unwrap_or("production".to_string())
        .to_lowercase()
        != "development"
    {
        return HttpResult::new(StatusCode::FORBIDDEN, None, None);
    }

    HttpResult::new(
        StatusCode::OK,
        Some("AppState is accessible.".to_string()),
        Some(format!("AppState: {:?}", &state)),
    )
}
