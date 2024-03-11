use lapin::Connection;

use crate::mq::utils::parse_amqp_settings;
use crate::settings::Settings;

pub async fn new_amqp_connection(settings: &Settings) -> Connection {
    let url = parse_amqp_settings(settings);

    log::debug!("Connecting to AMQP server: {}", url);

    Connection::connect(&url, lapin::ConnectionProperties::default())
        .await
        .unwrap()
}
