use crate::auth::{
    dto::{
        AdminUpdateUserRequest, AdminUserListResponse, AdminUserResponse, AuthResponse,
        LoginRequest, LoginResponse, RefreshRequest, RegisterRequest, RegisterResponse,
        SessionListResponse, TwoFactorConfirmRequest, TwoFactorDisableRequest,
        TwoFactorEnrollResponse, TwoFactorStatusResponse, UserLookupResponse, UserProfileResponse,
    },
    service::AuthService,
};
use crate::common::{ApiError, AuthenticatedUser};
use actix_web::{delete, get, patch, post, web};
use serde::Deserialize;
use std::sync::Arc;
use utoipa::OpenApi;

use tracing::error;

pub struct AuthApiState {
    pub auth_service: Arc<AuthService>,
}

// ── Register ──────────────────────────────────────────────────────────────────

#[post("/register")]
pub async fn register(
    state: web::Data<AuthApiState>,
    body: web::Json<RegisterRequest>,
) -> Result<web::Json<RegisterResponse>, ApiError> {
    let result = state.auth_service.register(body.into_inner());
    match result {
        Ok(response) => Ok(web::Json(response)),
        Err(e) => {
            error!("Error {}", e);
            Err(e)
        }
    }
}

// ── Login ─────────────────────────────────────────────────────────────────────

#[post("/login")]
pub async fn login(
    state: web::Data<AuthApiState>,
    req: actix_web::HttpRequest,
    body: web::Json<LoginRequest>,
) -> Result<web::Json<LoginResponse>, ApiError> {
    let device_name = req
        .headers()
        .get("X-Device-Name")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let user_agent = req
        .headers()
        .get("User-Agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let ip_address = req
        .connection_info()
        .realip_remote_addr()
        .map(|s| s.to_string());

    let response = state
        .auth_service
        .login(body.into_inner(), device_name, user_agent, ip_address)?;
    Ok(web::Json(response))
}

// ── Refresh ───────────────────────────────────────────────────────────────────

#[post("/refresh")]
pub async fn refresh(
    state: web::Data<AuthApiState>,
    body: web::Json<RefreshRequest>,
) -> Result<web::Json<AuthResponse>, ApiError> {
    let response = state.auth_service.refresh(body.into_inner())?;
    Ok(web::Json(response))
}

// ── Me ────────────────────────────────────────────────────────────────────────

#[get("/me")]
pub async fn me(
    state: web::Data<AuthApiState>,
    user: AuthenticatedUser,
) -> Result<web::Json<UserProfileResponse>, ApiError> {
    let profile = state.auth_service.get_profile(&user.user_id)?;
    Ok(web::Json(profile))
}

// ── User Lookup ───────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct LookupByEmailQuery {
    pub email: String,
}

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

// ── 2FA ───────────────────────────────────────────────────────────────────────

#[get("/2fa/status")]
pub async fn two_factor_status(
    state: web::Data<AuthApiState>,
    user: AuthenticatedUser,
) -> Result<web::Json<TwoFactorStatusResponse>, ApiError> {
    let status = state.auth_service.get_two_factor_status(&user.user_id)?;
    Ok(web::Json(status))
}

#[post("/2fa/enroll")]
pub async fn two_factor_enroll(
    state: web::Data<AuthApiState>,
    user: AuthenticatedUser,
) -> Result<web::Json<TwoFactorEnrollResponse>, ApiError> {
    let result = state
        .auth_service
        .enroll_two_factor(&user.user_id, &user.email)?;
    Ok(web::Json(result))
}

#[post("/2fa/confirm")]
pub async fn two_factor_confirm(
    state: web::Data<AuthApiState>,
    user: AuthenticatedUser,
    body: web::Json<TwoFactorConfirmRequest>,
) -> Result<web::Json<TwoFactorStatusResponse>, ApiError> {
    state
        .auth_service
        .confirm_two_factor(&user.user_id, &body.code)?;
    Ok(web::Json(TwoFactorStatusResponse { enabled: true }))
}

#[post("/2fa/disable")]
pub async fn two_factor_disable(
    state: web::Data<AuthApiState>,
    user: AuthenticatedUser,
    body: web::Json<TwoFactorDisableRequest>,
) -> Result<web::Json<TwoFactorStatusResponse>, ApiError> {
    state
        .auth_service
        .disable_two_factor(&user.user_id, body.into_inner())?;
    Ok(web::Json(TwoFactorStatusResponse { enabled: false }))
}

// ── Sessions ──────────────────────────────────────────────────────────────────

#[get("/sessions")]
pub async fn list_sessions(
    state: web::Data<AuthApiState>,
    user: AuthenticatedUser,
) -> Result<web::Json<SessionListResponse>, ApiError> {
    let result = state.auth_service.list_sessions(&user.user_id)?;
    Ok(web::Json(result))
}

#[delete("/sessions/{session_id}")]
pub async fn revoke_session(
    state: web::Data<AuthApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
) -> Result<actix_web::HttpResponse, ApiError> {
    let session_id = path.into_inner();
    state.auth_service.revoke_session(&user.user_id, &session_id)?;
    Ok(actix_web::HttpResponse::NoContent().finish())
}

#[delete("/sessions")]
pub async fn revoke_all_sessions(
    state: web::Data<AuthApiState>,
    user: AuthenticatedUser,
) -> Result<actix_web::HttpResponse, ApiError> {
    state.auth_service.revoke_all_sessions(&user.user_id)?;
    Ok(actix_web::HttpResponse::NoContent().finish())
}

// ── Admin ─────────────────────────────────────────────────────────────────────

fn require_admin(user: &AuthenticatedUser) -> Result<(), ApiError> {
    if !user.is_admin {
        Err(ApiError::forbidden("Admin access required"))
    } else {
        Ok(())
    }
}

#[derive(Deserialize)]
pub struct AdminListQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[get("/admin/users")]
pub async fn admin_list_users(
    state: web::Data<AuthApiState>,
    user: AuthenticatedUser,
    query: web::Query<AdminListQuery>,
) -> Result<web::Json<AdminUserListResponse>, ApiError> {
    require_admin(&user)?;
    let result = state.auth_service.admin_list_users(
        query.page.unwrap_or(1),
        query.page_size.unwrap_or(20),
    )?;
    Ok(web::Json(result))
}

#[get("/admin/users/{user_id}")]
pub async fn admin_get_user(
    state: web::Data<AuthApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
) -> Result<web::Json<AdminUserResponse>, ApiError> {
    require_admin(&user)?;
    let target_id = path.into_inner();
    let result = state.auth_service.admin_get_user(&target_id)?;
    Ok(web::Json(result))
}

#[patch("/admin/users/{user_id}")]
pub async fn admin_update_user(
    state: web::Data<AuthApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
    body: web::Json<AdminUpdateUserRequest>,
) -> Result<web::Json<AdminUserResponse>, ApiError> {
    require_admin(&user)?;
    let target_id = path.into_inner();
    let result = state
        .auth_service
        .admin_update_user(&target_id, body.into_inner())?;
    Ok(web::Json(result))
}

#[delete("/admin/users/{user_id}")]
pub async fn admin_delete_user(
    state: web::Data<AuthApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
) -> Result<actix_web::HttpResponse, ApiError> {
    require_admin(&user)?;
    let target_id = path.into_inner();
    state.auth_service.admin_delete_user(&target_id)?;
    Ok(actix_web::HttpResponse::NoContent().finish())
}

// ── Route Configuration ───────────────────────────────────────────────────────

pub fn configure(conf: &mut web::ServiceConfig) {
    conf.service(
        web::scope("/auth")
            .service(register)
            .service(login)
            .service(refresh)
            .service(me)
            .service(lookup_user_by_email)
            .service(get_user_by_id)
            .service(two_factor_status)
            .service(two_factor_enroll)
            .service(two_factor_confirm)
            .service(two_factor_disable)
            .service(list_sessions)
            .service(revoke_session)
            .service(revoke_all_sessions),
    )
    .service(
        web::scope("/admin")
            .service(admin_list_users)
            .service(admin_get_user)
            .service(admin_update_user)
            .service(admin_delete_user),
    );
}

#[derive(OpenApi)]
#[openapi(
    paths(),
    components(schemas(RegisterRequest, LoginRequest, RefreshRequest, AuthResponse, LoginResponse, RegisterResponse, UserProfileResponse, UserLookupResponse)),
    tags((name = "auth", description = "Authentication endpoints"))
)]
pub struct AuthApiDoc;
