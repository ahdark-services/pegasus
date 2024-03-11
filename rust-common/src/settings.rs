use std::fs::File;
use std::io::Read;
use std::path::Path;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Settings {
    pub namespace: String,
    pub version: String,
    pub instance_id: Option<String>,
    // always exist
    pub debug: bool,
    pub telegram_bot: Option<TelegramBot>,
    pub server: Option<Server>,
    pub observability: Option<Observability>,
    pub database: Option<Database>,
    pub redis: Option<Redis>,
    pub mq: Option<Mq>,
}

impl Settings {
    pub fn new(s: &str) -> Settings {
        let mut settings: Settings = serde_yaml::from_str(s).unwrap_or_default();
        if settings.instance_id.is_none() || settings.instance_id.as_ref().unwrap().is_empty() {
            settings.instance_id = Some(Uuid::new_v4().to_string());
        }

        settings
    }

    pub fn read_from_file<P: AsRef<Path>>(path: P) -> Result<Settings, Box<dyn std::error::Error>> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let settings: Settings = Settings::new(&contents);

        Ok(settings)
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseType {
    #[serde(rename = "postgres")]
    Postgres,
    #[serde(rename = "mysql")]
    Mysql,
    #[serde(rename = "mariadb")]
    MariaDB,
    #[serde(rename = "sqlite")]
    Sqlite,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseSSLMode {
    Disable,
    Allow,
    Prefer,
    Require,
    VerifyCA,
    VerifyFull,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Database {
    #[serde(rename = "type")]
    pub database_type: DatabaseType,
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub name: Option<String>,
    pub charset: Option<String>,
    #[serde(rename = "sslmode")]
    pub ssl_mode: Option<DatabaseSSLMode>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Mq {
    pub host: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub vhost: Option<String>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Observability {
    pub trace: Option<Trace>,
    pub metric: Option<Metric>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Metric {
    pub reader: Option<Reader>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum ReaderType {
    #[serde(rename = "prometheus")]
    Prometheus,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Reader {
    #[serde(rename = "type")]
    pub reader_type: Option<ReaderType>,
    pub listen: Option<String>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Trace {
    pub exporter: Exporter,
    pub batch_timeout: Option<String>,
    pub max_batch_entries: Option<i64>,
    pub export_timeout: Option<String>,
    pub max_queue_size: Option<i64>,
    pub sampling_ratio: Option<f64>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum ExporterType {
    #[serde(rename = "otlp-grpc")]
    OtlpGrpc,
    #[serde(rename = "otlp-http")]
    OtlpHttp,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Exporter {
    #[serde(rename = "type")]
    pub exporter_type: Option<ExporterType>,
    pub endpoint: Option<String>,
    pub timeout: Option<String>,
    pub insecure: Option<bool>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RedisMode {
    Standalone,
    Sentinel,
    Cluster,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Redis {
    pub mode: Option<RedisMode>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub db: Option<u8>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Server {
    pub network: Option<String>,
    pub address: Option<String>,
    pub port: Option<u16>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct TelegramBot {
    pub token: String,
    pub webhook: Option<Webhook>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Webhook {
    pub url: Option<String>,
    pub max_connections: Option<i64>,
    pub ip_address: Option<String>,
    pub allowed_updates: Option<Vec<String>>,
    pub drop_pending_updates: Option<bool>,
    pub secret_token: Option<String>,
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_new_settings() {
        {
            let test_file = r#"
                namespace: "pegasus-bot"
                version: "0.0.1"
                #instance_id: ""
                debug: false
                
                telegram_bot:
                  token: ""
                  webhook:
                    url: ""
                    max_connections: 100
                    ip_address: ""
                    allowed_updates:
                      - "message"
                      - "edited_message"
                      - "channel_post"
                      - "edited_channel_post"
                      - "inline_query"
                      - "chosen_inline_result"
                      - "callback_query"
                      - "shipping_query"
                      - "pre_checkout_query"
                      - "poll"
                      - "poll_answer"
                    drop_pending_updates: false
                    secret_token: ""
                
                logging:
                  caller: true
                  trace_id: true
                  stacktrace: error
                  core:
                    - encoder: console
                      target: stdout
                      level: debug
                
                server:
                  network: "tcp"
                  address: "0.0.0.0"
                  port: 8080
                
                observability:
                  trace:
                    exporter:
                      type: "otlp-grpc"
                      endpoint: "localhost:4317"
                      timeout: 10s
                      insecure: true
                    batch_timeout: 5s
                    max_batch_entries: 512
                    export_timeout: 30s
                    max_queue_size: 2048
                    sampling_ratio: 0.1
                  metric:
                    reader:
                      type: prometheus
                      listen: "0.0.0.0:9201"
                
                database:
                  type: postgres
                  host: localhost
                  port: 5432
                  username: pegasus
                  password: pegasus
                  name: pegasus
                  charset: utf8mb4
                  sslmode: disable
                  table_prefix: ""
                
                redis:
                  mode: standalone
                  host: localhost
                  port: 6379
                  username: ""
                  password: "pegasus"
                  db: 0
                
                mq:
                  host: localhost
                  port: 5672
                  username: pegasus
                  password: pegasus
                  vhost: ""
    "#;

            let settings = Settings::new(test_file);
            assert_eq!(settings.namespace, "pegasus-bot");
            assert_eq!(settings.version, "0.0.1");
        }
    }
}
