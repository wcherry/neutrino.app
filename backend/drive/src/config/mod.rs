use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub jwt_secret: String,
    pub jwt_access_expiry_secs: u64,
    pub jwt_refresh_expiry_secs: u64,
    pub log_level: String,
    pub storage_path: String,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "neutrino.db".to_string());

        let port = env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .map_err(|e| format!("Invalid PORT: {}", e))?;

        let jwt_secret =
            env::var("JWT_SECRET").map_err(|_| "JWT_SECRET environment variable is required")?;

        if jwt_secret.is_empty() {
            return Err("JWT_SECRET must not be empty".to_string());
        }

        let jwt_access_expiry_secs = env::var("JWT_ACCESS_EXPIRY_SECS")
            .unwrap_or_else(|_| "900".to_string())
            .parse::<u64>()
            .map_err(|e| format!("Invalid JWT_ACCESS_EXPIRY_SECS: {}", e))?;

        let jwt_refresh_expiry_secs = env::var("JWT_REFRESH_EXPIRY_SECS")
            .unwrap_or_else(|_| "604800".to_string())
            .parse::<u64>()
            .map_err(|e| format!("Invalid JWT_REFRESH_EXPIRY_SECS: {}", e))?;

        let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());

        let storage_path =
            env::var("STORAGE_PATH").unwrap_or_else(|_| "./storage".to_string());

        Ok(Config {
            database_url,
            port,
            jwt_secret,
            jwt_access_expiry_secs,
            jwt_refresh_expiry_secs,
            log_level,
            storage_path,
        })
    }
}
