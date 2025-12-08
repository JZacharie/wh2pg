use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};


#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub event: String,
    pub data: serde_json::Value,
    #[serde(default = "Utc::now")]
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, FromRow, Serialize)]
pub struct WebhookEvent {
    pub id: i32,
    pub payload: serde_json::Value,
    pub received_at: DateTime<Utc>,
    pub source_ip: Option<String>,
    pub headers: serde_json::Value,
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
