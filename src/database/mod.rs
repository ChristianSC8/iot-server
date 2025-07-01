use sqlx::PgPool;
use crate::{AppError, Config};

pub struct Database {
    pub pool: PgPool,
}

impl Database {
    pub async fn new(config: &Config) -> Result<Self, AppError> {
        let pool = PgPool::connect(&config.database_url).await?;
        Ok(Database { pool })
    }

    pub async fn save_sensor_data(&self, data: &serde_json::Value) -> Result<(), AppError> {
        let query = r#"
            INSERT INTO sensor_metrics_manis (mq7_co, mq135_no2, dht11_temperature, dht11_humidity, timestamp)
            VALUES ($1, $2, $3, $4, $5::timestamptz)
        "#;

        let timestamp_str = data["timestamp"]
            .as_str()
            .ok_or_else(|| AppError::Json(serde_json::from_str::<()>("").unwrap_err()))?;

        sqlx::query(query)
            .bind(data["mq7_co"].as_i64())
            .bind(data["mq135_no2"].as_i64())
            .bind(data["dht11_temperature"].as_f64())
            .bind(data["dht11_humidity"].as_f64())
            .bind(timestamp_str)
            .execute(&self.pool)
            .await?;

        println!("Sensor data saved to database");
        Ok(())
    }

    pub async fn test_connection(&self) -> Result<(), AppError> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await?;
        println!("Database connection successful");
        Ok(())
    }
}