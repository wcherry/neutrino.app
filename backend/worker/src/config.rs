use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    /// Base URL of the drive service (e.g. http://localhost:8882).
    pub drive_url: String,
    /// Base URL of the photos service (e.g. http://localhost:8084).
    pub photos_url: String,
    /// Shared secret for authenticating calls to the drive jobs API.
    pub worker_secret: String,
    /// Full URL that drive will POST to when dispatching a job
    /// (e.g. http://worker-host:9000/dispatch).
    pub callback_url: String,
    /// Port this worker listens on for callbacks.
    pub port: u16,
    /// Number of pending jobs to pull from drive on startup.
    pub initial_batch: i64,
    pub log_level: String,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let drive_url = env::var("DRIVE_URL")
            .unwrap_or_else(|_| "http://localhost:8882".to_string());

        let photos_url = env::var("PHOTOS_URL")
            .unwrap_or_else(|_| "http://localhost:8084".to_string());

        let worker_secret = env::var("WORKER_SECRET")
            .map_err(|_| "WORKER_SECRET environment variable is required")?;

        if worker_secret.is_empty() {
            return Err("WORKER_SECRET must not be empty".to_string());
        }

        let port = env::var("WORKER_PORT")
            .unwrap_or_else(|_| "9000".to_string())
            .parse::<u16>()
            .map_err(|e| format!("Invalid WORKER_PORT: {}", e))?;

        let callback_url = env::var("WORKER_CALLBACK_URL")
            .unwrap_or_else(|_| format!("http://localhost:{}/dispatch", port));

        let initial_batch = env::var("WORKER_INITIAL_BATCH")
            .unwrap_or_else(|_| "4".to_string())
            .parse::<i64>()
            .unwrap_or(4)
            .max(1);

        let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());

        Ok(Config {
            drive_url,
            photos_url,
            worker_secret,
            callback_url,
            port,
            initial_batch,
            log_level,
        })
    }
}
