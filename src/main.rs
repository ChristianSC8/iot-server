use std::sync::Arc;
use iot_server::{Config, Database, api, mqtt::MqttClient, AppError};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    println!("Starting IoT Server...");

    let config = Config::from_env()
        .map_err(|e| AppError::Io(std::io::Error::new(std::io::ErrorKind::InvalidInput, e)))?;

    let database = Arc::new(Database::new(&config).await?);
    database.test_connection().await?;

    let api_pool = database.pool.clone();
    let api_port = config.port;
    tokio::spawn(async move {
        if let Err(e) = api::create_server(api_pool, api_port).await {
            eprintln!("API Server zerror: {}", e);
        }
    });

    let mut mqtt_client = MqttClient::new(&config, database).await?;
    mqtt_client.start_listening(&config.mqtt_topic).await?;

    Ok(())
}