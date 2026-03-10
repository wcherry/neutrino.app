use crate::auth::{
    dto::{AuthResponse, LoginRequest, RefreshRequest, RegisterRequest, RegisterResponse, UserProfileResponse},
    repository::{AuthRepository, NewRefreshToken, NewUser},
    tokens::{hash_token, TokenService},
};
use crate::shared::ApiError;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::Utc;
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
                log::error!("Password hashing error: {:?}", e);
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

    pub fn login(&self, req: LoginRequest) -> Result<AuthResponse, ApiError> {
        let user = self
            .repo
            .find_user_by_email(&req.email)?
            .ok_or_else(|| ApiError::unauthorized("Invalid email or password"))?;

        let parsed_hash = PasswordHash::new(&user.password_hash).map_err(|e| {
            log::error!("Password hash parse error: {:?}", e);
            ApiError::internal("Authentication error")
        })?;

        Argon2::default()
            .verify_password(req.password.as_bytes(), &parsed_hash)
            .map_err(|_| ApiError::unauthorized("Invalid email or password"))?;

        let access_token = self
            .token_service
            .generate_access_token(&user.id, &user.email)?;
        let (refresh_token_raw, expires_at) = self.token_service.generate_refresh_token()?;
        let token_hash = hash_token(&refresh_token_raw);

        let token_id = Uuid::new_v4().to_string();
        self.repo.create_refresh_token(NewRefreshToken {
            id: &token_id,
            user_id: &user.id,
            token_hash: &token_hash,
            expires_at,
        })?;

        Ok(AuthResponse {
            access_token,
            refresh_token: refresh_token_raw,
            token_type: "Bearer".to_string(),
            expires_in: self.token_service.access_expiry_secs(),
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

        let access_token = self
            .token_service
            .generate_access_token(&user.id, &user.email)?;
        let (new_refresh_token_raw, new_expires_at) = self.token_service.generate_refresh_token()?;
        let new_token_hash = hash_token(&new_refresh_token_raw);

        let token_id = Uuid::new_v4().to_string();
        self.repo.create_refresh_token(NewRefreshToken {
            id: &token_id,
            user_id: &user.id,
            token_hash: &new_token_hash,
            expires_at: new_expires_at,
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
        })
    }
}
