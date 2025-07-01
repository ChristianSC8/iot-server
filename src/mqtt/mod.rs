use rumqttc::{AsyncClient, Event, EventLoop, Incoming, MqttOptions, QoS, TlsConfiguration, Transport};
use rustls::{ClientConfig, RootCertStore};
use rustls_pemfile::certs;
use rustls_pki_types::CertificateDer;
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use std::time::Duration;
use crate::{AppError, Config, Database};

pub struct MqttClient {
    client: AsyncClient,
    eventloop: EventLoop,
    database: Arc<Database>,
}

impl MqttClient {
    pub async fn new(config: &Config, database: Arc<Database>) -> Result<Self, AppError> {
        let tls_config = Self::create_tls_config()?;
        
        let mut mqtt_options = MqttOptions::new(
            &config.mqtt_client_id,
            &config.mqtt_host,
            config.mqtt_port
        );
        
        mqtt_options.set_transport(Transport::Tls(tls_config));
        mqtt_options.set_keep_alive(Duration::from_secs(30));

        let (client, eventloop) = AsyncClient::new(mqtt_options, 10);

        Ok(MqttClient {
            client,
            eventloop,
            database,
        })
    }

    fn create_tls_config() -> Result<TlsConfiguration, AppError> {
        let cert_file = File::open("/usr/local/bin/ca.crt")?;
        let mut reader = BufReader::new(cert_file);

        let certs_vec: Vec<CertificateDer<'static>> = certs(&mut reader)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| AppError::InvalidCertificate)?;

        if certs_vec.is_empty() {
            return Err(AppError::InvalidCertificate);
        }

        let mut root_cert_store = RootCertStore::empty();
        for cert in certs_vec {
            root_cert_store.add(cert)?;
        }

        let client_config = ClientConfig::builder()
            .with_root_certificates(root_cert_store)
            .with_no_client_auth();

        Ok(TlsConfiguration::Rustls(Arc::new(client_config)))
    }

    pub async fn start_listening(&mut self, topic: &str) -> Result<(), AppError> {
        self.client.subscribe(topic, QoS::AtLeastOnce).await?;
        println!("Subscribed to topic: {}", topic);

        loop {
            match self.eventloop.poll().await {
                Ok(Event::Incoming(Incoming::Publish(p))) => {
                    let payload = String::from_utf8_lossy(&p.payload);
                    println!("Message received on {}: {}", p.topic, payload);

                    let data: serde_json::Value = serde_json::from_str(&payload)?;
                    
                    if let Err(e) = self.database.save_sensor_data(&data).await {
                        eprintln!("Error saving to database: {}", e);
                    }
                }
                Ok(Event::Incoming(Incoming::ConnAck(_))) => {
                    println!("TLS connection established");
                }
                Ok(Event::Incoming(Incoming::SubAck(_))) => {
                    println!("Subscription confirmed");
                }
                Ok(Event::Incoming(Incoming::PingResp)) => {
                    println!("Ping response received");
                }
                Ok(_) => {}
                Err(e) => {
                    eprintln!("MQTT connection error: {}", e);
                    break;
                }
            }
        }
        
        Ok(())
    }
}