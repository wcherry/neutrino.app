use crate::auth::tokens::TokenService;
use crate::shared::ApiError;
use actix_web::{web, FromRequest, HttpRequest};
use std::future::{ready, Ready};
use std::sync::Arc;

#[allow(dead_code)]
pub struct AuthenticatedUser {
    pub user_id: String,
    pub email: String,
}

impl FromRequest for AuthenticatedUser {
    type Error = ApiError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        let result = extract_user(req);
        ready(result)
    }
}

fn extract_user(req: &HttpRequest) -> Result<AuthenticatedUser, ApiError> {
    let token_service = req
        .app_data::<web::Data<Arc<TokenService>>>()
        .ok_or_else(|| ApiError::internal("Token service unavailable"))?;

    let auth_header = req
        .headers()
        .get("Authorization")
        .ok_or_else(|| ApiError::unauthorized("Missing Authorization header"))?
        .to_str()
        .map_err(|_| ApiError::unauthorized("Invalid Authorization header"))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| ApiError::unauthorized("Authorization header must use Bearer scheme"))?;

    let claims = token_service.validate_access_token(token)?;

    Ok(AuthenticatedUser {
        user_id: claims.sub,
        email: claims.email,
    })
}
