use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Serialize;

use crate::service::{ServiceError, ServiceResult};

#[derive(Debug, Serialize)]
pub struct HttpResult<T>
where
    T: Serialize,
{
    pub code: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T> HttpResult<T>
where
    T: Serialize,
{
    pub fn new(code: StatusCode, message: Option<String>, data: Option<T>) -> Self {
        Self {
            code: code.into(),
            message,
            data,
        }
    }
}

impl<T> From<ServiceError> for HttpResult<T>
where
    T: Serialize,
{
    fn from(err: ServiceError) -> Self {
        match err {
            ServiceError::RedisError(_) => HttpResult {
                code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                message: None,
                data: None,
            },
            ServiceError::DatabaseError(_) => HttpResult {
                code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                message: None,
                data: None,
            },
        }
    }
}

impl<T> From<ServiceResult<T>> for HttpResult<T>
where
    T: Serialize,
{
    fn from(service_result: ServiceResult<T>) -> Self {
        match service_result {
            Ok(value) => HttpResult {
                code: value.code,
                message: value.message,
                data: value.data,
            },
            Err(err) => Self::from(err),
        }
    }
}

impl<T> IntoResponse for HttpResult<T>
where
    T: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        let status_code =
            StatusCode::from_u16(self.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let body = axum::Json(self);

        (status_code, body).into_response()
    }
}
