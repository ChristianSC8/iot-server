use axum::{
    extract::State,
    response::Json,
};
use sqlx::PgPool;
use std::sync::Arc;
use crate::{AppError, api::models::{SensorData, ApiResponse}};

pub type AppState = Arc<PgPool>;

pub async fn get_latest_record(
    State(pool): State<AppState>
) -> Result<Json<ApiResponse<SensorData>>, AppError> {
    let query = r#"
        SELECT id, mq7_co, mq135_no2, dht11_temperature, dht11_humidity, timestamp
        FROM sensor_metrics_manis
        ORDER BY timestamp DESC
        LIMIT 1
    "#;

    let record = sqlx::query_as::<_, SensorData>(query)
        .fetch_one(&*pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::NotFound,
            _ => AppError::Database(e),
        })?;

    Ok(Json(ApiResponse::success(record)))
}

pub async fn get_all_records(
    State(pool): State<AppState>
) -> Result<Json<ApiResponse<Vec<SensorData>>>, AppError> {
    let query = r#"
        SELECT id, mq7_co, mq135_no2, dht11_temperature, dht11_humidity, timestamp
        FROM sensor_metrics_manis
        ORDER BY timestamp DESC
        LIMIT 100
    "#;

    let records = sqlx::query_as::<_, SensorData>(query)
        .fetch_all(&*pool)
        .await?;

    Ok(Json(ApiResponse::success(records)))
}

pub async fn health_check() -> Json<ApiResponse<String>> {
    Json(ApiResponse::success("Server is running".to_string()))
}