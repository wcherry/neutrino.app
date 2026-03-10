use crate::auth::{
    dto::{AuthResponse, LoginRequest, RefreshRequest, RegisterRequest, RegisterResponse, UserProfileResponse},
    service::AuthService,
};
use crate::shared::{ApiError, AuthenticatedUser};
use actix_web::{get, post, web};
use std::sync::Arc;
use utoipa::OpenApi;

use log::error;


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

pub fn configure(conf: &mut web::ServiceConfig) {
    conf.service(
        web::scope("/auth")
            .service(register)
            .service(login)
            .service(refresh)
            .service(me),
    );
}

#[derive(OpenApi)]
#[openapi(
    paths(register, login, refresh, me),
    components(schemas(RegisterRequest, LoginRequest, RefreshRequest, AuthResponse, RegisterResponse, UserProfileResponse)),
    tags((name = "auth", description = "Authentication endpoints"))
)]
pub struct AuthApiDoc;
