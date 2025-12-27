use actix_web::{web, HttpResponse, Responder, HttpRequest};
use sqlx::{PgPool, Row};
use crate::trivy_models::TrivyWebhookPayload;
use tracing::{info, error, warn, instrument};
use serde_json::json;

/// Initialize Trivy-specific database schemas
pub async fn init_trivy_schemas(pool: &PgPool) -> Result<(), sqlx::Error> {
    info!("Initializing Trivy database schemas...");

    // Vulnerability Reports Table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS trivy_vulnerability_reports (
            id SERIAL PRIMARY KEY,
            report_uid VARCHAR(255) UNIQUE NOT NULL,
            name VARCHAR(255) NOT NULL,
            namespace VARCHAR(255),
            scanner_name VARCHAR(100),
            scanner_version VARCHAR(50),
            image_repository VARCHAR(500),
            image_tag VARCHAR(255),
            image_digest VARCHAR(255),
            critical_count INTEGER DEFAULT 0,
            high_count INTEGER DEFAULT 0,
            medium_count INTEGER DEFAULT 0,
            low_count INTEGER DEFAULT 0,
            unknown_count INTEGER DEFAULT 0,
            full_report JSONB NOT NULL,
            received_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            source_ip VARCHAR(45),
            headers JSONB
        );
        "#
    )
    .execute(pool)
    .await?;

    // Config Audit Reports Table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS trivy_configaudit_reports (
            id SERIAL PRIMARY KEY,
            report_uid VARCHAR(255) UNIQUE NOT NULL,
            name VARCHAR(255) NOT NULL,
            namespace VARCHAR(255),
            scanner_name VARCHAR(100),
            scanner_version VARCHAR(50),
            critical_count INTEGER DEFAULT 0,
            high_count INTEGER DEFAULT 0,
            medium_count INTEGER DEFAULT 0,
            low_count INTEGER DEFAULT 0,
            full_report JSONB NOT NULL,
            received_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            source_ip VARCHAR(45),
            headers JSONB
        );
        "#
    )
    .execute(pool)
    .await?;

    // RBAC Assessment Reports Table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS trivy_rbac_reports (
            id SERIAL PRIMARY KEY,
            report_uid VARCHAR(255) UNIQUE NOT NULL,
            name VARCHAR(255) NOT NULL,
            namespace VARCHAR(255),
            scanner_name VARCHAR(100),
            scanner_version VARCHAR(50),
            critical_count INTEGER DEFAULT 0,
            high_count INTEGER DEFAULT 0,
            medium_count INTEGER DEFAULT 0,
            low_count INTEGER DEFAULT 0,
            full_report JSONB NOT NULL,
            received_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            source_ip VARCHAR(45),
            headers JSONB
        );
        "#
    )
    .execute(pool)
    .await?;

    // Exposed Secrets Reports Table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS trivy_secret_reports (
            id SERIAL PRIMARY KEY,
            report_uid VARCHAR(255) UNIQUE NOT NULL,
            name VARCHAR(255) NOT NULL,
            namespace VARCHAR(255),
            scanner_name VARCHAR(100),
            scanner_version VARCHAR(50),
            critical_count INTEGER DEFAULT 0,
            high_count INTEGER DEFAULT 0,
            medium_count INTEGER DEFAULT 0,
            low_count INTEGER DEFAULT 0,
            full_report JSONB NOT NULL,
            received_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            source_ip VARCHAR(45),
            headers JSONB
        );
        "#
    )
    .execute(pool)
    .await?;

    // Cluster Compliance Reports Table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS trivy_compliance_reports (
            id SERIAL PRIMARY KEY,
            report_uid VARCHAR(255) UNIQUE NOT NULL,
            name VARCHAR(255) NOT NULL,
            compliance_title VARCHAR(255),
            pass_count INTEGER DEFAULT 0,
            fail_count INTEGER DEFAULT 0,
            full_report JSONB NOT NULL,
            received_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            source_ip VARCHAR(45),
            headers JSONB
        );
        "#
    )
    .execute(pool)
    .await?;

    // Create indexes for better query performance
    let indexes = vec![
        "CREATE INDEX IF NOT EXISTS idx_vuln_namespace ON trivy_vulnerability_reports(namespace);",
        "CREATE INDEX IF NOT EXISTS idx_vuln_received ON trivy_vulnerability_reports(received_at);",
        "CREATE INDEX IF NOT EXISTS idx_vuln_severity ON trivy_vulnerability_reports(critical_count, high_count);",
        "CREATE INDEX IF NOT EXISTS idx_config_namespace ON trivy_configaudit_reports(namespace);",
        "CREATE INDEX IF NOT EXISTS idx_config_received ON trivy_configaudit_reports(received_at);",
        "CREATE INDEX IF NOT EXISTS idx_rbac_namespace ON trivy_rbac_reports(namespace);",
        "CREATE INDEX IF NOT EXISTS idx_rbac_received ON trivy_rbac_reports(received_at);",
        "CREATE INDEX IF NOT EXISTS idx_secret_namespace ON trivy_secret_reports(namespace);",
        "CREATE INDEX IF NOT EXISTS idx_secret_received ON trivy_secret_reports(received_at);",
        "CREATE INDEX IF NOT EXISTS idx_compliance_received ON trivy_compliance_reports(received_at);",
    ];

    for index_sql in indexes {
        sqlx::query(index_sql).execute(pool).await?;
    }

    info!("Trivy database schemas initialized successfully");
    Ok(())
}

#[instrument(skip(pool, payload, req))]
pub async fn receive_trivy_webhook(
    pool: web::Data<PgPool>,
    payload: web::Json<TrivyWebhookPayload>,
    req: HttpRequest,
) -> impl Responder {
    let peer_addr = req.peer_addr().map(|a| a.to_string());
    
    // Extract headers
    let headers: serde_json::Value = req.headers()
        .iter()
        .map(|(k, v)| (k.as_str(), v.to_str().unwrap_or("")))
        .collect();

    info!("Received Trivy {} report: {} (UID: {})", 
          payload.report_type, payload.name, payload.uid);

    let result = match payload.report_type.as_str() {
        "VulnerabilityReport" => store_vulnerability_report(pool.get_ref(), &payload, peer_addr, headers).await,
        "ConfigAuditReport" => store_configaudit_report(pool.get_ref(), &payload, peer_addr, headers).await,
        "RbacAssessmentReport" => store_rbac_report(pool.get_ref(), &payload, peer_addr, headers).await,
        "ExposedSecretReport" => store_secret_report(pool.get_ref(), &payload, peer_addr, headers).await,
        "ClusterComplianceReport" => store_compliance_report(pool.get_ref(), &payload, peer_addr, headers).await,
        _ => {
            warn!("Unknown Trivy report type: {}", payload.report_type);
            return HttpResponse::BadRequest().json(json!({
                "status": "error",
                "message": format!("Unknown report type: {}", payload.report_type)
            }));
        }
    };

    match result {
        Ok(id) => {
            info!("Trivy report stored with ID: {}", id);
            HttpResponse::Ok().json(json!({
                "status": "success",
                "id": id,
                "report_type": payload.report_type,
                "message": "Trivy report received and stored"
            }))
        },
        Err(e) => {
            error!("Failed to store Trivy report: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to store Trivy report"
            }))
        }
    }
}

async fn store_vulnerability_report(
    pool: &PgPool,
    payload: &TrivyWebhookPayload,
    source_ip: Option<String>,
    headers: serde_json::Value,
) -> Result<i32, sqlx::Error> {
    let report = &payload.report;
    
    let scanner_name = report["scanner"]["name"].as_str().unwrap_or("unknown");
    let scanner_version = report["scanner"]["version"].as_str().unwrap_or("unknown");
    let image_repo = report["artifact"]["repository"].as_str().unwrap_or("");
    let image_tag = report["artifact"]["tag"].as_str();
    let image_digest = report["artifact"]["digest"].as_str();
    
    let summary = &report["summary"];
    let critical = summary["criticalCount"].as_i64().unwrap_or(0) as i32;
    let high = summary["highCount"].as_i64().unwrap_or(0) as i32;
    let medium = summary["mediumCount"].as_i64().unwrap_or(0) as i32;
    let low = summary["lowCount"].as_i64().unwrap_or(0) as i32;
    let unknown = summary["unknownCount"].as_i64().unwrap_or(0) as i32;

    let row = sqlx::query(
        r#"
        INSERT INTO trivy_vulnerability_reports 
        (report_uid, name, namespace, scanner_name, scanner_version, 
         image_repository, image_tag, image_digest,
         critical_count, high_count, medium_count, low_count, unknown_count,
         full_report, source_ip, headers)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
        ON CONFLICT (report_uid) DO UPDATE SET
            critical_count = EXCLUDED.critical_count,
            high_count = EXCLUDED.high_count,
            medium_count = EXCLUDED.medium_count,
            low_count = EXCLUDED.low_count,
            unknown_count = EXCLUDED.unknown_count,
            full_report = EXCLUDED.full_report,
            received_at = NOW()
        RETURNING id
        "#
    )
    .bind(&payload.uid)
    .bind(&payload.name)
    .bind(&payload.namespace)
    .bind(scanner_name)
    .bind(scanner_version)
    .bind(image_repo)
    .bind(image_tag)
    .bind(image_digest)
    .bind(critical)
    .bind(high)
    .bind(medium)
    .bind(low)
    .bind(unknown)
    .bind(report)
    .bind(source_ip)
    .bind(headers)
    .fetch_one(pool)
    .await?;

    Ok(row.try_get("id").unwrap_or(0))
}

async fn store_configaudit_report(
    pool: &PgPool,
    payload: &TrivyWebhookPayload,
    source_ip: Option<String>,
    headers: serde_json::Value,
) -> Result<i32, sqlx::Error> {
    let report = &payload.report;
    
    let scanner_name = report["scanner"]["name"].as_str().unwrap_or("unknown");
    let scanner_version = report["scanner"]["version"].as_str().unwrap_or("unknown");
    
    let summary = &report["summary"];
    let critical = summary["criticalCount"].as_i64().unwrap_or(0) as i32;
    let high = summary["highCount"].as_i64().unwrap_or(0) as i32;
    let medium = summary["mediumCount"].as_i64().unwrap_or(0) as i32;
    let low = summary["lowCount"].as_i64().unwrap_or(0) as i32;

    let row = sqlx::query(
        r#"
        INSERT INTO trivy_configaudit_reports 
        (report_uid, name, namespace, scanner_name, scanner_version,
         critical_count, high_count, medium_count, low_count,
         full_report, source_ip, headers)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        ON CONFLICT (report_uid) DO UPDATE SET
            critical_count = EXCLUDED.critical_count,
            high_count = EXCLUDED.high_count,
            medium_count = EXCLUDED.medium_count,
            low_count = EXCLUDED.low_count,
            full_report = EXCLUDED.full_report,
            received_at = NOW()
        RETURNING id
        "#
    )
    .bind(&payload.uid)
    .bind(&payload.name)
    .bind(&payload.namespace)
    .bind(scanner_name)
    .bind(scanner_version)
    .bind(critical)
    .bind(high)
    .bind(medium)
    .bind(low)
    .bind(report)
    .bind(source_ip)
    .bind(headers)
    .fetch_one(pool)
    .await?;

    Ok(row.try_get("id").unwrap_or(0))
}

async fn store_rbac_report(
    pool: &PgPool,
    payload: &TrivyWebhookPayload,
    source_ip: Option<String>,
    headers: serde_json::Value,
) -> Result<i32, sqlx::Error> {
    let report = &payload.report;
    
    let scanner_name = report["scanner"]["name"].as_str().unwrap_or("unknown");
    let scanner_version = report["scanner"]["version"].as_str().unwrap_or("unknown");
    
    let summary = &report["summary"];
    let critical = summary["criticalCount"].as_i64().unwrap_or(0) as i32;
    let high = summary["highCount"].as_i64().unwrap_or(0) as i32;
    let medium = summary["mediumCount"].as_i64().unwrap_or(0) as i32;
    let low = summary["lowCount"].as_i64().unwrap_or(0) as i32;

    let row = sqlx::query(
        r#"
        INSERT INTO trivy_rbac_reports 
        (report_uid, name, namespace, scanner_name, scanner_version,
         critical_count, high_count, medium_count, low_count,
         full_report, source_ip, headers)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        ON CONFLICT (report_uid) DO UPDATE SET
            critical_count = EXCLUDED.critical_count,
            high_count = EXCLUDED.high_count,
            medium_count = EXCLUDED.medium_count,
            low_count = EXCLUDED.low_count,
            full_report = EXCLUDED.full_report,
            received_at = NOW()
        RETURNING id
        "#
    )
    .bind(&payload.uid)
    .bind(&payload.name)
    .bind(&payload.namespace)
    .bind(scanner_name)
    .bind(scanner_version)
    .bind(critical)
    .bind(high)
    .bind(medium)
    .bind(low)
    .bind(report)
    .bind(source_ip)
    .bind(headers)
    .fetch_one(pool)
    .await?;

    Ok(row.try_get("id").unwrap_or(0))
}

async fn store_secret_report(
    pool: &PgPool,
    payload: &TrivyWebhookPayload,
    source_ip: Option<String>,
    headers: serde_json::Value,
) -> Result<i32, sqlx::Error> {
    let report = &payload.report;
    
    let scanner_name = report["scanner"]["name"].as_str().unwrap_or("unknown");
    let scanner_version = report["scanner"]["version"].as_str().unwrap_or("unknown");
    
    let summary = &report["summary"];
    let critical = summary["criticalCount"].as_i64().unwrap_or(0) as i32;
    let high = summary["highCount"].as_i64().unwrap_or(0) as i32;
    let medium = summary["mediumCount"].as_i64().unwrap_or(0) as i32;
    let low = summary["lowCount"].as_i64().unwrap_or(0) as i32;

    let row = sqlx::query(
        r#"
        INSERT INTO trivy_secret_reports 
        (report_uid, name, namespace, scanner_name, scanner_version,
         critical_count, high_count, medium_count, low_count,
         full_report, source_ip, headers)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        ON CONFLICT (report_uid) DO UPDATE SET
            critical_count = EXCLUDED.critical_count,
            high_count = EXCLUDED.high_count,
            medium_count = EXCLUDED.medium_count,
            low_count = EXCLUDED.low_count,
            full_report = EXCLUDED.full_report,
            received_at = NOW()
        RETURNING id
        "#
    )
    .bind(&payload.uid)
    .bind(&payload.name)
    .bind(&payload.namespace)
    .bind(scanner_name)
    .bind(scanner_version)
    .bind(critical)
    .bind(high)
    .bind(medium)
    .bind(low)
    .bind(report)
    .bind(source_ip)
    .bind(headers)
    .fetch_one(pool)
    .await?;

    Ok(row.try_get("id").unwrap_or(0))
}

async fn store_compliance_report(
    pool: &PgPool,
    payload: &TrivyWebhookPayload,
    source_ip: Option<String>,
    headers: serde_json::Value,
) -> Result<i32, sqlx::Error> {
    let report = &payload.report;
    
    let compliance_title = report["title"].as_str().unwrap_or("");
    
    let summary = &report["summary"];
    let pass_count = summary["passCount"].as_i64().unwrap_or(0) as i32;
    let fail_count = summary["failCount"].as_i64().unwrap_or(0) as i32;

    let row = sqlx::query(
        r#"
        INSERT INTO trivy_compliance_reports 
        (report_uid, name, compliance_title, pass_count, fail_count,
         full_report, source_ip, headers)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (report_uid) DO UPDATE SET
            pass_count = EXCLUDED.pass_count,
            fail_count = EXCLUDED.fail_count,
            full_report = EXCLUDED.full_report,
            received_at = NOW()
        RETURNING id
        "#
    )
    .bind(&payload.uid)
    .bind(&payload.name)
    .bind(compliance_title)
    .bind(pass_count)
    .bind(fail_count)
    .bind(report)
    .bind(source_ip)
    .bind(headers)
    .fetch_one(pool)
    .await?;

    Ok(row.try_get("id").unwrap_or(0))
}
