use serde::{Deserialize, Serialize};

fn default_timeout() -> i32 {
    30
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateJobRequest {
    pub job_type: String,
    pub payload: serde_json::Value,
    #[serde(default = "default_timeout")]
    pub timeout_secs: i32,
}

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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateJobStatusRequest {
    /// "C" (completed) or "E" (error)
    pub status: String,
    pub error_message: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterWorkerRequest {
    pub callback_url: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterWorkerResponse {
    pub worker_id: String,
}

#[derive(Debug, Deserialize)]
pub struct PendingJobsQuery {
    pub limit: Option<i64>,
}
