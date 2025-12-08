use actix_web::{web, HttpResponse, Responder, HttpRequest};
use sqlx::{PgPool, Row};
use crate::models::{WebhookPayload, WebhookResponse, HealthResponse};
use tracing::{info, error, instrument};
use std::time::SystemTime;
use serde_json::json;

// Global start time for uptime calculation
lazy_static::lazy_static! {
    static ref START_TIME: SystemTime = SystemTime::now();
}

#[instrument(skip(pool, payload, req))]
pub async fn receive_webhook(
    pool: web::Data<PgPool>,
    payload: web::Json<WebhookPayload>,
    req: HttpRequest,
) -> impl Responder {
    let peer_addr = req.peer_addr().map(|a| a.to_string());
    
    // Extract headers
    let headers: serde_json::Value = req.headers()
        .iter()
        .map(|(k, v)| (k.as_str(), v.to_str().unwrap_or("")))
        .collect();

    info!("Received webhook event: {}", payload.event);

    let result = sqlx::query(
        r#"
        INSERT INTO webhook_events (payload, source_ip, headers)
        VALUES ($1, $2, $3)
        RETURNING id
        "#
    )
    .bind(serde_json::to_value(&*payload).unwrap_or(json!({})))
    .bind(peer_addr)
    .bind(headers)
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(row) => {
            let id: i32 = row.try_get("id").unwrap_or(0);
            info!("Webhook stored with ID: {}", id);
            HttpResponse::Ok().json(WebhookResponse {
                status: "success".to_string(),
                id,
                message: "Webhook received and stored".to_string(),
            })
        },
        Err(e) => {
            error!("Failed to store webhook: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to store webhook"
            }))
        }
    }
}

pub async fn health_check(pool: web::Data<PgPool>) -> impl Responder {
    let db_status = match sqlx::query("SELECT 1").execute(pool.get_ref()).await {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    };

    let uptime = SystemTime::now()
        .duration_since(*START_TIME)
        .unwrap_or_default()
        .as_secs();

    HttpResponse::Ok().json(HealthResponse {
        status: "healthy".to_string(),
        database: db_status.to_string(),
        uptime_seconds: uptime,
    })
}

use prometheus::{Encoder, TextEncoder};

pub async fn metrics() -> impl Responder {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    
    if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
        error!("Failed to encode metrics: {}", e);
        return HttpResponse::InternalServerError().body("Failed to encode metrics");
    }

    HttpResponse::Ok()
        .content_type(encoder.format_type())
        .body(buffer)
}
