use crate::auth::{
    dto::{
        AdminUpdateUserRequest, AdminUserListResponse, AdminUserResponse, AuthResponse,
        LoginResponse, RefreshRequest, RegisterRequest, RegisterResponse, SessionListResponse,
        SessionResponse, TwoFactorDisableRequest, TwoFactorEnrollResponse, TwoFactorStatusResponse,
        UserLookupResponse, UserProfileResponse,
    },
    repository::{AuthRepository, NewRefreshToken, NewTotpBackupCode, NewUser},
    totp::{generate_otpauth_uri, generate_secret, verify_totp},
    tokens::{hash_token, TokenService},
};
use crate::auth::dto::LoginRequest;
use crate::common::ApiError;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::Utc;
use rand::Rng;
use std::sync::Arc;
use uuid::Uuid;

pub struct AuthService {
    repo: Arc<AuthRepository>,
    token_service: Arc<TokenService>,
}

impl AuthService {
    pub fn new(repo: Arc<AuthRepository>, token_service: Arc<TokenService>) -> Self {
        AuthService {
            repo,
            token_service,
        }
    }

    pub fn register(&self, req: RegisterRequest) -> Result<RegisterResponse, ApiError> {
        if req.email.is_empty() {
            return Err(ApiError::bad_request("Email is required"));
        }
        if req.password.len() < 8 {
            return Err(ApiError::bad_request(
                "Password must be at least 8 characters",
            ));
        }
        if req.name.is_empty() {
            return Err(ApiError::bad_request("Name is required"));
        }

        if self.repo.find_user_by_email(&req.email)?.is_some() {
            return Err(ApiError::conflict("Email already registered"));
        }

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(req.password.as_bytes(), &salt)
            .map_err(|e| {
                tracing::error!("Password hashing error: {:?}", e);
                ApiError::internal("Failed to hash password")
            })?
            .to_string();

        let user_id = Uuid::new_v4().to_string();
        let new_user = NewUser {
            id: &user_id,
            email: &req.email,
            name: &req.name,
            password_hash: &password_hash,
        };

        let user = self.repo.create_user(new_user)?;

        Ok(RegisterResponse {
            id: user.id,
            email: user.email,
            name: user.name,
        })
    }

    pub fn login(
        &self,
        req: LoginRequest,
        device_name: Option<String>,
        user_agent: Option<String>,
        ip_address: Option<String>,
    ) -> Result<LoginResponse, ApiError> {
        let user = self
            .repo
            .find_user_by_email(&req.email)?
            .ok_or_else(|| ApiError::unauthorized("Invalid email or password"))?;

        let parsed_hash = PasswordHash::new(&user.password_hash).map_err(|e| {
            tracing::error!("Password hash parse error: {:?}", e);
            ApiError::internal("Authentication error")
        })?;

        Argon2::default()
            .verify_password(req.password.as_bytes(), &parsed_hash)
            .map_err(|_| ApiError::unauthorized("Invalid email or password"))?;

        // Check if 2FA is required
        if user.totp_enabled == 1 {
            match &req.totp_code {
                None => {
                    return Ok(LoginResponse {
                        auth: None,
                        requires_two_factor: true,
                    });
                }
                Some(code) => {
                    let secret = user.totp_secret.as_deref().ok_or_else(|| {
                        ApiError::internal("TOTP configuration error")
                    })?;
                    if !verify_totp(secret, code) {
                        return Err(ApiError::unauthorized("Invalid two-factor code"));
                    }
                }
            }
        }

        let is_admin = user.role == "admin";
        let access_token = self
            .token_service
            .generate_access_token_with_admin(&user.id, &user.email, is_admin)?;
        let (refresh_token_raw, expires_at) = self.token_service.generate_refresh_token()?;
        let token_hash = hash_token(&refresh_token_raw);

        let token_id = Uuid::new_v4().to_string();
        self.repo.create_refresh_token(NewRefreshToken {
            id: &token_id,
            user_id: &user.id,
            token_hash: &token_hash,
            expires_at,
            device_name: device_name.as_deref(),
            user_agent: user_agent.as_deref(),
            ip_address: ip_address.as_deref(),
        })?;

        Ok(LoginResponse {
            auth: Some(AuthResponse {
                access_token,
                refresh_token: refresh_token_raw,
                token_type: "Bearer".to_string(),
                expires_in: self.token_service.access_expiry_secs(),
            }),
            requires_two_factor: false,
        })
    }

    pub fn refresh(&self, req: RefreshRequest) -> Result<AuthResponse, ApiError> {
        let token_hash = hash_token(&req.refresh_token);

        let stored_token = self
            .repo
            .find_refresh_token_by_hash(&token_hash)?
            .ok_or_else(|| ApiError::unauthorized("Invalid refresh token"))?;

        let now = Utc::now().naive_utc();
        if stored_token.expires_at < now {
            let _ = self.repo.delete_refresh_token(&stored_token.id);
            return Err(ApiError::unauthorized("Refresh token has expired"));
        }

        let user = self
            .repo
            .find_user_by_id(&stored_token.user_id)?
            .ok_or_else(|| ApiError::unauthorized("User not found"))?;

        self.repo.delete_refresh_token(&stored_token.id)?;

        let is_admin = user.role == "admin";
        let access_token = self
            .token_service
            .generate_access_token_with_admin(&user.id, &user.email, is_admin)?;
        let (new_refresh_token_raw, new_expires_at) = self.token_service.generate_refresh_token()?;
        let new_token_hash = hash_token(&new_refresh_token_raw);

        let token_id = Uuid::new_v4().to_string();
        self.repo.create_refresh_token(NewRefreshToken {
            id: &token_id,
            user_id: &user.id,
            token_hash: &new_token_hash,
            expires_at: new_expires_at,
            device_name: stored_token.device_name.as_deref(),
            user_agent: stored_token.user_agent.as_deref(),
            ip_address: stored_token.ip_address.as_deref(),
        })?;

        Ok(AuthResponse {
            access_token,
            refresh_token: new_refresh_token_raw,
            token_type: "Bearer".to_string(),
            expires_in: self.token_service.access_expiry_secs(),
        })
    }

    pub fn get_profile(&self, user_id: &str) -> Result<UserProfileResponse, ApiError> {
        let user = self
            .repo
            .find_user_by_id(user_id)?
            .ok_or_else(|| ApiError::not_found("User not found"))?;
        Ok(UserProfileResponse {
            id: user.id,
            email: user.email,
            name: user.name,
            created_at: user.created_at,
            role: user.role,
            totp_enabled: user.totp_enabled == 1,
        })
    }

    pub fn lookup_user_by_email(&self, email: &str) -> Result<Option<UserLookupResponse>, ApiError> {
        let user = self.repo.find_user_by_email(email)?;
        Ok(user.map(|u| UserLookupResponse {
            id: u.id,
            email: u.email,
            name: u.name,
        }))
    }

    pub fn get_user_by_id(&self, user_id: &str) -> Result<Option<UserLookupResponse>, ApiError> {
        let user = self.repo.find_user_by_id(user_id)?;
        Ok(user.map(|u| UserLookupResponse {
            id: u.id,
            email: u.email,
            name: u.name,
        }))
    }

    // ── 2FA ───────────────────────────────────────────────────────────────────

    pub fn get_two_factor_status(&self, user_id: &str) -> Result<TwoFactorStatusResponse, ApiError> {
        let user = self
            .repo
            .find_user_by_id(user_id)?
            .ok_or_else(|| ApiError::not_found("User not found"))?;
        Ok(TwoFactorStatusResponse {
            enabled: user.totp_enabled == 1,
        })
    }

    pub fn enroll_two_factor(&self, user_id: &str, email: &str) -> Result<TwoFactorEnrollResponse, ApiError> {
        let secret = generate_secret();
        let otpauth_uri = generate_otpauth_uri(&secret, email, "Neutrino")
            .map_err(|e| ApiError::internal(&format!("TOTP error: {e}")))?;

        // Store secret (not yet enabled)
        self.repo.update_user_totp(user_id, Some(&secret), false)?;

        // Generate 10 backup codes
        let mut rng = rand::thread_rng();
        let mut plaintext_codes = Vec::new();
        let mut db_codes = Vec::new();
        let argon2 = Argon2::default();

        for _ in 0..10 {
            let code: String = format!("{:08x}", rng.gen::<u32>());
            let salt = SaltString::generate(&mut OsRng);
            let hash = argon2
                .hash_password(code.as_bytes(), &salt)
                .map_err(|_| ApiError::internal("Failed to hash backup code"))?
                .to_string();
            plaintext_codes.push(code.clone());
            db_codes.push((Uuid::new_v4().to_string(), hash));
        }

        let new_codes: Vec<NewTotpBackupCode> = db_codes
            .iter()
            .map(|(id, hash)| NewTotpBackupCode {
                id: id.as_str(),
                user_id,
                code_hash: hash.as_str(),
            })
            .collect();

        self.repo.create_backup_codes(new_codes)?;

        Ok(TwoFactorEnrollResponse {
            otpauth_uri,
            secret: secret.clone(),
            backup_codes: plaintext_codes,
        })
    }

    pub fn confirm_two_factor(&self, user_id: &str, code: &str) -> Result<(), ApiError> {
        let user = self
            .repo
            .find_user_by_id(user_id)?
            .ok_or_else(|| ApiError::not_found("User not found"))?;

        let secret = user.totp_secret.as_deref().ok_or_else(|| {
            ApiError::bad_request("2FA enrollment not started")
        })?;

        if !verify_totp(secret, code) {
            return Err(ApiError::bad_request("Invalid verification code"));
        }

        self.repo.update_user_totp(user_id, Some(secret), true)?;
        Ok(())
    }

    pub fn disable_two_factor(
        &self,
        user_id: &str,
        req: TwoFactorDisableRequest,
    ) -> Result<(), ApiError> {
        let user = self
            .repo
            .find_user_by_id(user_id)?
            .ok_or_else(|| ApiError::not_found("User not found"))?;

        let parsed_hash = PasswordHash::new(&user.password_hash).map_err(|_| {
            ApiError::internal("Authentication error")
        })?;
        Argon2::default()
            .verify_password(req.password.as_bytes(), &parsed_hash)
            .map_err(|_| ApiError::unauthorized("Invalid password"))?;

        let secret = user.totp_secret.as_deref().ok_or_else(|| {
            ApiError::bad_request("2FA is not enabled")
        })?;
        if !verify_totp(secret, &req.code) {
            return Err(ApiError::unauthorized("Invalid two-factor code"));
        }

        self.repo.update_user_totp(user_id, None, false)?;
        Ok(())
    }

    // ── Sessions ──────────────────────────────────────────────────────────────

    pub fn list_sessions(&self, user_id: &str) -> Result<SessionListResponse, ApiError> {
        let tokens = self.repo.list_refresh_tokens_for_user(user_id)?;
        let sessions = tokens
            .into_iter()
            .map(|t| SessionResponse {
                id: t.id,
                device_name: t.device_name,
                user_agent: t.user_agent,
                ip_address: t.ip_address.map(|ip| anonymize_ip(&ip)),
                created_at: t.created_at,
                last_used_at: t.last_used_at,
            })
            .collect();
        Ok(SessionListResponse { sessions })
    }

    pub fn revoke_session(&self, user_id: &str, session_id: &str) -> Result<(), ApiError> {
        // Verify the session belongs to this user by checking the token list
        let tokens = self.repo.list_refresh_tokens_for_user(user_id)?;
        let belongs = tokens.iter().any(|t| t.id == session_id);
        if !belongs {
            return Err(ApiError::not_found("Session not found"));
        }
        self.repo.delete_refresh_token(session_id)
    }

    pub fn revoke_all_sessions(&self, user_id: &str) -> Result<(), ApiError> {
        self.repo.delete_all_refresh_tokens_for_user(user_id)
    }

    // ── Admin ─────────────────────────────────────────────────────────────────

    pub fn admin_list_users(
        &self,
        page: i64,
        page_size: i64,
    ) -> Result<AdminUserListResponse, ApiError> {
        let page_size = page_size.min(100).max(1);
        let page = page.max(1);
        let (users, total) = self.repo.list_users(page, page_size)?;
        let items = users
            .into_iter()
            .map(|u| AdminUserResponse {
                id: u.id,
                email: u.email,
                name: u.name,
                role: u.role,
                totp_enabled: u.totp_enabled == 1,
                created_at: u.created_at,
                deleted_at: u.deleted_at,
            })
            .collect();
        Ok(AdminUserListResponse {
            users: items,
            total,
            page,
            page_size,
        })
    }

    pub fn admin_get_user(&self, user_id: &str) -> Result<AdminUserResponse, ApiError> {
        let user = self
            .repo
            .find_user_by_id(user_id)?
            .ok_or_else(|| ApiError::not_found("User not found"))?;
        Ok(AdminUserResponse {
            id: user.id,
            email: user.email,
            name: user.name,
            role: user.role,
            totp_enabled: user.totp_enabled == 1,
            created_at: user.created_at,
            deleted_at: user.deleted_at,
        })
    }

    pub fn admin_update_user(
        &self,
        user_id: &str,
        req: AdminUpdateUserRequest,
    ) -> Result<AdminUserResponse, ApiError> {
        if let Some(ref role) = req.role {
            if role != "user" && role != "admin" {
                return Err(ApiError::bad_request("Role must be 'user' or 'admin'"));
            }
            self.repo.update_user_role(user_id, role)?;
        }
        if let Some(enabled) = req.totp_enabled {
            if !enabled {
                // Admin force-disabling 2FA
                self.repo.update_user_totp(user_id, None, false)?;
            }
        }
        self.admin_get_user(user_id)
    }

    pub fn admin_delete_user(&self, user_id: &str) -> Result<(), ApiError> {
        self.repo.soft_delete_user(user_id)
    }
}

fn anonymize_ip(ip: &str) -> String {
    // Strip last octet from IPv4 for privacy
    if let Some(pos) = ip.rfind('.') {
        format!("{}.xxx", &ip[..pos])
    } else {
        ip.to_string()
    }
}
