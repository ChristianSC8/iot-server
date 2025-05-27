use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::net::TcpListener;
use std::env;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct SensorData {
    pub id: i32,
    pub mq7_co: Option<i32>,
    pub mq135_no2: Option<i32>,
    pub dht11_temperature: Option<f64>,
    pub dht11_humidity: Option<f64>,
    pub timestamp: chrono::DateTime<chrono::Utc>, // ✅ ahora serializable
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub type AppState = Arc<PgPool>;

pub async fn get_latest_record(State(pool): State<AppState>) -> Result<Json<SensorData>, StatusCode> {
    let query = r#"
        SELECT id, mq7_co, mq135_no2, dht11_temperature, dht11_humidity, timestamp, created_at
        FROM sensor_metrics
        ORDER BY timestamp DESC
        LIMIT 1
    "#;

    match sqlx::query_as::<_, SensorData>(query)
        .fetch_one(&*pool)
        .await
    {
        Ok(record) => Ok(Json(record)),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn get_all_records(State(pool): State<AppState>) -> Result<Json<Vec<SensorData>>, StatusCode> {
    let query = r#"
        SELECT id, mq7_co, mq135_no2, dht11_temperature, dht11_humidity, timestamp, created_at
        FROM sensor_metrics
        ORDER BY timestamp DESC
    "#;

    match sqlx::query_as::<_, SensorData>(query)
        .fetch_all(&*pool)
        .await
    {
        Ok(records) => Ok(Json(records)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub fn create_router(pool: PgPool) -> Router {
    let app_state = Arc::new(pool);
    
    Router::new()
        .route("/api/sensors/latest", get(get_latest_record))
        .route("/api/sensors/all", get(get_all_records))
        .with_state(app_state)
}

pub async fn start_api_server(pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
    // Carga la variable de entorno API_PORT, o usa 3000 por defecto si no está definida
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    
    let app = create_router(pool);
    let listener = TcpListener::bind(&addr).await?;

    println!("API REST started at http://{}", addr);
    println!("Available endpoints:");
    println!("   - GET /api/sensors/latest - Latest record");
    println!("   - GET /api/sensors/all - All records");

    axum::serve(listener, app).await?;
    Ok(())
}