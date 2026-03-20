use serde::{Deserialize, Serialize};
use tracing::{debug};

/// A job dispatched from the drive service.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobResponse {
    pub id: String,
    pub job_type: String,
    pub payload: serde_json::Value,
    pub status: String,
    pub error_message: Option<String>,
    pub worker_id: Option<String>,
    pub timeout_secs: i32,
    pub started_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

pub struct DriveJobsClient {
    base_url: String,
    worker_secret: String,
    worker_id: String,
    http: reqwest::Client,
}

impl DriveJobsClient {
    pub fn new(base_url: String, worker_secret: String, worker_id: String) -> Self {
        DriveJobsClient {
            base_url,
            worker_secret,
            worker_id,
            http: reqwest::Client::new(),
        }
    }

    fn auth_header(&self) -> String {
        format!("Bearer {}", self.worker_secret)
    }

    pub fn worker_secret(&self) -> &str {
        &self.worker_secret
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Enqueue a new job via the drive jobs API.
    pub async fn enqueue_job(
        &self,
        job_type: &str,
        payload: serde_json::Value,
        timeout_secs: i64,
        secret: &str,
    ) -> Result<(), String> {
        let url = format!("{}/api/v1/jobs", self.base_url);
        let body = serde_json::json!({
            "jobType": job_type,
            "payload": payload,
            "timeoutSecs": timeout_secs,
        });
        let resp = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", secret))
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Enqueue job failed: {}", e))?;
        if resp.status().is_success() {
            Ok(())
        } else {
            let s = resp.status();
            let b = resp.text().await.unwrap_or_default();
            Err(format!("Enqueue job returned ({}): {}", s, b))
        }
    }

    /// Register this worker with drive and return the assigned worker ID.
    pub async fn register(
        base_url: &str,
        worker_secret: &str,
        callback_url: &str,
    ) -> Result<String, String> {
        let http = reqwest::Client::new();
        let url = format!("{}/api/v1/jobs/workers", base_url);
        let resp = http
            .post(&url)
            .header("Authorization", format!("Bearer {}", worker_secret))
            .json(&serde_json::json!({ "callbackUrl": callback_url }))
            .send()
            .await
            .map_err(|e| format!("Register request failed: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Register failed ({}): {}", status, body));
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Resp {
            worker_id: String,
        }
        let body: Resp = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse register response: {}", e))?;
        Ok(body.worker_id)
    }

    /// Deregister this worker from drive.
    pub async fn deregister(&self) -> Result<(), String> {
        let url = format!("{}/api/v1/jobs/workers/{}", self.base_url, self.worker_id);
        self.http
            .delete(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| format!("Deregister failed: {}", e))?;
        Ok(())
    }

    /// Pull up to `limit` pending jobs from drive (claims them immediately).
    pub async fn pull_pending(&self, limit: i64) -> Result<Vec<JobResponse>, String> {
        let url = format!("{}/api/v1/jobs/pending?limit={}", self.base_url, limit);
        let resp = self
            .http
            .get(&url)
            .header("Authorization", self.auth_header())
            .header("X-Worker-Id", &self.worker_id)
            .send()
            .await
            .map_err(|e| format!("Pull pending failed: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Pull pending returned ({}): {}", status, body));
        }

        resp.json::<Vec<JobResponse>>()
            .await
            .map_err(|e| format!("Failed to parse pending jobs: {}", e))
    }

    /// Fetch the raw bytes of a file (so the worker can generate a thumbnail).
    pub async fn get_file_content(&self, file_id: &str) -> Result<(Vec<u8>, String), String> {
        let url = format!("{}/api/v1/jobs/file-content/{}",
            self.base_url, file_id
        );
        debug!("Getting file: {}", &url);
        let resp = self
            .http
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| format!("Get file content failed: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Get file content returned ({}): {}", status, body));
        }

        let mime_type = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_owned();

        let bytes = resp
            .bytes()
            .await
            .map_err(|e| format!("Failed to read file bytes: {}", e))?
            .to_vec();

        Ok((bytes, mime_type))
    }

    /// Report job completed.
    pub async fn complete_job(&self, job_id: &str) -> Result<(), String> {
        self.update_status(job_id, "C", None).await
    }

    /// Report job failed with an error message.
    pub async fn fail_job(&self, job_id: &str, error: &str) -> Result<(), String> {
        self.update_status(job_id, "E", Some(error)).await
    }

    async fn update_status(
        &self,
        job_id: &str,
        status: &str,
        error_message: Option<&str>,
    ) -> Result<(), String> {
        let url = format!("{}/api/v1/jobs/{}/status", self.base_url, job_id);
        let body = serde_json::json!({
            "status": status,
            "errorMessage": error_message,
        });
        let resp = self
            .http
            .patch(&url)
            .header("Authorization", self.auth_header())
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Update status failed: {}", e))?;

        if !resp.status().is_success() {
            let s = resp.status();
            let b = resp.text().await.unwrap_or_default();
            return Err(format!("Update status returned ({}): {}", s, b));
        }
        Ok(())
    }
}
