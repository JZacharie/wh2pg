use sqlx::postgres::{PgPoolOptions, PgPool};
use std::env;
use tracing::info;

pub async fn init_pool() -> PgPool {
    // Try to get DATABASE_URL first, otherwise build it from individual components
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        let host = env::var("DB_HOST").expect("DB_HOST must be set");
        let port = env::var("DB_PORT").unwrap_or_else(|_| "5432".to_string());
        let database = env::var("DB_NAME").expect("DB_NAME must be set");
        let user = env::var("DB_USER").expect("DB_USER must be set");
        let password = env::var("DB_PASSWORD").expect("DB_PASSWORD must be set");
        let ssl_mode = env::var("DB_SSL_MODE").unwrap_or_else(|_| "prefer".to_string());
        
        format!(
            "postgresql://{}:{}@{}:{}/{}?sslmode={}",
            user, password, host, port, database, ssl_mode
        )
    });

    let pool_size = env::var("DB_POOL_SIZE")
        .unwrap_or_else(|_| "10".to_string())
        .parse::<u32>()
        .expect("DB_POOL_SIZE must be a number");

    info!("Connecting to database...");
    
    PgPoolOptions::new()
        .max_connections(pool_size)
        .connect(&database_url)
        .await
        .expect("Failed to create pool")
}

pub async fn init_db(pool: &PgPool) -> Result<(), sqlx::Error> {
    info!("Initializing database schema...");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS webhook_events (
            id SERIAL PRIMARY KEY,
            payload JSONB NOT NULL,
            received_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            source_ip VARCHAR(45),
            headers JSONB
        );
        "#
    )
    .execute(pool)
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_received_at ON webhook_events(received_at);")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_payload ON webhook_events USING GIN(payload);")
        .execute(pool)
        .await?;

    info!("Database schema initialized successfully");
    Ok(())
}
