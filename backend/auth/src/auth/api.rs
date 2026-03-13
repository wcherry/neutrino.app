use crate::auth::{
    dto::{AuthResponse, LoginRequest, RefreshRequest, RegisterRequest, RegisterResponse, UserLookupResponse, UserProfileResponse},
    service::AuthService,
};
use crate::common::{ApiError, AuthenticatedUser};
use actix_web::{get, post, web};
use serde::Deserialize;
use std::sync::Arc;
use utoipa::OpenApi;

use tracing::error;


pub struct AuthApiState {
    pub auth_service: Arc<AuthService>,
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered successfully", body = RegisterResponse),
        (status = 400, description = "Bad request"),
        (status = 409, description = "Email already registered"),
    ),
    tag = "auth"
)]
#[post("/register")]
pub async fn register(
    state: web::Data<AuthApiState>,
    body: web::Json<RegisterRequest>,
) -> Result<web::Json<RegisterResponse>, ApiError> {
    let result = state.auth_service.register(body.into_inner());
    match result {
        Ok(response) => Ok(web::Json(response)),
        Err(e) => { error!("Error {}", e); Err(e) }
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 401, description = "Invalid credentials"),
    ),
    tag = "auth"
)]
#[post("/login")]
pub async fn login(
    state: web::Data<AuthApiState>,
    body: web::Json<LoginRequest>,
) -> Result<web::Json<AuthResponse>, ApiError> {
    let response = state.auth_service.login(body.into_inner())?;
    Ok(web::Json(response))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/refresh",
    request_body = RefreshRequest,
    responses(
        (status = 200, description = "Token refreshed successfully", body = AuthResponse),
        (status = 401, description = "Invalid or expired refresh token"),
    ),
    tag = "auth"
)]
#[post("/refresh")]
pub async fn refresh(
    state: web::Data<AuthApiState>,
    body: web::Json<RefreshRequest>,
) -> Result<web::Json<AuthResponse>, ApiError> {
    let response = state.auth_service.refresh(body.into_inner())?;
    Ok(web::Json(response))
}

#[utoipa::path(
    get,
    path = "/api/v1/auth/me",
    responses(
        (status = 200, description = "Current user profile", body = UserProfileResponse),
        (status = 401, description = "Not authenticated"),
    ),
    security(("bearer_auth" = [])),
    tag = "auth"
)]
#[get("/me")]
pub async fn me(
    state: web::Data<AuthApiState>,
    user: AuthenticatedUser,
) -> Result<web::Json<UserProfileResponse>, ApiError> {
    let profile = state.auth_service.get_profile(&user.user_id)?;
    Ok(web::Json(profile))
}

#[derive(Deserialize)]
pub struct LookupByEmailQuery {
    pub email: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/auth/users/lookup",
    params(("email" = String, Query, description = "Email address to look up")),
    responses(
        (status = 200, description = "User found", body = UserLookupResponse),
        (status = 404, description = "User not found"),
        (status = 401, description = "Not authenticated"),
    ),
    security(("bearer_auth" = [])),
    tag = "auth"
)]
#[get("/users/lookup")]
pub async fn lookup_user_by_email(
    state: web::Data<AuthApiState>,
    _user: AuthenticatedUser,
    query: web::Query<LookupByEmailQuery>,
) -> Result<web::Json<UserLookupResponse>, ApiError> {
    match state.auth_service.lookup_user_by_email(&query.email)? {
        Some(u) => Ok(web::Json(u)),
        None => Err(ApiError::not_found("User not found")),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/auth/users/{user_id}",
    params(("user_id" = String, Path, description = "User ID")),
    responses(
        (status = 200, description = "User found", body = UserLookupResponse),
        (status = 404, description = "User not found"),
        (status = 401, description = "Not authenticated"),
    ),
    security(("bearer_auth" = [])),
    tag = "auth"
)]
#[get("/users/{user_id}")]
pub async fn get_user_by_id(
    state: web::Data<AuthApiState>,
    _user: AuthenticatedUser,
    path: web::Path<String>,
) -> Result<web::Json<UserLookupResponse>, ApiError> {
    let user_id = path.into_inner();
    match state.auth_service.get_user_by_id(&user_id)? {
        Some(u) => Ok(web::Json(u)),
        None => Err(ApiError::not_found("User not found")),
    }
}

pub fn configure(conf: &mut web::ServiceConfig) {
    conf.service(
        web::scope("/auth")
            .service(register)
            .service(login)
            .service(refresh)
            .service(me)
            .service(lookup_user_by_email)
            .service(get_user_by_id),
    );
}

#[derive(OpenApi)]
#[openapi(
    paths(register, login, refresh, me, lookup_user_by_email, get_user_by_id),
    components(schemas(RegisterRequest, LoginRequest, RefreshRequest, AuthResponse, RegisterResponse, UserProfileResponse, UserLookupResponse)),
    tags((name = "auth", description = "Authentication endpoints"))
)]
pub struct AuthApiDoc;
