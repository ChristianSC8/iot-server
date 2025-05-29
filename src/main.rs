use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, QoS, TlsConfiguration, Transport};
use rustls::{ClientConfig, RootCertStore};
use rustls_pemfile::certs;
use rustls_pki_types::CertificateDer;
use sqlx::PgPool;
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use std::time::Duration;
use std::error::Error;
use serde_json::Value;

mod api;
use std::env;
use dotenv::dotenv;

pub async fn get_db_pool() -> PgPool {
    dotenv().ok(); // Carga el archivo .env

    let database_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        env::var("DATABASE_USER").expect("DATABASE_USER not set"),
        env::var("DATABASE_PASSWORD").expect("DATABASE_PASSWORD not set"),
        env::var("DATABASE_HOST").expect("DATABASE_HOST not set"),
        env::var("DATABASE_PORT").expect("DATABASE_PORT not set"),
        env::var("DATABASE_NAME").expect("DATABASE_NAME not set"),
    );

    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database")
}

async fn save_sensor_data(pool: &PgPool, payload: &str) -> Result<(), Box<dyn Error>> {
    let data: Value = serde_json::from_str(payload)?;

    let query = r#"
        INSERT INTO sensor_metrics (mq7_co, mq135_no2, dht11_temperature, dht11_humidity, timestamp)
        VALUES ($1, $2, $3, $4, $5::timestamptz)
    "#;

    // Get timestamp string directly
    let timestamp_str = data["timestamp"].as_str()
        .ok_or("Missing or invalid timestamp field")?;

    sqlx::query(query)
        .bind(data["mq7_co"].as_i64())
        .bind(data["mq135_no2"].as_i64())
        .bind(data["dht11_temperature"].as_f64())
        .bind(data["dht11_humidity"].as_f64())
        .bind(timestamp_str) // Use string with explicit cast in SQL
        .execute(pool)
        .await?;

    println!("Datos guardados en BD");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();

    // Conectar a base de datos
    let pool = get_db_pool().await;
    println!("Conectado a Supabase");

    // Iniciar API REST en una tarea separada
    let api_pool = pool.clone();
    tokio::spawn(async move {
        if let Err(e) = api::start_api_server(api_pool).await {
            eprintln!("❌ Error en servidor API: {}", e);
        }
    });

    // Leer el certificado de la CA
    let cert_file = File::open("/etc/ssl/certs/ca.crt")?;
    let mut reader = BufReader::new(cert_file);

    let certs_vec: Vec<CertificateDer<'static>> = certs(&mut reader)
        .collect::<Result<Vec<_>, _>>()?;

    if certs_vec.is_empty() {
        return Err("No se encontraron certificados en el archivo".into());
    }

    // Crear RootCertStore y añadir certificado
    let mut root_cert_store = RootCertStore::empty();
    for cert in certs_vec {
        root_cert_store.add(cert)?;
    }

    // Crear configuración TLS con validación
    let client_config = ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();

    let tls_config = TlsConfiguration::Rustls(Arc::new(client_config));

    let mut mqtt_options = MqttOptions::new("rust-mqtt-client", "serveo.net", 42082);
    mqtt_options.set_transport(Transport::Tls(tls_config));
    mqtt_options.set_keep_alive(Duration::from_secs(30));

    let (client, mut eventloop) = AsyncClient::new(mqtt_options, 10);

    client.subscribe("esp32/sensors", QoS::AtLeastOnce).await?;
     println!("Subscribed to topic: esp32/sensors");

    loop {
        match eventloop.poll().await {
             Ok(Event::Incoming(Incoming::Publish(p))) => {
                let payload = String::from_utf8_lossy(&p.payload);
                println!("Message received on {}: {}", p.topic, payload);

                if let Err(e) = save_sensor_data(&pool, &payload).await {
                    eprintln!("Error saving to DB: {}", e);
                }
            }
            Ok(Event::Incoming(Incoming::ConnAck(_))) => {
                println!("TLS connection established and certificate validated");
            }
            Ok(Event::Incoming(Incoming::SubAck(_))) => {
                println!("Subscription confirmed");
            }
            Ok(Event::Incoming(Incoming::PingResp)) => {
                println!("Ping response received");
            }
            Ok(other) => {
                println!("Event received: {:?}", other);
            }
            Err(e) => {
                println!("TLS or MQTT connection error: {}", e);
                println!("Check certificates and server connection");
                break;
            }
        }
    }

    Ok(())
}