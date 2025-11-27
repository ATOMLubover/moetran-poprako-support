use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};

use crate::model::auth::Claims;
use crate::state::AppState;

/// Authentication middleware to validate JWT tokens in the Authorization header.
/// The format of token is expected to be "Bearer <token>".
pub async fn auth_middleware(
    State(state): State<AppState>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract the bearer token from the Authorization header
    // and validate it using the JWT secret from AppState.
    let claims: Claims = state
        .jwt_codec()
        .decode_token(auth.token())
        .map_err(|err| {
            tracing::warn!("JWT decoding error: {}", err);
            StatusCode::UNAUTHORIZED
        })?;

    // Insert claims into request extensions for downstream handlers to access.
    req.extensions_mut().insert(claims);

    Ok(next.run(req).await)
}
