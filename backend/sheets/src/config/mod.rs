use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub jwt_secret: String,
    pub log_level: String,
    pub drive_base_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let database_url = env::var("SHEETS_DATABASE_URL")
            .or_else(|_| env::var("DATABASE_URL"))
            .unwrap_or_else(|_| "sheets.db".to_string());

        let port = env::var("SHEETS_PORT")
            .or_else(|_| env::var("PORT"))
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .map_err(|e| format!("Invalid PORT: {}", e))?;

        let jwt_secret =
            env::var("JWT_SECRET").map_err(|_| "JWT_SECRET environment variable is required")?;

        if jwt_secret.is_empty() {
            return Err("JWT_SECRET must not be empty".to_string());
        }

        let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());

        let drive_base_url =
            env::var("DRIVE_URL").unwrap_or_else(|_| "http://localhost:8882".to_string());

        Ok(Config {
            database_url,
            port,
            jwt_secret,
            log_level,
            drive_base_url,
        })
    }
}
