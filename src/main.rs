use actix_web::{web, App, HttpServer, middleware};
use actix_web_opentelemetry::RequestTracing;
use dotenv::dotenv;
use opentelemetry::{global, KeyValue, trace::TracerProvider};
use opentelemetry_sdk::{
    trace::{self as sdktrace},
    Resource,
    propagation::TraceContextPropagator,
};
use std::env;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, Registry};

mod db;
mod handlers;
mod models;
mod trivy_models;
mod trivy_handlers;

fn init_telemetry() {
    // Set OpenTelemetry propagator
    global::set_text_map_propagator(TraceContextPropagator::new());

    // Initialize Prometheus exporter
    let registry = prometheus::Registry::new();
    let _exporter = opentelemetry_prometheus::exporter()
        .with_registry(registry.clone())
        .build()
        .expect("Failed to create prometheus exporter");
    
    // Initialize Tracing
    // Note: Stdout exporter removed to avoid dependency version mismatches.
    // In production, use OTLP exporter.
    let tracer = sdktrace::TracerProvider::builder()
        .with_config(sdktrace::Config::default().with_resource(Resource::new(vec![KeyValue::new("service.name", "wh2pg")])))
        .build();

    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer.tracer("wh2pg"));
    let subscriber = Registry::default()
        .with(tracing_subscriber::EnvFilter::new(
            env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer().json())
        .with(telemetry);

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set subscriber");
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    
    // Initialize telemetry (Tracing + Prometheus)
    init_telemetry();

    info!("Starting wh2pg service...");

    // Database initialization
    let pool = db::init_pool().await;
    
    // Run migrations/init schema
    if let Err(e) = db::init_db(&pool).await {
        panic!("Failed to initialize database: {}", e);
    }

    // Initialize Trivy-specific schemas
    if let Err(e) = trivy_handlers::init_trivy_schemas(&pool).await {
        panic!("Failed to initialize Trivy schemas: {}", e);
    }

    // Check if we should exit after initialization (useful for K8s init jobs)
    if env::var("RUN_INIT_ONLY").unwrap_or_else(|_| "false".to_string()) == "true" {
        info!("Database initialization complete. RUN_INIT_ONLY is true, exiting.");
        return Ok(());
    }

    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("{}:{}", host, port);
    
    let workers = env::var("WORKERS")
        .unwrap_or_else(|_| "4".to_string())
        .parse::<usize>()
        .expect("WORKERS must be a number");

    info!("Server listening on {}", addr);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(middleware::Logger::default())
            .wrap(RequestTracing::new())
            .route("/webhook", web::post().to(handlers::receive_webhook))
            .route("/webhook/trivy", web::post().to(trivy_handlers::receive_trivy_webhook))
            .route("/health", web::get().to(handlers::health_check))
            .route("/metrics", web::get().to(handlers::metrics))
    })
    .bind(addr)?
    .workers(workers)
    .run()
    .await
}
