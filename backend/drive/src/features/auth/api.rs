use crate::features::auth::{
    dto::{AuthResponse, LoginRequest, RefreshRequest, RegisterRequest, RegisterResponse},
    service::AuthService,
};
use crate::features::shared::ApiError;
use actix_web::{post, web};
use std::sync::Arc;
use utoipa::OpenApi;

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
    let response = state.auth_service.register(body.into_inner())?;
    Ok(web::Json(response))
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

pub fn configure(conf: &mut web::ServiceConfig) {
    conf.service(
        web::scope("/auth")
            .service(register)
            .service(login)
            .service(refresh),
    );
}

#[derive(OpenApi)]
#[openapi(
    paths(register, login, refresh),
    components(schemas(RegisterRequest, LoginRequest, RefreshRequest, AuthResponse, RegisterResponse)),
    tags((name = "auth", description = "Authentication endpoints"))
)]
pub struct AuthApiDoc;
