use sqlx::postgres::{PgPoolOptions, PgPool};
use std::env;
use tracing::{info, error};

pub async fn init_pool() -> PgPool {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

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
