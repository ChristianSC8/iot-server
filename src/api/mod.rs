pub mod handlers;
pub mod models;

use axum::{
    routing::get,
    Router,
};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use crate::AppError;

pub async fn create_server(pool: PgPool, port: u16) -> Result<(), AppError> {
    let app_state = Arc::new(pool);
    
    let app = Router::new()
        .route("/health", get(handlers::health_check))
        .route("/api/sensors/latest", get(handlers::get_latest_record))
        .route("/api/sensors/all", get(handlers::get_all_records))
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;

    println!("API Server started at http://{}", addr);
    println!("Endpoints:");
    println!("   - GET /health");
    println!("   - GET /api/sensors/latest");
    println!("   - GET /api/sensors/all");

    axum::serve(listener, app).await?;
    Ok(())
}