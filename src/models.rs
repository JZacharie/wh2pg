use serde::{Deserialize, Serialize};

use chrono::{DateTime, Utc};


#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub event: String,
    pub data: serde_json::Value,
    #[serde(default = "Utc::now")]
    pub timestamp: DateTime<Utc>,
}


#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub database: String,
    pub uptime_seconds: u64,
}

#[derive(Debug, Serialize)]
pub struct WebhookResponse {
    pub status: String,
    pub id: i32,
    pub message: String,
}
