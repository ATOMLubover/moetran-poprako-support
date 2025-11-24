use serde::Serialize;
use thiserror::Error;

pub type ServiceResult<T> = Result<ServiceValue<T>, ServiceError>;

pub fn succeed<T>() -> ServiceValue<T>
where
    T: Serialize,
{
    ServiceValue::<T>::default()
}

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Redis error: {0}")]
    RedisError(#[from] redis::RedisError),
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
}

#[derive(Debug, Serialize)]
pub struct ServiceValue<T = ()>
where
    T: Serialize,
{
    pub code: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T> ServiceValue<T>
where
    T: Serialize,
{
    pub fn default() -> Self {
        Self {
            code: 200,
            message: None,
            data: None,
        }
    }

    pub fn with_code(mut self, code: u16) -> Self {
        self.code = code;

        self
    }

    pub fn with_message<S: Into<String>>(mut self, message: S) -> Self {
        self.message = Some(message.into());

        self
    }

    pub fn with_data(mut self, data: T) -> Self {
        self.data = Some(data);

        self
    }
}
