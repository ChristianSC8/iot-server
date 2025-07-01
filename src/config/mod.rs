use std::env;
use dotenv::dotenv;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub mqtt_host: String,
    pub mqtt_port: u16,
    pub mqtt_client_id: String,
    pub mqtt_topic: String,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        dotenv().ok();

        let database_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            env::var("DATABASE_USER")?,
            env::var("DATABASE_PASSWORD")?,
            env::var("DATABASE_HOST")?,
            env::var("DATABASE_PORT")?,
            env::var("DATABASE_NAME")?,
        );

        Ok(Config {
            database_url,
            port: env::var("PORT")?.parse().unwrap_or(4848),
            mqtt_host: env::var("MQTT_HOST")?,
            mqtt_port: env::var("MQTT_PORT")?.parse().unwrap_or(8883),
            mqtt_client_id: env::var("MQTT_CLIENT_ID")?,
            mqtt_topic: env::var("MQTT_TOPIC")?,
        })
    }
}