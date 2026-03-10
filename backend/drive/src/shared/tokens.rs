use crate::shared::ApiError;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub exp: i64,
    pub iat: i64,
}

pub struct TokenService {
    secret: String,
}

impl TokenService {
    pub fn new(secret: String) -> Self {
        TokenService { secret }
    }

    pub fn validate_access_token(&self, token: &str) -> Result<Claims, ApiError> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|e| {
            log::warn!("Token validation failed: {:?}", e);
            ApiError::unauthorized("Invalid or expired token")
        })
    }
}
